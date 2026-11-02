// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native snapshot framing constituents without history or message admission.
//! Explicit profiles select codecs; the host owns reference epochs and retention.

mod header;

pub use header::{SnapshotDelta, SnapshotHeader};

/// Native snapshot service tag; the header parser starts immediately after it.
pub const SNAPSHOT_OPCODE: u8 = 6;
/// Physical byte fields before the native player-state bitstream.
pub const SNAPSHOT_HEADER_BYTES: usize = 6;
