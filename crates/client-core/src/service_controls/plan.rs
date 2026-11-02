// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{
    ClientDataUpdateSlot, ServerCommand, ServiceControlBudget, ServiceControlError,
    client_scan::Clients,
    publish,
    scan::{self, Layout},
};
use crate::client_data::ClientDataUpdate;
use crate::{BaselineProtocol, server_payload::DecodedServerPayload};

/// Complete normalized baseline control records, before host deduplication/application.
/// Only successful whole-stream validation exposes this plan; no command is executed.
pub struct LegacyServicePlan<'commands> {
    profile: BaselineProtocol,
    message_sequence: u32,
    client_acknowledge: u32,
    layout: Layout,
    bytes: &'commands [u8],
    client_updates: &'commands [ClientDataUpdateSlot],
}

impl<'commands> LegacyServicePlan<'commands> {
    /// Validates all supported records through EOF before populating command output.
    /// Scratch is private string-reading workspace; its contents are unspecified on
    /// failure. Final output and session/history state remain unchanged on any failure.
    /// Logical EOF excludes native Huffman padding from further service inspection.
    ///
    /// # Errors
    /// Rejects invalid/exhausted budgets, unsupported/truncated records, missing EOF,
    /// invalid command sequences, string/profile overflow and insufficient caller storage.
    pub fn parse(
        payload: &DecodedServerPayload<'_, '_>,
        budget: ServiceControlBudget,
        scratch: &mut [u8],
        output: &'commands mut [u8],
    ) -> Result<Self, ServiceControlError> {
        let body = payload.body();
        let layout = scan::scan(body, budget, scratch, output.len(), &mut Clients::Reject)?;
        publish::commands(body, &layout, output)?;
        Ok(Self::from_layout(payload, layout, output, &[]))
    }

    pub(super) fn from_layout(
        payload: &DecodedServerPayload<'_, '_>,
        layout: Layout,
        output: &'commands [u8],
        client_updates: &'commands [ClientDataUpdateSlot],
    ) -> Self {
        Self::from_parts(
            payload.profile(),
            payload.sequence(),
            payload.client_command_acknowledge(),
            layout,
            output,
            client_updates,
        )
    }

    pub(crate) fn from_parts(
        profile: BaselineProtocol,
        message_sequence: u32,
        client_acknowledge: u32,
        layout: Layout,
        output: &'commands [u8],
        client_updates: &'commands [ClientDataUpdateSlot],
    ) -> Self {
        let bytes = &output[..layout.byte_count];
        Self {
            profile,
            message_sequence,
            client_acknowledge,
            layout,
            bytes,
            client_updates,
        }
    }

    /// Returns the reviewed original message profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the complete validated control message's channel sequence.
    #[must_use]
    pub fn message_sequence(&self) -> u32 {
        self.message_sequence
    }
    /// Returns the checked client acknowledgement. Apply it to the same connection
    /// only after preflighting host resources for this validated plan.
    #[must_use]
    pub fn client_command_acknowledge(&self) -> u32 {
        self.client_acknowledge
    }
    /// Returns all inspected records, including NOPs and EOF.
    #[must_use]
    pub fn record_count(&self) -> u32 {
        self.layout.record_count
    }
    /// Returns logical service bytes through EOF, excluding compression padding.
    #[must_use]
    pub fn logical_bytes(&self) -> usize {
        self.layout.logical_bytes
    }
    /// Returns complete command records retained in order, including duplicates.
    #[must_use]
    pub fn command_count(&self) -> usize {
        self.layout.command_count
    }
    /// Returns the total normalized command bytes, without terminator overhead.
    #[must_use]
    pub fn command_bytes(&self) -> usize {
        self.layout.byte_count
    }
    /// Returns complete identity updates from the opt-in mixed-control parser.
    /// The legacy commands-only entry point always exposes zero updates.
    #[must_use]
    pub fn client_update_count(&self) -> usize {
        self.client_updates.len()
    }
    /// Borrows owned normalized updates only after the whole service stream validated.
    /// Input payload, private scratch and command history may already be released.
    pub fn client_updates(&self) -> impl Iterator<Item = &ClientDataUpdate> {
        self.client_updates
            .iter()
            .filter_map(|slot| slot.0.as_ref())
    }
    /// Borrows one initialized command; out-of-range indices expose no backing bytes.
    #[must_use]
    pub fn command(&self, index: usize) -> Option<ServerCommand<'_>> {
        if index >= self.layout.command_count {
            return None;
        }
        let descriptor = self.layout.commands[index];
        Some(ServerCommand {
            sequence: descriptor.sequence,
            bytes: &self.bytes[descriptor.destination..descriptor.destination + descriptor.length],
        })
    }
    /// Iterates complete normalized records without executing or deduplicating them.
    #[must_use]
    pub fn commands(&self) -> impl ExactSizeIterator<Item = ServerCommand<'_>> {
        self.layout.commands[..self.layout.command_count]
            .iter()
            .map(|descriptor| ServerCommand {
                sequence: descriptor.sequence,
                bytes: &self.bytes
                    [descriptor.destination..descriptor.destination + descriptor.length],
            })
    }
}

impl fmt::Debug for LegacyServicePlan<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LegacyServicePlan")
            .field("profile", &self.profile)
            .field("message_sequence", &self.message_sequence)
            .field("client_acknowledgement", &self.client_acknowledge)
            .field("records", &self.layout.record_count)
            .field("commands", &self.layout.command_count)
            .field("client_updates", &self.client_updates.len())
            .field("bytes", &self.layout.byte_count)
            .finish_non_exhaustive()
    }
}
