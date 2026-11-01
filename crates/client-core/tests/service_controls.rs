// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete service control plans expose normalized records only after logical EOF.
#![allow(
    clippy::expect_used,
    reason = "Finite service vectors use explicit caller buffers."
)]
#[path = "service_cases/limits.rs"]
mod limits;
#[path = "service_cases/records.rs"]
mod records;

use cod4x_protocol::{MAX_MESSAGE_BYTES, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::LegacyClientCommands,
    server_payload::{DecodedServerPayload, ServerPayloadDecoder},
};

struct Fixture {
    packet: Vec<u8>,
    work: Vec<u8>,
    output: Vec<u8>,
}

impl Fixture {
    fn new(source: &[u8], acknowledge: u32, key: &[u8]) -> Self {
        // This fixture composes already verified codecs; record vectors below are literal.
        let mut packet = vec![0; MAX_MESSAGE_BYTES];
        packet[..4].copy_from_slice(&acknowledge.to_le_bytes());
        let length = compression::compress(source, &mut packet[4..]).expect("Fixture compression");
        packet.truncate(4 + length);
        obfuscation::apply_server_payload(&mut packet[4..], 0x1234_5678, 7, key)
            .expect("Fixture transform");
        Self {
            packet,
            work: vec![0; MAX_MESSAGE_BYTES],
            output: vec![0; MAX_MESSAGE_BYTES],
        }
    }

    fn decode<'decoded, 'history>(
        &'decoded mut self,
        history: &'history LegacyClientCommands<'_>,
    ) -> DecodedServerPayload<'decoded, 'history> {
        ServerPayloadDecoder::new(BaselineProtocol::Cod4x21, 0x1234_5678)
            .decode(7, &self.packet, history, &mut self.work, &mut self.output)
            .expect("Fixture decode")
    }
}

fn append_command(output: &mut Vec<u8>, sequence: i32, text: &[u8]) {
    output.push(4);
    output.extend_from_slice(&sequence.to_le_bytes());
    output.extend_from_slice(text);
    output.push(0);
}
