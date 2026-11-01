// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

/// A 128-bit random query correlation token encoded as 32 lowercase ASCII hex bytes.
/// This proves response correlation, not server identity or client authentication.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct QueryChallenge([u8; 32]);

impl QueryChallenge {
    /// Encodes caller-supplied entropy without allocation. The runtime supplies OS randomness.
    #[must_use]
    pub fn from_entropy(entropy: [u8; 16]) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut encoded = [0; 32];
        for (index, byte) in entropy.into_iter().enumerate() {
            encoded[index * 2] = HEX[usize::from(byte >> 4)];
            encoded[index * 2 + 1] = HEX[usize::from(byte & 15)];
        }
        Self(encoded)
    }

    pub(crate) fn bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for QueryChallenge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("QueryChallenge(..)")
    }
}
