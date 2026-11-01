// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ordinary server payloads bind decoded output to retained transmitted command keys.
#![allow(
    clippy::expect_used,
    reason = "Bounded explicit input/output/history fixtures."
)]
#[path = "server_payload_cases/limits.rs"]
mod limits;
#[path = "server_payload_cases/wire.rs"]
mod wire;

use cod4xr_client_core::{BaselineProtocol, server_payload::ServerPayloadDecoder};

const CHALLENGE: i32 = 0x1234_5678;
// Independent native-codebook vectors for byte opcodes NOP (0), EOF (7).
// Huffman e1 00 becomes 9e 7f at sequence 7 with an empty key, ff 42 with key a.
const INITIAL: [u8; 6] = [0, 0, 0, 0, 0x9e, 0x7f];
const KEYED: [u8; 6] = [1, 0, 0, 0, 0xff, 0x42];

fn decoder() -> ServerPayloadDecoder {
    ServerPayloadDecoder::new(BaselineProtocol::Cod4x21, CHALLENGE)
}
