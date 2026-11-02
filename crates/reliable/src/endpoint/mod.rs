// SPDX-License-Identifier: AGPL-3.0-or-later
//! One bounded reliable endpoint over caller storage, without sockets or command execution.
//! Routing/profile admission precedes ingestion; complete frame bodies remain untrusted.

mod drain;
mod error;
mod input;
mod state;
mod storage;

pub use drain::{DrainRun, DrainStatus, DrainUsage};
pub use error::EndpointError;
pub use input::IngestReport;
pub use state::{EndpointState, ReliableEndpoint};
pub use storage::{EndpointConfig, EndpointStorage};
