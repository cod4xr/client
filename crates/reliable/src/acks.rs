// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{MAX_SELECTIVE_ACK_RANGES, ReliableError};
use std::fmt;

/// One range relative to the cumulative acknowledgement, not an absolute authority.
/// Stateful admission must check the entire range against actually transmitted data.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SelectiveAckRange {
    /// Nonnegative native signed-short offset from the cumulative base.
    pub offset: u16,
    /// Positive native signed-short count of consecutive acknowledged fragments.
    pub length: u16,
}

/// Owned bounded selective metadata. Construction validates representability only.
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct SelectiveAcks {
    ranges: [SelectiveAckRange; MAX_SELECTIVE_ACK_RANGES],
    length: u8,
}

impl SelectiveAcks {
    /// Copies up to three relative ranges without allocating or applying acknowledgements.
    /// Overlap/order is preserved; the stateful window owns temporal validity.
    ///
    /// # Errors
    /// Rejects range count, zero lengths and fields above positive signed-short bounds.
    pub fn new(ranges: &[SelectiveAckRange]) -> Result<Self, ReliableError> {
        if ranges.len() > MAX_SELECTIVE_ACK_RANGES {
            return Err(ReliableError::RangeCount);
        }
        for range in ranges {
            if range.length == 0
                || i16::try_from(range.offset).is_err()
                || i16::try_from(range.length).is_err()
            {
                return Err(ReliableError::Range);
            }
        }
        let mut result = Self::default();
        result.ranges[..ranges.len()].copy_from_slice(ranges);
        result.length = u8::try_from(ranges.len()).map_err(|_| ReliableError::RangeCount)?;
        Ok(result)
    }
    /// Returns only initialized selective metadata.
    #[must_use]
    pub fn ranges(&self) -> &[SelectiveAckRange] {
        &self.ranges[..usize::from(self.length)]
    }
    /// Returns the bounded number of ranges.
    #[must_use]
    pub fn len(&self) -> usize {
        usize::from(self.length)
    }
    /// Returns whether no selective ranges are present.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}

impl fmt::Debug for SelectiveAcks {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.debug_list().entries(self.ranges()).finish()
    }
}
