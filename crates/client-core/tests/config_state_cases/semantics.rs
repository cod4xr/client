// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, defaults, dirty_slots, patches};
use cod4xr_client_core::config_state::{
    ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigStateSlot, ConfigValue,
};

#[test]
fn native_defaults_restore_per_block_while_sparse_nondefault_values_persist() {
    let mut output = [0xa5; 20];
    let mut slots = dirty_slots(5);
    let unused = slots[4];
    let mut scratch = [ConfigMergeSlot::default(); 8];
    let snapshot = {
        let owned = [b"base%".to_vec(), b"setting".to_vec()];
        let source = [
            ConfigValue::new(12, &owned[0]),
            ConfigValue::new(100, &owned[1]),
        ];
        let patches = patches();
        ConfigSnapshot::build(
            &source,
            &patches,
            BUDGET,
            &mut scratch,
            &mut output,
            &mut slots,
        )
        .expect("Complete sparse native state")
    };
    scratch.fill(ConfigMergeSlot::default()); // Scratch retains no source lifetimes.
    assert_eq!(snapshot.value(12), Some(&b"base%"[..]));
    assert_eq!(snapshot.value(42), Some(&b"persist"[..]));
    assert_eq!(snapshot.value(100), Some(&b""[..]));
    assert_eq!(snapshot.value(u32::MAX), Some(&b"wide\x92%"[..]));
    assert_eq!(snapshot.value(0), None);
    assert_eq!(snapshot.value(u32::MAX - 1), None);
    assert_eq!(
        snapshot
            .records()
            .map(ConfigValue::index)
            .collect::<Vec<_>>(),
        [12, 42, 100, u32::MAX]
    );
    let stats = snapshot.stats();
    assert_eq!(
        (
            stats.input_records,
            stats.input_bytes,
            stats.blocks,
            stats.records,
            stats.text_bytes
        ),
        (7, 41, 2, 4, 18)
    );
    assert!(!format!("{snapshot:?}").contains("base"));
    assert!(!format!("{:?}", snapshot.records().last()).contains("wide"));
    assert_eq!(&output[18..], &[0xa5; 2]);
    assert_eq!(slots[4], unused);
}

#[test]
fn no_native_blocks_produce_empty_state_and_patch_only_blocks_keep_last_explicit_values() {
    let mut output = [0xa5; 4];
    let mut slots = dirty_slots(3);
    let before = slots.clone();
    let empty = ConfigSnapshot::build(
        &defaults(),
        &[],
        BUDGET,
        &mut [ConfigMergeSlot::default(); 2],
        &mut output,
        &mut slots,
    )
    .expect("No default reapplication without blocks");
    assert_eq!(empty.stats().records, 0);
    assert_eq!(empty.stats().blocks, 0);
    assert_eq!(empty.value(12), None);
    assert_eq!(output, [0xa5; 4]);
    assert_eq!(slots, before);
    let patches = [
        ConfigPatch::new(0, b"a", true),
        ConfigPatch::new(u32::MAX, b"b", false),
        ConfigPatch::new(0, b"c", true),
    ];
    let result = ConfigSnapshot::build(
        &[],
        &patches,
        BUDGET,
        &mut [ConfigMergeSlot::default(); 3],
        &mut output,
        &mut slots,
    )
    .expect("Sparse patch-only state");
    assert_eq!(result.value(0), Some(&b"c"[..]));
    assert_eq!(result.value(u32::MAX), Some(&b"b"[..]));
    let zero =
        ConfigSnapshot::build(&[], &[], BUDGET, &mut [], &mut [], &mut []).expect("Empty sources");
    assert_eq!(zero.records().len(), 0);
}

#[test]
fn large_raw_values_and_empty_overrides_do_not_inherit_legacy_field_or_global_arena_limits() {
    let source = vec![b'%'; 100_000];
    let defaults = [ConfigValue::new(3, &source)];
    let patches = [ConfigPatch::new(9, b"\x92%", true)];
    let mut output = vec![0; source.len() + 2];
    let mut slots = [ConfigStateSlot::default(); 2];
    let result = ConfigSnapshot::build(
        &defaults,
        &patches,
        BUDGET,
        &mut [ConfigMergeSlot::default(); 2],
        &mut output,
        &mut slots,
    )
    .expect("Explicit host byte budgets");
    assert_eq!(result.value(3), Some(source.as_slice()));
    assert_eq!(result.value(9), Some(&b"\x92%"[..]));
    assert_eq!(result.stats().text_bytes, 100_002);
}
