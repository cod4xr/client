// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, block, dirty_slots, state};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::configstrings::{ConfigStringBlockPlan, ConfigStringBudget, ConfigStringError},
    message_body::BodyReader,
};

#[test]
fn nonpositive_over_namespace_and_host_counts_fail_before_any_entry_or_output() {
    for (profile, maximum) in [
        (BaselineProtocol::Legacy6, 2442),
        (BaselineProtocol::Cod4x21, 4884),
    ] {
        for count in [-1_i32, 0, maximum + 1, i16::MAX.into()] {
            let input = match profile {
                BaselineProtocol::Legacy6 => i16::try_from(count)
                    .expect("Short fixture")
                    .to_le_bytes()
                    .to_vec(),
                BaselineProtocol::Cod4x21 => count.to_le_bytes().to_vec(),
            };
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            let before = state(&reader);
            let mut output = [0xa5; 1];
            let mut slots = dirty_slots(1);
            let old = slots.clone();
            let result = ConfigStringBlockPlan::parse(
                &mut reader,
                profile,
                BUDGET,
                &mut [],
                &mut output,
                &mut slots,
            );
            assert_eq!(result.err(), Some(ConfigStringError::Count));
            assert_eq!(state(&reader), before);
            assert_eq!(output, [0xa5]);
            assert_eq!(slots, old);
        }
        let input = block(profile, &[(0, b"value")]);
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        assert_eq!(
            ConfigStringBlockPlan::parse(
                &mut reader,
                profile,
                ConfigStringBudget {
                    max_records: 0,
                    ..BUDGET
                },
                &mut [],
                &mut [],
                &mut []
            )
            .err(),
            Some(ConfigStringError::RecordBudget)
        );
    }
}

#[test]
fn late_duplicate_decreasing_and_out_of_namespace_indices_publish_nothing() {
    for (profile, maximum) in [
        (BaselineProtocol::Legacy6, 2442),
        (BaselineProtocol::Cod4x21, 4884),
    ] {
        let mut pairs = vec![(17, 17), (17, 16), (17, maximum), (maximum - 1, maximum)];
        if profile == BaselineProtocol::Cod4x21 {
            pairs.extend([(0, -1), (17, i32::MAX)]);
        }
        for (first, last) in pairs {
            let input = block(profile, &[(first, b"first-private"), (last, b"bad")]);
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            let before = state(&reader);
            let mut output = [0xa5; 64];
            let mut slots = dirty_slots(2);
            let old = slots.clone();
            assert_eq!(
                ConfigStringBlockPlan::parse(
                    &mut reader,
                    profile,
                    BUDGET,
                    &mut [0; 32],
                    &mut output,
                    &mut slots
                )
                .err(),
                Some(ConfigStringError::Index)
            );
            assert_eq!(state(&reader), before);
            assert_eq!(output, [0xa5; 64]);
            assert_eq!(slots, old);
        }
    }
}

#[test]
fn native_gamestate_string_limit_is_stricter_than_general_big_string_limit() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        for length in [8190, 8191, 8192] {
            let text = vec![b'x'; length];
            let input = block(profile, &[(0, b"earlier"), (1, &text)]);
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            let before = state(&reader);
            let mut output = vec![0xa5; 8200];
            let mut slots = dirty_slots(2);
            let old = slots.clone();
            let result = ConfigStringBlockPlan::parse(
                &mut reader,
                profile,
                BUDGET,
                &mut [0; 8192],
                &mut output,
                &mut slots,
            );
            if length == 8190 {
                let plan = result.expect("Maximum native gamestate string");
                assert_eq!(plan.text_bytes(), length + 7);
                assert_eq!(plan.record(1).expect("Last").bytes(), text);
            } else {
                let error = result.expect_err("Native gamestate bound");
                if length == 8191 {
                    assert_eq!(error, ConfigStringError::StringLength);
                } else {
                    assert!(matches!(error, ConfigStringError::Body(_)));
                }
                assert_eq!(state(&reader), before);
                assert!(output.iter().all(|&byte| byte == 0xa5));
                assert_eq!(slots, old);
            }
        }
    }
}
