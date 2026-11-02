// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{CheckedClientAcknowledgement, ClientCommandError, LegacyClientCommands};

impl LegacyClientCommands<'_> {
    /// Records the queued prefix admitted to actual outgoing message I/O by the host.
    /// Queueing or merely checking a peer acknowledgement does not transmit commands.
    ///
    /// # Errors
    /// Rejects backwards transmit movement or a prefix above the latest queued sequence.
    pub fn mark_transmitted(&mut self, through: u32) -> Result<(), ClientCommandError> {
        if through < self.transmitted || through > self.latest {
            return Err(ClientCommandError::TransmitRange);
        }
        self.transmitted = through;
        Ok(())
    }

    /// Checks a peer number and borrows its retained canonical key without mutation.
    /// Decode/validate the entire message before committing this acknowledgement.
    ///
    /// # Errors
    /// Rejects regression, never-transmitted numbers and unavailable slot references.
    pub fn check_acknowledgement(
        &self,
        sequence: u32,
    ) -> Result<CheckedClientAcknowledgement<'_>, ClientCommandError> {
        if sequence < self.acknowledged {
            return Err(ClientCommandError::AcknowledgementRegression);
        }
        if sequence > self.transmitted {
            return Err(ClientCommandError::UnsentAcknowledgement);
        }
        Ok(CheckedClientAcknowledgement {
            command: self.command(sequence)?,
        })
    }

    /// Commits an acknowledgement only after successful complete-message validation.
    /// The retained key is copied before its pending slot becomes reusable. Repeated
    /// acknowledgement of the same command is valid and does not copy or release more.
    ///
    /// # Errors
    /// Revalidates regression/transmit/retention constraints without mutation on failure.
    pub fn commit_acknowledgement(&mut self, sequence: u32) -> Result<(), ClientCommandError> {
        self.prepare_acknowledgement(sequence)?.commit();
        Ok(())
    }
}
