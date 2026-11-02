// SPDX-License-Identifier: AGPL-3.0-or-later
//! Effective sparse configs: native block scopes, bounded publication and source release.
#![allow(
    clippy::expect_used,
    reason = "Finite validated sparse-state fixtures."
)]
#[path = "config_state_cases/bounds.rs"]
mod bounds;
#[path = "config_state_cases/model.rs"]
mod model;
#[path = "config_state_cases/semantics.rs"]
mod semantics;
#[path = "config_state_cases/wire.rs"]
mod wire;

use cod4xr_client_core::config_state::{
    ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigStateBudget, ConfigStateSlot, ConfigValue,
};

const BUDGET: ConfigStateBudget = ConfigStateBudget {
    max_input_records: 1024,
    max_input_bytes: 1024 * 1024,
    max_records: 1024,
    max_text_bytes: 1024 * 1024,
};
fn defaults() -> [ConfigValue<'static>; 2] {
    [
        ConfigValue::new(12, b"base%"),
        ConfigValue::new(100, b"setting"),
    ]
}
fn patches() -> [ConfigPatch<'static>; 5] {
    [
        ConfigPatch::new(12, b"first", true),
        ConfigPatch::new(42, b"persist", false),
        ConfigPatch::new(100, b"old setting", false),
        ConfigPatch::new(100, b"", true),
        ConfigPatch::new(u32::MAX, b"wide\x92%", false),
    ]
}
fn dirty_slots(count: usize) -> Vec<ConfigStateSlot> {
    let mut bytes = [0; 3];
    let mut slots = [ConfigStateSlot::default()];
    ConfigSnapshot::build(
        &[],
        &[ConfigPatch::new(2001, b"old", true)],
        BUDGET,
        &mut [ConfigMergeSlot::default()],
        &mut bytes,
        &mut slots,
    )
    .expect("Reused metadata seed");
    vec![slots[0]; count]
}
