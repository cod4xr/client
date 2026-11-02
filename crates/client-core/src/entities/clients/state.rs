// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{CLIENT_STATE_WORDS, ClientStateWord};
use std::fmt;

/// Owned snapshot client-state words without native pointers or engine side effects.
/// Snapshot names are sixteen raw bytes; they are distinct from identity messages.
#[derive(Clone, Eq, PartialEq)]
pub struct ClientState {
    pub(super) number: u32,
    pub(super) words: [u32; CLIENT_STATE_WORDS],
}
impl ClientState {
    /// Returns the validated six-bit native client index.
    #[must_use]
    pub fn number(&self) -> u32 {
        self.number
    }
    /// Returns an inert semantic word requiring later resource/gameplay validation.
    #[must_use]
    pub fn word(&self, word: ClientStateWord) -> u32 {
        self.words[word as usize]
    }
    /// Returns every raw name byte, including NULs, percent signs and non-UTF-8 bytes.
    /// No native string reader, truncation or percent replacement applies here.
    #[must_use]
    pub fn name_bytes(&self) -> [u8; 16] {
        let mut bytes = [0; 16];
        for (chunk, word) in bytes.chunks_exact_mut(4).zip([
            ClientStateWord::Name0,
            ClientStateWord::Name1,
            ClientStateWord::Name2,
            ClientStateWord::Name3,
        ]) {
            chunk.copy_from_slice(&self.word(word).to_le_bytes());
        }
        bytes
    }
}
impl fmt::Debug for ClientState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClientState")
            .field("number", &self.number)
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}

/// One complete inert client-state update or removal; no snapshot is acknowledged.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ClientStateDelta {
    /// Owned decoded words; all omitted fields retain their explicit prior values.
    State(ClientState),
    /// The wire client was removed from this snapshot's client-state set.
    Removed {
        /// Validated six-bit native client index.
        number: u32,
    },
}
