// SPDX-License-Identifier: AGPL-3.0-or-later
//! Explicit unsigned PC version-5 fastfile framing, zlib integrity and size metadata.
//! No serialized pointers are followed, memory blocks allocated or assets registered.
mod decoder;
mod error;
mod metadata;
mod view;

pub use decoder::Fastfile5Decoder;
pub use error::Fastfile5Error;
pub use metadata::{Fastfile5Block, Fastfile5Metadata};
pub use view::CheckedFastfile5;

/// Independent host policy for encoded bytes, decoded bytes and declared memory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Fastfile5Budget {
    /// Maximum complete encoded file, including its twelve-byte outer header.
    pub max_file_bytes: usize,
    /// Maximum complete decoded section, including its forty-four-byte size header.
    pub max_decoded_bytes: usize,
    /// Maximum wide sum of all nine declared block sizes; no block is allocated here.
    pub max_total_block_bytes: u64,
    /// Maximum declared external size, independently of decoded and block sizes.
    pub max_external_bytes: u64,
}
