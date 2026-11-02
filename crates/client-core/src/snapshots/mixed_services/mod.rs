// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole command/identity/one-snapshot service carriers, with complete EOF validation.
//! Cooked data is inert; connection-wide history/ACK and engine admission are separate.
mod decode;
mod error;
mod plan;
mod prefix;
mod storage;

pub use error::MixedSnapshotError;
pub use plan::MixedSnapshotPayloadPlan;
pub(crate) use prefix::inspect_header;
pub use storage::MixedSnapshotStorage;
