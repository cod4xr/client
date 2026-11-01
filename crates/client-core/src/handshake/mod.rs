// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit challenge-probe state, without socket ownership or gameplay admission.
mod budget;
mod error;
mod probe;
mod receipt;
mod usage;

pub use budget::ChallengeBudget;
pub use error::{ChallengeFailure, ChallengeRejection};
pub use probe::{ChallengeAction, ChallengeProbe, ChallengeProgress};
pub use receipt::ChallengeReceipt;
pub use usage::ChallengeUsage;
