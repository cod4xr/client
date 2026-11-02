// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed identity-state admission failures, without names, tags or source bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientIdentityError {
    /// Only the explicit protocol-21 identity carrier is currently supported.
    Profile,
    /// No gamestate baseline has been committed for this owner.
    MissingBaseline,
    /// Input record count exceeds host inspection policy.
    RecordBudget,
    /// Distinct retained clients exceed host retention policy.
    ClientBudget,
    /// Private staging cells cannot hold the proposed retained state.
    ScratchCapacity,
    /// Final caller cells cannot hold the proposed retained state.
    OutputCapacity,
    /// Local baseline generation cannot increment without reuse.
    GenerationExhausted,
}
impl fmt::Display for ClientIdentityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Profile => "client identity state requires the reviewed protocol-21 profile",
            Self::MissingBaseline => "client identity state has no committed gamestate baseline",
            Self::RecordBudget => "client identity record budget exhausted",
            Self::ClientBudget => "client identity retention budget exhausted",
            Self::ScratchCapacity => "client identity private staging capacity insufficient",
            Self::OutputCapacity => "client identity final capacity insufficient",
            Self::GenerationExhausted => "client identity baseline generation exhausted",
        })
    }
}
impl Error for ClientIdentityError {}
