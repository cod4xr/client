// SPDX-License-Identifier: AGPL-3.0-or-later
use super::BodyReadError;
use cod4x_protocol::MessageError;
use std::fmt;

/// Allocation-free complete-body adapter using shared primitive readers over bounded views.
/// Cached legacy bits survive intervening byte operations and storage rebasing.
#[derive(Clone)]
pub struct BodyReader<'body> {
    pub(super) input: &'body [u8],
    pub(super) position: usize,
    pub(super) cached_byte: u8,
    pub(super) cached_bits: u8,
}

impl<'body> BodyReader<'body> {
    /// Borrows a complete body within an explicit host-selected byte budget.
    /// This neither allocates from a peer length nor changes any wire-profile limits.
    ///
    /// # Errors
    /// Rejects input beyond the supplied budget before creating a view.
    pub fn try_new(input: &'body [u8], maximum_bytes: usize) -> Result<Self, BodyReadError> {
        if input.len() > maximum_bytes {
            return Err(MessageError::MessageTooLarge {
                length: input.len(),
                maximum: maximum_bytes,
            }
            .into());
        }
        Ok(Self {
            input,
            position: 0,
            cached_byte: 0,
            cached_bits: 0,
        })
    }
    /// Returns consumed whole bytes; a partial bit byte is already included.
    #[must_use]
    pub fn byte_position(&self) -> usize {
        self.position
    }
    /// Returns unconsumed whole bytes. Cached bits may remain when this is zero.
    #[must_use]
    pub fn remaining_bytes(&self) -> usize {
        self.input.len() - self.position
    }
    /// Returns unread bits in an already consumed partial byte, never its private value.
    #[must_use]
    pub fn remaining_cached_bits(&self) -> u8 {
        self.cached_bits
    }
}

impl fmt::Debug for BodyReader<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("BodyReader")
            .field("input_bytes", &self.input.len())
            .field("consumed_bytes", &self.position)
            .field("cached_bits", &self.cached_bits)
            .finish_non_exhaustive()
    }
}
