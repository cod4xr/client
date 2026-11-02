// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, dirty, fixture, rehash, standard, word};
use crate::defaults::{DefaultsBudget, DefaultsError, ImportedDefaults};

#[test]
fn rehashed_synthetic_layout_errors_reject_order_reserved_fields_addresses_and_missing_terminators()
{
    let (source, original_profile) = standard();
    for (offset, value) in [
        (0x210, 12),
        (0x210, 11),
        (0x210, 0),
        (0x220, 2442),
        (0x220, u32::MAX),
        (0x228, 1),
        (0x22c, 1),
        (0x224, 0x0050_0000),
        (0x230, 1),
    ] {
        let mut input = source.clone();
        word(&mut input, offset, value);
        let mut profile = original_profile;
        rehash(&input, &mut profile);
        let mut output = [0xa5; 12];
        let mut slots = dirty(3);
        let before = slots.clone();
        assert!(
            ImportedDefaults::with_profile(&input, BUDGET, &mut output, &mut slots, profile)
                .is_err()
        );
        assert_eq!(output, [0xa5; 12]);
        assert_eq!(slots, before);
    }
    for property in 0..4 {
        let mut input = source.clone();
        let mut profile = original_profile;
        match property {
            0 => profile.text_bytes += 1,
            1 => profile.image_base += 0x1000,
            2 => profile.longest_value += 1,
            _ => input[0x405..].fill(b'x'), // Late missing terminator through the file-backed tail.
        }
        rehash(&input, &mut profile);
        let mut output = [0xa5; 13];
        let mut slots = dirty(3);
        let before = slots.clone();
        assert!(
            ImportedDefaults::with_profile(&input, BUDGET, &mut output, &mut slots, profile)
                .is_err()
        );
        assert_eq!(output, [0xa5; 13]);
        assert_eq!(slots, before);
    }
    let mut input = source;
    let mut profile = original_profile;
    profile.table_address = 0x0040_13f8;
    word(&mut input, 0x5f8, 12); // The first field exists, but its 16-byte row crosses raw storage.
    rehash(&input, &mut profile);
    assert!(
        ImportedDefaults::with_profile(&input, BUDGET, &mut [0xa5; 12], &mut dirty(3), profile)
            .is_err()
    );
}

#[test]
fn installed_raw_defaults_do_not_inherit_wire_string_limits_or_percent_normalization() {
    let value = vec![b'%'; 9000];
    let (input, profile) = fixture(&[(12, &value)]);
    let mut output = vec![0xa5; value.len()];
    let mut slots = dirty(1);
    let before = slots.clone();
    assert_eq!(
        ImportedDefaults::with_profile(
            &input,
            DefaultsBudget {
                max_value_bytes: 8999,
                ..BUDGET
            },
            &mut output,
            &mut slots,
            profile
        )
        .expect_err("Explicit per-value budget"),
        DefaultsError::ValueBudget
    );
    assert!(output.iter().all(|&byte| byte == 0xa5));
    assert_eq!(slots, before);
    let plan = ImportedDefaults::with_profile(
        &input,
        DefaultsBudget {
            max_value_bytes: 9000,
            ..BUDGET
        },
        &mut output,
        &mut slots,
        profile,
    )
    .expect("No legacy wire-string cap in installed data");
    assert_eq!(plan.text_bytes(), 9000);
    assert_eq!(
        plan.records().next().expect("Long raw value").bytes(),
        value
    );
}
