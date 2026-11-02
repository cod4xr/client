// SPDX-License-Identifier: AGPL-3.0-or-later
use super::LegacyMapCenter;
use crate::BaselineProtocol;

/// Explicit native coordinate dialect and its required checked geometry context.
/// This selects a wire reader; it proves neither peer negotiation nor session binding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EntityDecodeContext {
    /// Absolute binary32 origins in protocol 21.
    Cod4x21,
    /// Native short/full origin deltas with an explicit protocol-6 map center.
    Legacy6(LegacyMapCenter),
}
impl EntityDecodeContext {
    /// Returns the native protocol selected explicitly by this context.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        match self {
            Self::Cod4x21 => BaselineProtocol::Cod4x21,
            Self::Legacy6(_) => BaselineProtocol::Legacy6,
        }
    }
    pub(super) fn legacy_center(self) -> Option<LegacyMapCenter> {
        match self {
            Self::Cod4x21 => None,
            Self::Legacy6(center) => Some(center),
        }
    }
    pub(super) fn replace_legacy(&mut self, center: LegacyMapCenter) -> bool {
        if matches!(self, Self::Legacy6(_)) {
            *self = Self::Legacy6(center);
            true
        } else {
            false
        }
    }
}
