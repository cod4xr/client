// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole-message baseline service-control validation without command execution.
//! Gamestate/snapshots/downloads need their own decoders and are not skipped here.

mod client_plan;
pub(crate) mod client_scan;
mod client_storage;
mod command;
mod error;
mod plan;
pub(crate) mod publish;
pub(crate) mod scan;

pub use client_storage::{ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage};
pub use command::ServerCommand;
pub use error::ServiceControlError;
pub use plan::LegacyServicePlan;

/// Baseline reliable server-command window; not an engine-wide script/queue limit.
pub const MAX_BASELINE_SERVER_COMMANDS: usize = 128;

/// Host inspection budget for one decoded service-control stream.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServiceControlBudget {
    /// Maximum records inspected, including NOPs, commands and the terminating EOF.
    pub max_records: u32,
}

impl Default for ServiceControlBudget {
    fn default() -> Self {
        Self { max_records: 1024 }
    }
}
