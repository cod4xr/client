// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientCommandError, LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands};
use std::fmt;

/// Exclusive retained-key publication lease. Dropping it leaves the ACK unchanged.
/// The checked original owner cannot change while the lease is used:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::commands::LegacyClientCommands;
/// fn original_owner(history: &mut LegacyClientCommands<'_>) {
///     let pending = history.prepare_acknowledgement(0).unwrap();
///     history.mark_transmitted(0).unwrap();
///     pending.commit();
/// }
/// ```
#[must_use = "Dropping the acknowledgement preparation cancels key release."]
pub struct PreparedClientAcknowledgement<'owner, 'storage> {
    owner: &'owner mut LegacyClientCommands<'storage>,
    sequence: u32,
    source: Option<(usize, usize)>,
}
impl<'storage> LegacyClientCommands<'storage> {
    /// Checks the transmitted/retained prefix and reserves exclusive original-owner
    /// access through later host resource preflight. No bytes or counters change.
    ///
    /// # Errors
    /// Rejects ACK regression, unsent prefixes and unavailable retained slots.
    pub fn prepare_acknowledgement(
        &mut self,
        sequence: u32,
    ) -> Result<PreparedClientAcknowledgement<'_, 'storage>, ClientCommandError> {
        self.check_acknowledgement(sequence)?;
        let source = if sequence == self.acknowledged {
            None
        } else {
            Some(self.position(sequence)?)
        };
        Ok(PreparedClientAcknowledgement {
            owner: self,
            sequence,
            source,
        })
    }
}
impl PreparedClientAcknowledgement<'_, '_> {
    /// Returns the checked prefix without exposing canonical key bytes.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    pub(crate) fn history(&self) -> &LegacyClientCommands<'_> {
        self.owner
    }
    /// Copies the prechecked canonical key before its pending slot becomes reusable.
    /// No allocation, external callback or fallible operation occurs. Exclusive owner
    /// access preserves all checked positions/lengths until this consuming commit.
    pub fn commit(self) {
        if let Some((index, length)) = self.source {
            let start = index * LEGACY_COMMAND_SLOT_BYTES;
            self.owner.key[..length].copy_from_slice(&self.owner.pending[start..start + length]);
            self.owner.key[length] = 0;
            self.owner.key_length = length;
            self.owner.acknowledged = self.sequence;
        }
    }
}
impl fmt::Debug for PreparedClientAcknowledgement<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedClientAcknowledgement")
            .field("sequence", &self.sequence)
            .field("advances_prefix", &self.source.is_some())
            .finish_non_exhaustive()
    }
}
