// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::client_data::ClientDataUpdate;
use std::fmt;

/// Caller-owned cell containing a complete normalized update after whole-plan success.
/// Cells have no independent public record/sequence admission authority.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ClientDataUpdateSlot(pub(super) Option<ClientDataUpdate>);
impl fmt::Debug for ClientDataUpdateSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientDataUpdateSlot")
            .finish_non_exhaustive()
    }
}
/// Independent inspection/identity counts for mixed protocol-21 control messages.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientServiceBudget {
    /// All inspected records, including NOPs, commands, identity updates and EOF.
    pub max_records: u32,
    /// Maximum inspected identity records, independently of distinct players or storage.
    pub max_client_updates: usize,
}
/// Distinct private/final capacities; no output changes before complete logical EOF.
pub struct ClientServiceStorage<'scratch, 'output> {
    /// Private owned-update staging, unspecified after any failure.
    pub client_scratch: &'scratch mut [ClientDataUpdateSlot],
    /// Final complete normalized update records; unused tails remain unchanged.
    pub client_updates: &'output mut [ClientDataUpdateSlot],
    /// Final normalized command text, excluding terminators; tails remain unchanged.
    pub command_bytes: &'output mut [u8],
}
