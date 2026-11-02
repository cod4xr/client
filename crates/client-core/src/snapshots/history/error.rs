// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed retention/reference failures without private player, entity or client data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotHistoryError {
    /// A history needs at least one caller-provided snapshot slot.
    Storage,
    /// The message belongs to another host-managed world epoch.
    Epoch,
    /// The decoded profile differs from this owner's explicit profile.
    Profile,
    /// This message is no newer than the last committed snapshot.
    MessageOrder,
    /// The next selected retention slot cannot hold all merged entities.
    EntityCapacity,
    /// The next selected retention slot cannot hold all merged clients.
    ClientCapacity,
    /// Delta arithmetic does not select a positive prior ordinary message.
    ReferenceSequence,
    /// The exact selected message is absent or has been evicted.
    MissingReference,
    /// Reset requires a strictly newer host-managed world epoch.
    EpochOrder,
}
impl fmt::Display for SnapshotHistoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Storage => "snapshot history requires storage",
            Self::Epoch => "snapshot history epoch mismatch",
            Self::Profile => "snapshot history profile mismatch",
            Self::MessageOrder => "snapshot history message is stale or already committed",
            Self::EntityCapacity => "snapshot history entity capacity exhausted",
            Self::ClientCapacity => "snapshot history client capacity exhausted",
            Self::ReferenceSequence => "invalid snapshot history reference sequence",
            Self::MissingReference => "exact snapshot history reference is not retained",
            Self::EpochOrder => "snapshot history reset requires a newer epoch",
        })
    }
}
impl Error for SnapshotHistoryError {}
