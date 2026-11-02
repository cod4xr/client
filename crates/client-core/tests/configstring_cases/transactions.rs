// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, LEGACY_VECTOR, block, dirty_slots, state};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::configstrings::{ConfigStringBlockPlan, ConfigStringBudget, ConfigStringError},
    message_body::BodyReader,
};

#[test]
fn every_input_truncation_rolls_back_reader_cache_metadata_and_final_output() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let input = block(profile, &[(0, b"one"), (17, b"two")]);
        for end in 0..input.len() {
            let mut reader = BodyReader::try_new(&input[..end], input.len()).expect("Body");
            let before = state(&reader);
            let mut output = [0xa5; 6];
            let mut slots = dirty_slots(2);
            let old = slots.clone();
            assert!(
                ConfigStringBlockPlan::parse(
                    &mut reader,
                    profile,
                    BUDGET,
                    &mut [0; 3],
                    &mut output,
                    &mut slots
                )
                .is_err()
            );
            assert_eq!(state(&reader), before);
            assert_eq!(output, [0xa5; 6]);
            assert_eq!(slots, old);
        }
    }
    let mut input = vec![0xdd];
    input.extend_from_slice(&block(
        BaselineProtocol::Cod4x21,
        &[(0, b"one"), (17, b"bad")],
    ));
    input.pop(); // Late missing string terminator with an existing unrelated bit cache.
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    assert_eq!(reader.read_bits(3), Ok(5));
    let before = state(&reader);
    let mut output = [0xa5; 6];
    let mut slots = dirty_slots(2);
    let old = slots.clone();
    assert!(
        ConfigStringBlockPlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut [0; 3],
            &mut output,
            &mut slots
        )
        .is_err()
    );
    assert_eq!(state(&reader), before);
    assert_eq!(reader.read_bits(5), Ok(27));
    assert_eq!(output, [0xa5; 6]);
    assert_eq!(slots, old);
}

#[test]
fn every_short_caller_capacity_preserves_final_storage_including_late_text_failures() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let input = block(profile, &[(0, b"one"), (17, b"longer")]);
        for (scratch_len, output_len, slots_len) in (0..6)
            .map(|length| (length, 9, 2))
            .chain((0..9).map(|length| (6, length, 2)))
            .chain((0..2).map(|length| (6, 9, length)))
        {
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            let before = state(&reader);
            let mut output = vec![0xa5; output_len];
            let mut slots = dirty_slots(slots_len);
            let old = slots.clone();
            let error = ConfigStringBlockPlan::parse(
                &mut reader,
                profile,
                BUDGET,
                &mut vec![0; scratch_len],
                &mut output,
                &mut slots,
            )
            .expect_err("Insufficient caller capacity");
            if slots_len < 2 {
                assert_eq!(error, ConfigStringError::RecordCapacity);
            } else if output_len < 9 {
                assert_eq!(error, ConfigStringError::OutputCapacity);
            } else {
                assert!(matches!(error, ConfigStringError::Body(_)));
            }
            assert_eq!(state(&reader), before);
            assert!(output.iter().all(|&byte| byte == 0xa5));
            assert_eq!(slots, old);
        }
    }
}

#[test]
fn text_work_budgets_are_independent_of_storage_and_blocks_reset_index_order() {
    for max_text_bytes in 0..6 {
        let mut reader = BodyReader::try_new(LEGACY_VECTOR, LEGACY_VECTOR.len()).expect("Body");
        let before = state(&reader);
        let mut output = [0xa5; 32];
        let mut slots = dirty_slots(3);
        let old = slots.clone();
        assert_eq!(
            ConfigStringBlockPlan::parse(
                &mut reader,
                BaselineProtocol::Legacy6,
                ConfigStringBudget {
                    max_text_bytes,
                    ..BUDGET
                },
                &mut [0; 3],
                &mut output,
                &mut slots
            )
            .err(),
            Some(ConfigStringError::TextBudget)
        );
        assert_eq!(state(&reader), before);
        assert_eq!(output, [0xa5; 32]);
        assert_eq!(slots, old);
    }
    let mut input = block(BaselineProtocol::Cod4x21, &[(4883, b"first")]);
    input.extend_from_slice(&block(BaselineProtocol::Cod4x21, &[(0, b"later")]));
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    for expected in [4883, 0] {
        let mut slots = dirty_slots(1);
        let mut output = [0; 5];
        let plan = ConfigStringBlockPlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            ConfigStringBudget {
                max_records: 1,
                max_text_bytes: 5,
            },
            &mut [0; 5],
            &mut output,
            &mut slots,
        )
        .expect("Each block has its own ordered namespace");
        assert_eq!(plan.record(0).expect("Entry").index(), expected);
    }
    assert_eq!(reader.remaining_bytes(), 0);
}
