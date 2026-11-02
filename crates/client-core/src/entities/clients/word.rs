// SPDX-License-Identifier: AGPL-3.0-or-later
/// Semantic client-state words, independent of C memory layout and wire field order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ClientStateWord {
    /// Team identifier; not engine authority.
    Team,
    /// Model resource index.
    Model,
    /// Attached model index zero.
    AttachmentModel0,
    /// Attached model index one.
    AttachmentModel1,
    /// Attached model index two.
    AttachmentModel2,
    /// Attached model index three.
    AttachmentModel3,
    /// Attached model index four.
    AttachmentModel4,
    /// Attached model index five.
    AttachmentModel5,
    /// Attachment tag index zero.
    AttachmentTag0,
    /// Attachment tag index one.
    AttachmentTag1,
    /// Attachment tag index two.
    AttachmentTag2,
    /// Attachment tag index three.
    AttachmentTag3,
    /// Attachment tag index four.
    AttachmentTag4,
    /// Attachment tag index five.
    AttachmentTag5,
    /// First four name bytes, packed little endian without text normalization.
    Name0,
    /// Name bytes four through seven.
    Name1,
    /// Name bytes eight through eleven.
    Name2,
    /// Name bytes twelve through fifteen.
    Name3,
    /// Maximum sprint-time multiplier, as raw binary32 bits.
    SprintTimeMultiplier,
    /// Rank identifier.
    Rank,
    /// Prestige identifier.
    Prestige,
    /// Perk bits.
    Perks,
    /// Attached vehicle entity index.
    VehicleEntity,
    /// Attached vehicle seat index.
    VehicleSeat,
}
