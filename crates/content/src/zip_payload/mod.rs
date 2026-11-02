// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-funded stored/raw-Deflate extraction with whole-stream length and CRC checks.
//! Successful payloads remain data only, without path, mount or execution authority.

mod decoder;
mod error;
mod payload;

pub use decoder::ZipPayloadDecoder;
pub use error::ZipPayloadError;
pub use payload::CheckedZipPayload;

/// Explicit per-operation work/output budgets, independent of archive size declarations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ZipPayloadBudget {
    /// Maximum compressed bytes to inspect in one entry.
    pub max_compressed_bytes: usize,
    /// Maximum declared output bytes admitted before decoding or checksum work.
    pub max_uncompressed_bytes: u64,
}
