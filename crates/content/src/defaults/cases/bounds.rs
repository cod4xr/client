// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, dirty, standard};
use crate::{
    defaults::{DefaultsBudget, DefaultsError, ImportedDefaults},
    pe32::Pe32Error,
};

#[test]
fn image_identity_and_byte_budgets_precede_table_output_or_unreviewed_profile_selection() {
    let (bytes, profile) = standard();
    let mut output = [0xa5; 12];
    let mut slots = dirty(3);
    let old_slots = slots.clone();
    assert_eq!(
        ImportedDefaults::import(&bytes, BUDGET, &mut output, &mut slots)
            .expect_err("Public importer accepts only its audited installed image"),
        DefaultsError::UnknownImage
    );
    for end in 0..bytes.len() {
        assert_eq!(
            ImportedDefaults::with_profile(&bytes[..end], BUDGET, &mut output, &mut slots, profile)
                .expect_err("Exact reviewed size"),
            DefaultsError::UnknownImage
        );
    }
    for position in [0, 0x98, 0x200, 0x400, bytes.len() - 1] {
        let mut changed = bytes.clone();
        changed[position] ^= 1;
        assert_eq!(
            ImportedDefaults::with_profile(&changed, BUDGET, &mut output, &mut slots, profile)
                .expect_err("Same-length modification"),
            DefaultsError::UnknownImage
        );
    }
    assert_eq!(
        ImportedDefaults::with_profile(
            &bytes,
            DefaultsBudget {
                max_image_bytes: bytes.len() - 1,
                ..BUDGET
            },
            &mut output,
            &mut slots,
            profile
        )
        .expect_err("Before hashing"),
        DefaultsError::Image(Pe32Error::ImageBudget)
    );
    assert_eq!(output, [0xa5; 12]);
    assert_eq!(slots, old_slots);
}

#[test]
fn every_short_host_budget_or_caller_capacity_preserves_dirty_output_through_late_values() {
    let (bytes, profile) = standard();
    let mut cases = vec![];
    for count in 0..3 {
        cases.push((
            DefaultsBudget {
                max_records: count,
                ..BUDGET
            },
            12,
            3,
        ));
    }
    for count in 0..12 {
        cases.push((
            DefaultsBudget {
                max_text_bytes: count,
                ..BUDGET
            },
            12,
            3,
        ));
    }
    for count in 0..9 {
        cases.push((
            DefaultsBudget {
                max_value_bytes: count,
                ..BUDGET
            },
            12,
            3,
        ));
    }
    for count in 0..12 {
        cases.push((BUDGET, count, 3));
    }
    for count in 0..3 {
        cases.push((BUDGET, 12, count));
    }
    cases.push((
        DefaultsBudget {
            max_sections: 0,
            ..BUDGET
        },
        12,
        3,
    ));
    for (budget, capacity, records) in cases {
        let mut output = vec![0xa5; capacity];
        let mut slots = dirty(records);
        let old_slots = slots.clone();
        assert!(
            ImportedDefaults::with_profile(&bytes, budget, &mut output, &mut slots, profile)
                .is_err()
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
        assert_eq!(slots, old_slots);
    }
    let mut output = [0; 12];
    let mut slots = dirty(3);
    assert!(
        ImportedDefaults::with_profile(
            &bytes,
            DefaultsBudget {
                max_records: 3,
                max_text_bytes: 12,
                max_value_bytes: 9,
                max_sections: 1,
                max_image_bytes: bytes.len()
            },
            &mut output,
            &mut slots,
            profile
        )
        .is_ok()
    );
}
