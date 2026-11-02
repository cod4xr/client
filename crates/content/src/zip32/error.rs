// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed catalog failures without names, archive comments or payload bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Zip32Error {
    /// Complete archive bytes exceed the host inspection budget.
    ArchiveBudget,
    /// Central-directory bytes exceed the host inspection budget.
    DirectoryBudget,
    /// Entry count exceeds the host inspection budget.
    EntryBudget,
    /// Per-entry or combined name bytes exceed the host budget.
    NameBudget,
    /// Per-entry or combined declared output exceeds the host budget.
    UncompressedBudget,
    /// Caller private descriptor scratch is insufficient.
    ScratchCapacity,
    /// Caller final descriptor storage is insufficient.
    OutputCapacity,
    /// A required byte span or arithmetic extent lies outside the archive.
    Bounds,
    /// No exact terminal classic end record was found.
    MissingEnd,
    /// More than one exact terminal end record candidate exists.
    AmbiguousEnd,
    /// Multiple disks, ZIP64, encryption or an unsupported feature was requested.
    UnsupportedFeature,
    /// An expected record signature, count or enclosing extent is invalid.
    Structure,
    /// A name is empty, contains NUL, or violates an explicit UTF-8 flag.
    Name,
    /// An extra field has incomplete TLV framing.
    Extra,
    /// Local and central entry metadata disagree.
    LocalMismatch,
    /// A required data descriptor is absent, ambiguous or disagrees with metadata.
    Descriptor,
    /// Two local-header/payload/descriptor ranges overlap or alias.
    Overlap,
}

impl fmt::Display for Zip32Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ArchiveBudget => "ZIP archive byte budget exhausted",
            Self::DirectoryBudget => "ZIP directory byte budget exhausted",
            Self::EntryBudget => "ZIP entry budget exhausted",
            Self::NameBudget => "ZIP name byte budget exhausted",
            Self::UncompressedBudget => "ZIP declared output budget exhausted",
            Self::ScratchCapacity => "ZIP private scratch capacity insufficient",
            Self::OutputCapacity => "ZIP final descriptor capacity insufficient",
            Self::Bounds => "ZIP record span outside archive",
            Self::MissingEnd => "ZIP exact terminal end record missing",
            Self::AmbiguousEnd => "ZIP terminal end record ambiguous",
            Self::UnsupportedFeature => "ZIP feature outside reviewed classic profile",
            Self::Structure => "ZIP record structure invalid",
            Self::Name => "ZIP entry name encoding invalid",
            Self::Extra => "ZIP extra-field framing invalid",
            Self::LocalMismatch => "ZIP local and central metadata disagree",
            Self::Descriptor => "ZIP data descriptor invalid",
            Self::Overlap => "ZIP local entry ranges overlap",
        })
    }
}
impl Error for Zip32Error {}
