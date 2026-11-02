// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit-clock reliable pacing. One successful callback admits one datagram.
//! No OS, sleeps, catch-up bursts or automatic retry loops are owned here.

mod config;
mod error;
mod observation;
mod plan;
mod poll;
mod scheduler;

pub use config::ScheduleConfig;
pub use error::ScheduleError;
pub use observation::PreparedPacketObservation;
pub use plan::{ScheduleDecision, SchedulePoll, ScheduledDatagram, ScheduledSend, SendAction};
pub use scheduler::ReliableScheduler;
