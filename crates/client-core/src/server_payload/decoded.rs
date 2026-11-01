// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BaselineProtocol, commands::CheckedClientAcknowledgement};
use std::fmt;

/// Decoded untrusted bytes, not a validated service stream or playable gamestate.
/// The history borrow prevents key-slot release while this payload is being inspected.
pub struct DecodedServerPayload<'decoded, 'history> {
    pub(super) profile: BaselineProtocol,
    pub(super) sequence: u32,
    pub(super) acknowledgement: CheckedClientAcknowledgement<'history>,
    pub(super) body: &'decoded [u8],
}

impl DecodedServerPayload<'_, '_> {
    /// Returns the original connection's reviewed profile, not a new negotiation.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the validated complete channel sequence supplied by the host.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.sequence
    }
    /// Returns the checked number; commit it only after complete service validation.
    #[must_use]
    pub fn client_command_acknowledge(&self) -> u32 {
        self.acknowledgement.sequence()
    }
    /// Borrows decompressed, untrusted service bytes, including Huffman padding symbols.
    /// A higher layer must validate opcodes/EOF; never execute or log these bytes.
    #[must_use]
    pub fn body(&self) -> &[u8] {
        self.body
    }
}

impl fmt::Debug for DecodedServerPayload<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DecodedServerPayload")
            .field("profile", &self.profile)
            .field("sequence", &self.sequence)
            .field("acknowledgement", &self.client_command_acknowledge())
            .field("bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}
