// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Checked compression and service fixtures."
)]
use cod4x_protocol::{MAX_MESSAGE_BYTES, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::LegacyClientCommands,
    server_payload::{DecodedServerPayload, ServerPayloadDecoder},
};

pub(crate) struct Fixture {
    packet: Vec<u8>,
    work: Vec<u8>,
    output: Vec<u8>,
    sequence: u32,
}

impl Fixture {
    pub(crate) fn packet(&self) -> &[u8] {
        &self.packet
    }

    pub(crate) fn new(source: &[u8], acknowledge: u32, key: &[u8]) -> Self {
        Self::at_sequence(source, acknowledge, key, 7)
    }

    pub(crate) fn at_sequence(source: &[u8], acknowledge: u32, key: &[u8], sequence: u32) -> Self {
        // Fixture composition only; dedicated payload tests use independent golden bytes.
        let mut packet = vec![0; MAX_MESSAGE_BYTES];
        packet[..4].copy_from_slice(&acknowledge.to_le_bytes());
        let length = compression::compress(source, &mut packet[4..]).expect("Fixture compression");
        packet.truncate(4 + length);
        obfuscation::apply_server_payload(&mut packet[4..], 0x1234_5678, sequence, key)
            .expect("Fixture transform");
        Self {
            packet,
            work: vec![0; MAX_MESSAGE_BYTES],
            output: vec![0; MAX_MESSAGE_BYTES],
            sequence,
        }
    }

    pub(crate) fn decode<'decoded, 'history>(
        &'decoded mut self,
        history: &'history LegacyClientCommands<'_>,
    ) -> DecodedServerPayload<'decoded, 'history> {
        self.decode_profile(history, BaselineProtocol::Cod4x21)
    }

    pub(crate) fn decode_profile<'decoded, 'history>(
        &'decoded mut self,
        history: &'history LegacyClientCommands<'_>,
        profile: BaselineProtocol,
    ) -> DecodedServerPayload<'decoded, 'history> {
        ServerPayloadDecoder::new(profile, 0x1234_5678)
            .decode(
                self.sequence,
                &self.packet,
                history,
                &mut self.work,
                &mut self.output,
            )
            .expect("Fixture decode")
    }
}

pub(crate) fn append_command(output: &mut Vec<u8>, sequence: i32, text: &[u8]) {
    output.push(4);
    output.extend_from_slice(&sequence.to_le_bytes());
    output.extend_from_slice(text);
    output.push(0);
}
