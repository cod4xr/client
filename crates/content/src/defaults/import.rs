// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ConfigDefault, DefaultSlot, DefaultsBudget, DefaultsError,
    profile::{self, Profile},
    walk::{self, Noop, Sink},
};
use crate::pe32::{Pe32Budget, Pe32Error, Pe32Image};
use sha2::{Digest, Sha256};
use std::fmt;

/// Complete imported raw defaults borrowing cooked storage only, not the source image.
/// Values still require engine interpretation and explicit merging with wire patches.
pub struct ImportedDefaults<'storage> {
    bytes: &'storage [u8],
    slots: &'storage [DefaultSlot],
    digest: [u8; 32],
}
impl<'storage> ImportedDefaults<'storage> {
    /// Imports the one exact audited installed multiplayer image profile.
    /// Hash identity is a data-layout binding, not proof of publisher authenticity.
    /// The full table validates before any output writes; unused tails stay unchanged.
    ///
    /// # Errors
    /// Rejects unknown images, budgets, malformed PE/table geometry, unterminated values
    /// and insufficient caller storage. Final text/metadata remain unchanged on failure.
    pub fn import(
        image: &[u8],
        budget: DefaultsBudget,
        output: &'storage mut [u8],
        slots: &'storage mut [DefaultSlot],
    ) -> Result<Self, DefaultsError> {
        Self::with_profile(image, budget, output, slots, profile::MULTIPLAYER)
    }

    pub(super) fn with_profile(
        input: &[u8],
        budget: DefaultsBudget,
        output: &'storage mut [u8],
        slots: &'storage mut [DefaultSlot],
        profile: Profile,
    ) -> Result<Self, DefaultsError> {
        if input.len() > budget.max_image_bytes {
            return Err(Pe32Error::ImageBudget.into());
        }
        if input.len() != profile.image_bytes || Sha256::digest(input)[..] != profile.digest[..] {
            return Err(DefaultsError::UnknownImage);
        }
        if profile.records > budget.max_records {
            return Err(DefaultsError::RecordBudget);
        }
        if profile.records > slots.len() {
            return Err(DefaultsError::RecordCapacity);
        }
        if profile.text_bytes > budget.max_text_bytes {
            return Err(DefaultsError::TextBudget);
        }
        if profile.text_bytes > output.len() {
            return Err(DefaultsError::OutputCapacity);
        }
        let image = Pe32Image::parse(
            input,
            Pe32Budget {
                max_image_bytes: budget.max_image_bytes,
                max_sections: budget.max_sections,
            },
        )?;
        walk::walk(image, profile, budget, &mut Noop)?;
        // Exact immutable image/profile/budget and every extent were preflighted.
        walk::walk(
            image,
            profile,
            budget,
            &mut Writer {
                output,
                slots,
                records: 0,
                bytes: 0,
            },
        )?;
        Ok(Self {
            bytes: &output[..profile.text_bytes],
            slots: &slots[..profile.records],
            digest: profile.digest,
        })
    }
    /// Returns complete reviewed defaults in increasing baseline identifier order.
    #[must_use]
    pub fn records(&self) -> impl ExactSizeIterator<Item = ConfigDefault<'_>> {
        self.slots.iter().map(|slot| ConfigDefault {
            index: slot.index,
            bytes: &self.bytes[slot.offset..slot.offset + slot.length],
        })
    }
    /// Returns complete imported records, excluding the native sentinel.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.slots.len()
    }
    /// Returns combined raw value bytes, excluding terminators.
    #[must_use]
    pub fn text_bytes(&self) -> usize {
        self.bytes.len()
    }
    /// Returns the last reviewed default identifier, or none for an empty catalog.
    #[must_use]
    pub fn maximum_index(&self) -> Option<u32> {
        self.slots.last().map(|slot| slot.index)
    }
    /// Returns the exact matched image digest, without retaining its bytes or path.
    #[must_use]
    pub fn image_sha256(&self) -> [u8; 32] {
        self.digest
    }
}

struct Writer<'writer> {
    output: &'writer mut [u8],
    slots: &'writer mut [DefaultSlot],
    records: usize,
    bytes: usize,
}
impl Sink for Writer<'_> {
    fn record(&mut self, index: u32, bytes: &[u8]) {
        let end = self.bytes + bytes.len();
        self.output[self.bytes..end].copy_from_slice(bytes);
        self.slots[self.records] = DefaultSlot {
            index,
            offset: self.bytes,
            length: bytes.len(),
        };
        self.records += 1;
        self.bytes = end;
    }
}
impl fmt::Debug for ImportedDefaults<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ImportedDefaults")
            .field("records", &self.record_count())
            .field("text_bytes", &self.text_bytes())
            .finish_non_exhaustive()
    }
}
