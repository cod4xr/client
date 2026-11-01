// SPDX-License-Identifier: AGPL-3.0-or-later
/// Explicit reviewed baseline message profiles, selected from original connect settings.
/// Discovery advertisements and challenge suffixes never negotiate this choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BaselineProtocol {
    /// Original protocol-6 message layout; legacy handshake support remains separate.
    Legacy6,
    /// Pinned public `CoD4X` 21 message layout.
    Cod4x21,
}

impl BaselineProtocol {
    /// Returns the original explicitly requested wire-protocol number.
    #[must_use]
    pub const fn number(self) -> i32 {
        match self {
            Self::Legacy6 => 6,
            Self::Cod4x21 => 21,
        }
    }
    /// Maps original connect settings to a reviewed profile, without fallback.
    /// Never call this on discovery advertisements or a challenge compatibility hint.
    #[must_use]
    pub const fn from_number(protocol: i32) -> Option<Self> {
        match protocol {
            6 => Some(Self::Legacy6),
            21 => Some(Self::Cod4x21),
            _ => None,
        }
    }
}
