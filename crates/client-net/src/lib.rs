// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned, bounded blocking client network phases. Run these on an I/O worker.
//! Acknowledgement is pre-channel state; no playable connection is claimed.

mod challenge;
mod connect;
pub mod datagram;
mod error;
mod outcome;
mod peer;

pub use challenge::{ChallengeRun, ChallengedSocket, challenge};
pub use connect::{ConnectRun, PendingConnect};
pub use error::NetworkError;
pub use outcome::{AcknowledgedSocket, ConnectOutcome, ConnectSummary};
