// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, FIELDS, HUD_SLOTS, HUD_WORDS, MODERN, SnapshotHud, W, array, encoded, integer,
    legacy, parse,
};

#[test]
fn independent_empty_array_vector_keeps_following_byte_and_cached_bits() {
    for context in [MODERN, legacy([0.0; 3])] {
        // Changed outer bit followed by two zero five-bit counts; next five bits=21.
        let mut reader = BodyReader::try_new(&[1, 0xa8, 0xa7], 3).expect("Physical vector");
        let result = SnapshotHud::parse(&mut reader, context, 0, None).expect("Empty arrays");
        assert_eq!(result, SnapshotHud::zero(context));
        assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
        assert_eq!(reader.read_bits(5).expect("Following cached bits"), 21);
        assert_eq!(HUD_SLOTS, 31);
        assert_eq!(HUD_WORDS, 40);
    }
}

#[test]
fn inclusive_zero_last_index_decodes_one_color_field_not_an_empty_prefix() {
    for context in [MODERN, legacy([0.0; 3])] {
        // Outer=1,count=1,last=0,color-changed=1,alpha-shortcut=1,current-count=0.
        let result = parse(context, 0, &[3, 0x30, 0], None);
        assert_eq!(result.archival()[0].word(W::Color), 0xff00_0000);
        assert_eq!(result.archival()[0].word(W::Kind), 0);
        assert!(
            result
                .current()
                .iter()
                .all(|element| element.word(W::Color) == 0)
        );
    }
}

#[test]
fn archival_and_current_arrays_have_independent_references_and_native_order() {
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("HUD changed");
        for value in [17, 93] {
            array(writer, 1, 7, |_, index, writer| {
                if FIELDS[index].0 == W::Width {
                    integer(writer, 10, 0, value);
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            });
        }
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let first = SnapshotHud::parse(&mut reader, MODERN, 0, None).expect("Two arrays");
    assert_eq!(first.archival()[0].word(W::Width), 17);
    assert_eq!(first.current()[0].word(W::Width), 93);
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
    let bytes = encoded(|writer| {
        writer.write_bits(1, 1).expect("HUD changed");
        for (from, to) in [(17, 511), (93, 7)] {
            array(writer, 1, 7, |_, index, writer| {
                if FIELDS[index].0 == W::Width {
                    integer(writer, 10, from, to);
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            });
        }
    });
    let next = parse(MODERN, 0, &bytes, Some(&first));
    assert_eq!(next.archival()[0].word(W::Width), 511);
    assert_eq!(next.current()[0].word(W::Width), 7);
}

#[test]
fn all_thirty_one_slots_in_both_arrays_use_existing_cached_bits_without_alignment() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Cached prefix");
        writer.write_bits(1, 1).expect("HUD changed");
        for base in [10, 100] {
            array(writer, 31, 39, |slot, index, writer| {
                if FIELDS[index].0 == W::Width {
                    integer(writer, 10, 0, base + u32::try_from(slot).expect("Slot"));
                } else {
                    writer.write_bits(0, 1).expect("Unchanged");
                }
            });
        }
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    let result = SnapshotHud::parse(&mut reader, MODERN, 0, None).expect("All slots");
    assert_eq!(result.archival()[30].word(W::Width), 40);
    assert_eq!(result.current()[30].word(W::Width), 130);
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
