// SPDX-License-Identifier: AGPL-3.0-or-later
mod error;
mod plan;
mod sink;
mod storage;
mod walk;

pub use error::GameStateError;
pub use plan::GameStatePlan;
pub use storage::{ClientDataSlot, EntityBaselineSlot, GameStateStorage};

/// Explicit host work/retention budgets for a complete gamestate constituent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GameStateBudget {
    /// Maximum service opcodes inspected, including the terminating EOF.
    pub max_services: usize,
    /// Maximum explicit configstring patches, across all blocks.
    pub max_configstrings: usize,
    /// Maximum combined normalized configstring bytes, without NULs.
    pub max_text_bytes: usize,
    /// Maximum entity baseline records.
    pub max_entities: usize,
    /// Maximum client identity records, including repeated identifiers.
    pub max_client_records: usize,
}

#[derive(Clone, Copy, Default)]
struct Counts {
    services: usize,
    configstrings: usize,
    text_bytes: usize,
    entities: usize,
    clients: usize,
}

struct Layout {
    counts: Counts,
    server_command_sequence: u32,
    config_data_sequence: u32,
    client_index: u32,
    checksum_feed: u32,
    consumed_bytes: usize,
}
