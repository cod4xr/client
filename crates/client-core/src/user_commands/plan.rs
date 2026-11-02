// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MAX_ABSOLUTE_COMMAND_BYTES, UserCommandEncodeError, UserCommandInput, write};
use cod4x_protocol::MessageWriter;
use std::fmt;

/// Checked native full-form command, with absolute time and all common fields sent.
/// Conditional fields are transmitted only when enabled by their button bits;
/// disabled fields retain the receiver's prior inert values, as in the native codec.
/// The compatibility key is not authentication or authorization.
#[derive(Clone, Copy)]
pub struct AbsoluteUserCommandPlan {
    pub(super) key: u32,
    pub(super) input: UserCommandInput,
}
impl AbsoluteUserCommandPlan {
    /// Validates every native width and conditional field before any output write.
    /// No snapshot-derived default command, implicit float quantization, engine
    /// action, sequence admission, allocation or I/O occurs here.
    ///
    /// # Errors
    /// Rejects oversized buttons/weapon/offhand and mismatched action presence.
    pub fn new(key: u32, input: UserCommandInput) -> Result<Self, UserCommandEncodeError> {
        if input.buttons >= 0x0020_0000 {
            return Err(UserCommandEncodeError::Buttons);
        }
        if input.weapon >= 128 {
            return Err(UserCommandEncodeError::Weapon);
        }
        if input.offhand >= 128 {
            return Err(UserCommandEncodeError::Offhand);
        }
        if input.selected_location.is_some() != (input.buttons & 0x10000 != 0) {
            return Err(UserCommandEncodeError::SelectedLocation);
        }
        if input.melee.is_some() != (input.buttons & 4 != 0) {
            return Err(UserCommandEncodeError::Melee);
        }
        Ok(Self { key, input })
    }
    /// Returns the exact byte length at a fresh bit cursor, excluding packet framing.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        let selection = usize::from(self.input.selected_location.is_some());
        let melee = usize::from(self.input.melee.is_some());
        let bit_count = 49 + 2 * selection + 10 * melee;
        10 + 2 * selection + 2 * melee + bit_count.div_ceil(8)
    }
    /// Publishes a complete standalone command at a fresh bit cursor atomically.
    /// Existing output tails stay unchanged. Encoded command byte slices cannot be
    /// concatenated to form a batch: native bit bytes span interleaved byte fields.
    /// Packet assembly must use the same private writer across the complete batch.
    ///
    /// # Errors
    /// Insufficient final capacity or a primitive failure preserves all final bytes.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, UserCommandEncodeError> {
        let mut private = [0; MAX_ABSOLUTE_COMMAND_BYTES];
        let mut writer =
            MessageWriter::try_new(&mut private).map_err(UserCommandEncodeError::Message)?;
        self.write(&mut writer)
            .map_err(UserCommandEncodeError::Message)?;
        let length = writer.len();
        if output.len() < length {
            return Err(UserCommandEncodeError::OutputCapacity);
        }
        output[..length].copy_from_slice(writer.as_bytes());
        Ok(length)
    }
    pub(crate) fn write(
        &self,
        writer: &mut MessageWriter<'_>,
    ) -> Result<(), cod4x_protocol::MessageError> {
        write::write(writer, self.key, self.input)
    }
}
impl fmt::Debug for AbsoluteUserCommandPlan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AbsoluteUserCommandPlan")
            .field("bytes", &self.encoded_len())
            .finish_non_exhaustive()
    }
}
