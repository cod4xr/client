// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed root-catalog failures, without string data, encoded links or names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zone5Error {
    /// Required root, pointer table, string or directory bytes are missing.
    Truncated,
    /// A signed root count is negative or unrepresentable.
    Count,
    /// Root pointer/count relation is inconsistent or not an inline/null profile.
    RootLink,
    /// Script-string offset/insert links require an unimplemented alias layer.
    StringLink,
    /// Script-string count exceeds host policy.
    StringCountBudget,
    /// Asset count exceeds host policy.
    AssetCountBudget,
    /// Inline string bytes exceed host policy.
    StringByteBudget,
    /// Physical prefix exceeds host policy or arithmetic capacity.
    PrefixBudget,
    /// Logical virtual-block accounting exceeds its checked declaration.
    VirtualExtent,
    /// Caller private metadata scratch is insufficient.
    ScratchCapacity,
    /// Caller final slots or cooked bytes are insufficient.
    OutputCapacity,
    /// Asset type is outside the explicit PC zone namespace.
    AssetKind,
}
impl fmt::Display for Zone5Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Truncated => "zone root catalog prefix incomplete",
            Self::Count => "zone root count invalid",
            Self::RootLink => "zone root link/count profile unsupported",
            Self::StringLink => "zone script-string link requires alias handling",
            Self::StringCountBudget => "zone script-string count budget exhausted",
            Self::AssetCountBudget => "zone asset count budget exhausted",
            Self::StringByteBudget => "zone inline string byte budget exhausted",
            Self::PrefixBudget => "zone serialized prefix byte budget exhausted",
            Self::VirtualExtent => "zone logical virtual prefix exceeds declaration",
            Self::ScratchCapacity => "zone private metadata capacity insufficient",
            Self::OutputCapacity => "zone final catalog capacity insufficient",
            Self::AssetKind => "zone asset type outside reviewed PC namespace",
        })
    }
}
impl Error for Zone5Error {}
