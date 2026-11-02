// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, ClientStateDecoder, ClientStateDelta, ClientStateError, PROFILES, W, WORDS, decode,
    encoded, header, integer, position, present, raw_float, reference,
};

#[test]
fn independent_physical_vectors_cover_framing_integer_widths_and_empty_prefix() {
    for profile in PROFILES {
        for (bytes, word, value) in [
            (&[0x15, 0][..], W::Model, 0),
            (&[0x15, 0x86, 3, 0xff][..], W::Model, 511),
            (&[0x15, 0x0e, 6, 0xff][..], W::Rank, 255),
            (&[0x15, 2][..], W::Model, 0), // Native accepts delta count zero.
            (&[0xfd, 0][..], W::Model, 0), // Client 63 is not a terminator.
        ] {
            let state = decode(profile, bytes, None);
            assert_eq!(state.number(), u32::from(bytes[0] >> 2));
            assert_eq!(state.word(word), value);
        }
        let mut owner = ClientStateDecoder::new(profile);
        let mut reader = BodyReader::try_new(&[0xfd, 1], 2).expect("Removal");
        assert_eq!(
            owner
                .prepare(&mut reader)
                .expect("Header")
                .expect("63")
                .decode(None),
            Ok(ClientStateDelta::Removed { number: 63 })
        );
        assert!(owner.prepare(&mut reader).expect("End").is_none());
    }
}

#[test]
fn all_fields_keep_native_widths_semantic_order_and_complete_prior_xor() {
    for profile in PROFILES {
        let prior = reference(17, 0xa5a5_a5a5);
        let bytes = encoded(|writer| {
            header(writer, 17, Some(24));
            for &(word, width) in &WORDS {
                if width == 0 {
                    raw_float(writer, prior.word(word), 0x7fc0_4321);
                } else {
                    integer(
                        writer,
                        width,
                        prior.word(word),
                        0x5a5a_5a5a & (u32::MAX >> (32 - width)),
                    );
                }
            }
        });
        let result = decode(profile, &bytes, Some(&prior));
        for &(word, width) in &WORDS {
            assert_eq!(
                result.word(word),
                if width == 0 {
                    0x7fc0_4321
                } else {
                    0x5a5a_5a5a & (u32::MAX >> (32 - width))
                }
            );
        }
        assert_eq!(prior.word(W::Perks), 0xa5a5_a5a5);
    }
}

#[test]
fn packed_names_preserve_nuls_percent_high_bytes_and_avoid_debug_disclosure() {
    let name = *b"P%\xff\0RIVATE\x80NAME!";
    let bytes = encoded(|writer| {
        header(writer, 2, Some(11));
        for &(word, width) in &WORDS[..11] {
            if let Some(index) = [W::Name0, W::Name1, W::Name2, W::Name3]
                .iter()
                .position(|w| *w == word)
            {
                integer(
                    writer,
                    width,
                    0,
                    u32::from_le_bytes(name[index * 4..index * 4 + 4].try_into().expect("Word")),
                );
            } else {
                writer.write_bits(0, 1).expect("Unchanged");
            }
        }
    });
    let state = decode(PROFILES[1], &bytes, None);
    assert_eq!(state.name_bytes(), name);
    let debug = format!("{:?}", ClientStateDelta::State(state));
    assert!(!debug.contains("RIVATE"));
    assert!(!debug.contains("80"));
}

#[test]
fn float_forms_distinguish_raw_xor_signed_zero_and_compact_truncated_prior() {
    for profile in PROFILES {
        for (prior_bits, payload, expected) in [
            (0x7fc0_1234, &[0][..], 0),
            (0x7fc0_1234, &[2][..], 0x8000_0000),
            (1.75_f32.to_bits(), &[0x7d, 0xff][..], (-2.0_f32).to_bits()),
            ((-1.75_f32).to_bits(), &[0x75, 0xff][..], 2.0_f32.to_bits()),
            (
                1.5_f32.to_bits(),
                &[3, 0x34, 0x12, 0, 0x40][..],
                0x7fc0_1234,
            ),
        ] {
            // Rebase an independent shared-value byte vector into cached framing.
            let prior = reference(4, prior_bits);
            let bytes = encoded(|writer| {
                header(writer, 4, Some(13));
                for _ in 0..12 {
                    writer.write_bits(0, 1).expect("Unchanged prefix");
                }
                writer.write_bits(1, 1).expect("Float changed");
                let width = if payload[0] & 3 == 1 { 7 } else { 2 };
                writer
                    .write_bits(u32::from(payload[0]), width)
                    .expect("Independent float bits");
                for &byte in &payload[1..] {
                    writer.write_u8(byte).expect("Independent float bytes");
                }
            });
            assert_eq!(
                decode(profile, &bytes, Some(&prior)).word(W::SprintTimeMultiplier),
                expected
            );
        }
    }
}

#[test]
fn unchanged_prefix_no_delta_and_zero_count_keep_every_prior_word() {
    let prior = reference(10, 0x7fc0_1234);
    for count in [None, Some(0), Some(24)] {
        let bytes = encoded(|writer| {
            header(writer, 10, count);
            for _ in 0..count.unwrap_or(0) {
                writer.write_bits(0, 1).expect("Unchanged");
            }
        });
        assert_eq!(decode(PROFILES[1], &bytes, Some(&prior)), prior);
    }
    let bytes = encoded(|writer| {
        header(writer, 10, Some(1));
        integer(writer, 9, prior.word(W::Model), 0);
    });
    let state = decode(PROFILES[1], &bytes, Some(&prior));
    assert_eq!(state.word(W::Model), 0);
    for &(word, _) in &WORDS[1..] {
        assert_eq!(state.word(word), prior.word(word));
    }
}

#[test]
fn cached_bits_end_bit_and_following_bytes_keep_independent_cursors() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prior bit field");
        header(writer, 7, Some(1));
        integer(writer, 9, 0, 341);
        writer.write_bits(0, 1).expect("End");
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following cached bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prior bits"), 5);
    let mut owner = ClientStateDecoder::new(PROFILES[1]);
    assert_eq!(
        present(
            owner
                .prepare(&mut reader)
                .expect("Header")
                .expect("Record")
                .decode(None)
                .expect("State")
        )
        .word(W::Model),
        341
    );
    assert!(owner.prepare(&mut reader).expect("End").is_none());
    let ended = position(&reader);
    assert!(matches!(
        owner.prepare(&mut reader),
        Err(ClientStateError::StreamEnded)
    ));
    assert_eq!(position(&reader), ended);
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
    owner.reset();
    assert_eq!(owner.profile(), PROFILES[1]);
}
