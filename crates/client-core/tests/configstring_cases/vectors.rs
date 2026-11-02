// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, LEGACY_VECTOR, block, dirty_slots};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::configstrings::{ConfigStringBlockPlan, ConfigStringSlot},
    message_body::BodyReader,
};

#[test]
fn independent_legacy_vector_keeps_bit_cache_across_strings_and_following_opcode() {
    for padding in [0, MAX_MESSAGE_BYTES - 1, MAX_MESSAGE_BYTES + 7] {
        let mut input = vec![0xfe; padding];
        input.extend_from_slice(LEGACY_VECTOR);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Complete body");
        reader.read_bytes(&mut vec![0; padding]).expect("Prefix");
        let mut scratch = [0; 3];
        let mut output = [0xa5; 8];
        let mut slots = dirty_slots(3);
        let unused = slots[2];
        let plan = ConfigStringBlockPlan::parse(
            &mut reader,
            BaselineProtocol::Legacy6,
            BUDGET,
            &mut scratch,
            &mut output,
            &mut slots,
        )
        .expect("Native increment/12-bit fields");
        let records: Vec<_> = plan.records().collect();
        assert_eq!(plan.profile(), BaselineProtocol::Legacy6);
        assert_eq!(plan.text_bytes(), 6);
        assert_eq!(records.len(), 2);
        assert!(records[0].starts_block());
        assert!(!records[1].starts_block());
        assert_eq!((records[0].index(), records[0].bytes()), (0, &b"one"[..]));
        assert_eq!((records[1].index(), records[1].bytes()), (17, &b"two"[..]));
        assert!(plan.record(2).is_none());
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(2), Ok(3));
        assert_eq!(reader.remaining_bytes(), 0);
        assert_eq!(output, *b"onetwo\xa5\xa5");
        assert_eq!(slots[2], unused);
    }
}

#[test]
fn independent_modern_vector_is_owned_normalized_raw_and_private_in_debug() {
    let mut scratch = [0; 64];
    let mut output = [0xa5; 32];
    let mut slots = [ConfigStringSlot::default(); 3];
    let plan = {
        let input = [
            2, 0, 0, 0, 0, 0, 0, 0, b'a', b'%', 0x92, 0, 0x13, 0x13, 0, 0, 0, 7,
        ];
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        assert_eq!(u32::from_le_bytes([0x13, 0x13, 0, 0]), 4883);
        let plan = ConfigStringBlockPlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut scratch,
            &mut output,
            &mut slots,
        )
        .expect("Independent 32-bit fields");
        assert_eq!(reader.read_u8(), Ok(7));
        plan // Neither immutable input nor reader lifetime is retained.
    };
    assert_eq!(plan.record_count(), 2);
    assert_eq!(plan.text_bytes(), 3);
    assert_eq!(plan.record(0).expect("First").bytes(), b"a.\x92");
    assert_eq!(plan.record(1).expect("Last").index(), 4883);
    assert!(plan.record(1).expect("Last").bytes().is_empty());
    let debug = format!("{plan:?} {:?}", plan.record(0));
    assert!(!debug.contains("a."));
    assert!(!debug.contains("146"));
    assert!(!debug.contains("[97"));
}

#[test]
fn complete_native_namespaces_fit_caller_metadata_and_accept_empty_values() {
    for (profile, count) in [
        (BaselineProtocol::Legacy6, 2442),
        (BaselineProtocol::Cod4x21, 4884),
    ] {
        let entries: Vec<_> = (0..count).map(|index| (index, &b""[..])).collect();
        let input = block(profile, &entries);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let mut slots = vec![ConfigStringSlot::default(); entries.len()];
        let plan = ConfigStringBlockPlan::parse(
            &mut reader,
            profile,
            BUDGET,
            &mut [],
            &mut [],
            &mut slots,
        )
        .expect("Full namespace, no fixed arena or remote allocation");
        assert_eq!(plan.record_count(), entries.len());
        assert_eq!(plan.text_bytes(), 0);
        assert_eq!(plan.records().len(), entries.len());
        assert_eq!(plan.record(0).expect("First").index(), 0);
        assert_eq!(
            plan.record(entries.len() - 1).expect("Last").index(),
            u32::try_from(count - 1).expect("Last ID")
        );
        assert_eq!(reader.remaining_bytes(), 0);
    }
}

#[test]
fn large_complete_block_crosses_shared_message_view_size_without_global_string_arena() {
    let mut input = 20_i32.to_le_bytes().to_vec();
    for index in 0_i32..20 {
        input.extend_from_slice(&index.to_le_bytes());
        input.extend_from_slice(&[b'x'; 8190]);
        input.push(0);
    }
    input.push(7);
    assert!(input.len() > MAX_MESSAGE_BYTES);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Caller body budget");
    let mut slots = [ConfigStringSlot::default(); 20];
    let mut output = vec![0; 20 * 8190];
    let plan = ConfigStringBlockPlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        BUDGET,
        &mut [0; 8190],
        &mut output,
        &mut slots,
    )
    .expect("No native global arena inherited by the decoder");
    assert_eq!(plan.text_bytes(), 20 * 8190);
    for record in plan.records() {
        assert_eq!(record.bytes(), &[b'x'; 8190]);
    }
    assert_eq!(reader.read_u8(), Ok(7));
}
