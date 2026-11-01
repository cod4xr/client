// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// One retained, canonicalized private command, excluding its terminator.
#[derive(Clone, Copy)]
pub struct ClientCommand<'history> {
    pub(super) sequence: u32,
    pub(super) bytes: &'history [u8],
}

impl<'history> ClientCommand<'history> {
    /// Returns the command sequence; zero identifies the initial empty key.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Borrows canonicalized command/key bytes. Never include them in operational logs.
    #[must_use]
    pub fn as_bytes(&self) -> &'history [u8] {
        self.bytes
    }
}

impl fmt::Debug for ClientCommand<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientCommand")
            .field("sequence", &self.sequence)
            .field("bytes", &self.bytes.len())
            .finish_non_exhaustive()
    }
}

/// An acknowledgement checked against transmitted and retained command history.
/// Checking does not release slots; commit only after complete message validation.
#[derive(Clone, Copy)]
pub struct CheckedClientAcknowledgement<'history> {
    pub(super) command: ClientCommand<'history>,
}

impl<'history> CheckedClientAcknowledgement<'history> {
    /// Returns the checked sequence without advancing history state.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.command.sequence()
    }
    /// Borrows the exact retained key required for server-message obfuscation.
    /// It is command data, not authentication material; never log it.
    #[must_use]
    pub fn command(&self) -> ClientCommand<'history> {
        self.command
    }
}

impl fmt::Debug for CheckedClientAcknowledgement<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CheckedClientAcknowledgement")
            .field("command", &self.command)
            .finish_non_exhaustive()
    }
}
