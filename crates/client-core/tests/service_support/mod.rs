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

// Same empty native baseline used by control/identity and snapshot-owner fixtures.
pub(crate) fn seed_identities(
    state: &mut cod4xr_client_core::client_identities::ClientIdentityState<'_>,
    sequence: u32,
) {
    use cod4xr_client_core::{
        client_identities::ClientIdentityBudget,
        gamestate::{GameStateBudget, GameStateStorage, ReliableGameStatePlan},
    };
    let mut input = vec![1, 0, 0, 0, 1, 0, 0, 0, 0, 7];
    for value in [sequence, 3, 0x1234_5678, 0] {
        input.extend_from_slice(&value.to_le_bytes());
    }
    let plan = ReliableGameStatePlan::parse(
        &input,
        BaselineProtocol::Cod4x21,
        input.len(),
        GameStateBudget {
            max_services: 1,
            max_configstrings: 0,
            max_text_bytes: 0,
            max_entities: 0,
            max_client_records: 0,
        },
        &mut [],
        GameStateStorage {
            configstring_bytes: &mut [],
            configstring_slots: &mut [],
            entity_slots: &mut [],
            client_slots: &mut [],
        },
    )
    .expect("Whole empty baseline");
    let _ = state
        .prepare_game_state(
            &plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 0,
            },
            &mut [],
        )
        .expect("Seed state")
        .commit();
}
