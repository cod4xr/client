// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, defaults, dirty_slots, patches};
use cod4xr_client_core::config_state::{
    ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigStateBudget, ConfigStateError, ConfigValue,
};

#[test]
fn every_short_work_budget_scratch_and_output_capacity_preserves_final_storage() {
    let mut cases = vec![];
    for value in 0..7 {
        cases.push((
            ConfigStateBudget {
                max_input_records: value,
                ..BUDGET
            },
            7,
            18,
            4,
        ));
    }
    for value in 0..41 {
        cases.push((
            ConfigStateBudget {
                max_input_bytes: value,
                ..BUDGET
            },
            7,
            18,
            4,
        ));
    }
    for value in 0..4 {
        cases.push((
            ConfigStateBudget {
                max_records: value,
                ..BUDGET
            },
            7,
            18,
            4,
        ));
    }
    for value in 0..18 {
        cases.push((
            ConfigStateBudget {
                max_text_bytes: value,
                ..BUDGET
            },
            7,
            18,
            4,
        ));
    }
    for value in 0..7 {
        cases.push((BUDGET, value, 18, 4));
    }
    for value in 0..18 {
        cases.push((BUDGET, 7, value, 4));
    }
    for value in 0..4 {
        cases.push((BUDGET, 7, 18, value));
    }
    for (budget, scratch_len, bytes, records) in cases {
        let mut output = vec![0xa5; bytes];
        let mut slots = dirty_slots(records);
        let before = slots.clone();
        let mut scratch = vec![ConfigMergeSlot::default(); scratch_len];
        assert!(
            ConfigSnapshot::build(
                &defaults(),
                &patches(),
                budget,
                &mut scratch,
                &mut output,
                &mut slots
            )
            .is_err()
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
        assert_eq!(slots, before);
    }
    let exact = ConfigStateBudget {
        max_input_records: 7,
        max_input_bytes: 41,
        max_records: 4,
        max_text_bytes: 18,
    };
    assert!(
        ConfigSnapshot::build(
            &defaults(),
            &patches(),
            exact,
            &mut [ConfigMergeSlot::default(); 7],
            &mut [0; 18],
            &mut dirty_slots(4)
        )
        .is_ok()
    );
}

#[test]
fn malformed_default_order_patch_scopes_and_embedded_nuls_never_publish_partial_state() {
    let cases = [
        (
            vec![ConfigValue::new(2, b"a"), ConfigValue::new(2, b"b")],
            vec![],
            ConfigStateError::DefaultOrder,
        ),
        (
            vec![ConfigValue::new(2, b"a"), ConfigValue::new(1, b"b")],
            vec![],
            ConfigStateError::DefaultOrder,
        ),
        (
            vec![],
            vec![ConfigPatch::new(1, b"Private", false)],
            ConfigStateError::MissingBlockStart,
        ),
        (
            vec![],
            vec![
                ConfigPatch::new(2, b"a", true),
                ConfigPatch::new(2, b"Private", false),
            ],
            ConfigStateError::PatchOrder,
        ),
        (
            vec![],
            vec![
                ConfigPatch::new(2, b"a", true),
                ConfigPatch::new(1, b"Private", false),
            ],
            ConfigStateError::PatchOrder,
        ),
        (
            vec![ConfigValue::new(1, b"Private\0bad")],
            vec![],
            ConfigStateError::EmbeddedNul,
        ),
        (
            vec![],
            vec![ConfigPatch::new(1, b"Private\0bad", true)],
            ConfigStateError::EmbeddedNul,
        ),
    ];
    for (defaults, patches, expected) in cases {
        let mut output = [0xa5; 32];
        let mut slots = dirty_slots(3);
        let before = slots.clone();
        let error = ConfigSnapshot::build(
            &defaults,
            &patches,
            BUDGET,
            &mut [ConfigMergeSlot::default(); 8],
            &mut output,
            &mut slots,
        )
        .expect_err("Complete semantic admission");
        assert_eq!(error, expected);
        assert!(!format!("{error:?} {error}").contains("Private"));
        assert_eq!(output, [0xa5; 32]);
        assert_eq!(slots, before);
    }
}

#[test]
fn reused_private_descriptors_and_unused_tails_cannot_redirect_source_lookup() {
    let mut scratch = [ConfigMergeSlot::default(); 4];
    let old = [
        ConfigPatch::new(1000, b"old", true),
        ConfigPatch::new(0, b"other", true),
    ];
    ConfigSnapshot::build(
        &[],
        &old,
        BUDGET,
        &mut scratch,
        &mut [0; 8],
        &mut dirty_slots(2),
    )
    .expect("Old scratch sort puts later source index first");
    let mut output = [0xa5; 8];
    let mut slots = dirty_slots(3);
    let unused = slots[2];
    let current = [ConfigPatch::new(u32::MAX, b"new", true)];
    let result =
        ConfigSnapshot::build(&[], &current, BUDGET, &mut scratch, &mut output, &mut slots)
            .expect("All active scratch rewritten");
    assert_eq!(result.value(u32::MAX), Some(&b"new"[..]));
    assert_eq!(result.value(0), None);
    assert_eq!(result.records().len(), 1);
    assert_eq!(&output[3..], &[0xa5; 5]);
    assert_eq!(slots[2], unused);
}
