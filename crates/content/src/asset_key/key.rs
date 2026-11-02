// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{AssetKeyBudget, AssetKeyError};
use std::fmt;

/// Canonical strict relative virtual file key over caller-owned cooked bytes.
/// ASCII case and slash direction normalize; other byte encodings remain intact.
/// Equality/order/hash are suitable for virtual lookup, without OS path authority.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AssetKey<'storage> {
    bytes: &'storage [u8],
    components: usize,
}
impl<'storage> AssetKey<'storage> {
    /// Validates a strict relative file name and publishes its complete canonical key.
    /// Rejects roots, directory markers, empty/dot components, native-forbidden `..`
    /// substrings, colons and ASCII controls. Converts backslashes to forward slashes
    /// and ASCII letters to lowercase; high bytes and percent signs remain verbatim.
    /// This does not implement native loose request sanitation, Unicode case folding,
    /// Windows device-name policy, extraction or physical filesystem resolution.
    ///
    /// # Errors
    /// Rejects invalid names, host budgets and short output before any final writes.
    /// No fixed native path-size limit, terminator or hidden allocation is imposed.
    pub fn parse(
        input: &[u8],
        budget: AssetKeyBudget,
        output: &'storage mut [u8],
    ) -> Result<Self, AssetKeyError> {
        if input.is_empty() {
            return Err(AssetKeyError::Empty);
        }
        if input.len() > budget.max_bytes {
            return Err(AssetKeyError::ByteBudget);
        }
        if output.len() < input.len() {
            return Err(AssetKeyError::OutputCapacity);
        }
        if separator(input[0]) {
            return Err(AssetKeyError::Rooted);
        }
        if input.last().copied().is_some_and(separator) {
            return Err(AssetKeyError::Directory);
        }
        let mut components = 0_usize;
        for component in input.split(|&byte| separator(byte)) {
            components = components
                .checked_add(1)
                .filter(|&count| count <= budget.max_components)
                .ok_or(AssetKeyError::ComponentBudget)?;
            if component.is_empty() {
                return Err(AssetKeyError::EmptyComponent);
            }
            if component == b"." || component.windows(2).any(|pair| pair == b"..") {
                return Err(AssetKeyError::DotComponent);
            }
            if component
                .iter()
                .any(|byte| matches!(byte, 0..=31 | 127 | b':'))
            {
                return Err(AssetKeyError::Character);
            }
        }
        for (&byte, target) in input.iter().zip(&mut output[..input.len()]) {
            *target = if separator(byte) {
                b'/'
            } else {
                byte.to_ascii_lowercase()
            };
        }
        Ok(Self {
            bytes: &output[..input.len()],
            components,
        })
    }
    /// Returns complete canonical virtual bytes, excluding unused caller storage.
    /// Do not pass them directly to a host filesystem without separate resolution policy.
    #[must_use]
    pub fn bytes(&self) -> &'storage [u8] {
        self.bytes
    }
    /// Returns the validated number of nonempty relative file components.
    #[must_use]
    pub fn component_count(&self) -> usize {
        self.components
    }
}
fn separator(byte: u8) -> bool {
    matches!(byte, b'/' | b'\\')
}

impl fmt::Debug for AssetKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AssetKey")
            .field("bytes", &self.bytes.len())
            .field("components", &self.components)
            .finish_non_exhaustive()
    }
}
