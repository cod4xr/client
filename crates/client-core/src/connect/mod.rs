// SPDX-License-Identifier: AGPL-3.0-or-later
//! Client-owned no-auth connect policy over shared info-string and envelope codecs.
mod attempt;
mod attempt_budget;
mod attempt_error;
mod attempt_outcome;
mod attempt_usage;
mod error;
mod number_text;
mod plan;
mod reply;
mod reply_error;
mod settings;

pub use attempt::NoAuthConnectAttempt;
pub use attempt_budget::ConnectBudget;
pub use attempt_error::ConnectFailure;
pub use attempt_outcome::{ConnectAction, ConnectObservation, ConnectProgress};
pub use attempt_usage::{ConnectRejection, ConnectUsage};
pub use error::ConnectPlanError;
pub use plan::{MAX_CONNECT_PACKET_BYTES, NoAuthConnectPlan};
pub use reply::{ConnectReply, ConnectReplyKind, MAX_CONNECT_REPLY_BYTES};
pub use reply_error::ConnectReplyError;
pub use settings::NoAuthConnectSettings;
