// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::services::response;
use crate::{channel_support::accept, support::receive as datagram};
use cod4x_protocol::{MessageReader, MessageStringLimit, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    client_identities::{
        ClientIdentityBudget, ClientIdentityChange, ClientIdentitySlot, ClientIdentityState,
    },
    commands::{ClientCommandError, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan},
    gamestate::{GameStateBudget, GameStateStorage, ReliableGameStatePlan},
    server_commands::{ServerCommandHistory, ServerCommandUpdate},
    service_controls::{
        ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage, LegacyServicePlan,
        ServiceControlError,
    },
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};
use std::{
    net::UdpSocket,
    sync::mpsc::Receiver,
    time::{Duration, Instant},
};

pub(super) fn source(valid: bool) -> Vec<u8> {
    let mut bytes = b"\x04\x03\0\0\0Private%Stored\0".to_vec();
    for (sequence, index, name) in [
        (10_u32, 3, b"Player%".as_slice()),
        (12, 3, b"Gap"),
        (11, 63, b"Other\xff"),
    ] {
        bytes.push(11);
        bytes.extend_from_slice(&sequence.to_le_bytes());
        bytes.push(index);
        bytes.extend_from_slice(name);
        bytes.push(0);
        bytes.extend_from_slice(b"Tag%\0");
    }
    bytes.push(if valid { 7 } else { 0xfe });
    bytes
}
pub(super) fn serve(socket: &UdpSocket, proceed: &Receiver<()>) {
    let client = accept(socket);
    let (first, peer) = datagram(socket);
    assert_eq!(peer, client);
    assert_eq!(&first[..4], &1_u32.to_le_bytes());
    assert_eq!(
        &first[6..22],
        &[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 0, 0]
    );
    response(socket, client, 7, &source(false));
    proceed
        .recv_timeout(Duration::from_secs(4))
        .expect("Whole bad message rejected");
    response(socket, client, 8, &source(true));
    let (next, peer) = datagram(socket);
    assert_eq!(peer, client);
    assert_eq!(&next[..4], &2_u32.to_le_bytes());
    assert_eq!(&next[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(
        &next[6..22],
        &[0, 0, 0, 0, 8, 0, 0, 0, 3, 0, 0, 0, 11, 0, 0, 0]
    );
    let mut body = next[22..].to_vec();
    obfuscation::apply_client_payload(
        &mut body,
        (-2_147_481_234_i32).cast_unsigned(),
        0,
        8,
        b"Private.Stored",
    )
    .expect("Original connection and retained key");
    let mut output = [0; 128];
    let length = compression::decompress(&body, &mut output).expect("Next control");
    let mut reader = MessageReader::try_new(&output[..length]).expect("Control fields");
    assert_eq!(reader.read_bits(3).expect("Command"), 2);
    assert_eq!(reader.read_i32().expect("Sequence"), 2);
    let mut text = [0; 32];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Text");
    assert_eq!(&text[..length], b"next");
    assert_eq!(reader.read_bits(3).expect("EOF"), 3);
}
pub(crate) fn seed(state: &mut ClientIdentityState<'_>) {
    let body = [
        1, 0, 0, 0, 1, 0, 0, 0, 0, 7, 9, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];
    let plan = ReliableGameStatePlan::parse(
        &body,
        BaselineProtocol::Cod4x21,
        body.len(),
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
    .expect("Complete identity baseline");
    let _ = state
        .prepare_game_state(
            &plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 0,
            },
            &mut [],
        )
        .expect("Baseline preflight")
        .commit();
}
pub(crate) fn send(
    channel: &mut ClientChannel<'_>,
    client: &mut LegacyClientCommands<'_>,
    server: &ServerCommandHistory<'_>,
    identities: &ClientIdentityState<'_>,
) {
    let header = ClientControlHeader::Cod4x21 {
        server_id: 0,
        message_acknowledge: server.last_message_sequence(),
        server_command: server.latest_key().expect("Key"),
        config_acknowledge: identities.accepted_sequence().expect("Baseline"),
    };
    let frame = ControlMessagePlan::new(header, channel.context().server_challenge(), client)
        .expect("Control context");
    {
        let mut sender = channel.split().sender;
        let mut message = sender.begin_control(&frame).expect("Original owner");
        let deadline = Instant::now() + Duration::from_secs(3);
        while message.send_next(deadline).expect("Complete I/O").is_some() {}
    }
    client
        .mark_transmitted(frame.last_command_sequence().expect("Included command"))
        .expect("Actual transmission");
}
fn next(channel: &mut ClientChannel<'_>) -> (u32, Vec<u8>) {
    channel
        .split()
        .receiver
        .receive_until(
            Instant::now() + Duration::from_secs(3),
            |event| match event {
                ChannelEvent::Message(message) => {
                    Some((message.sequence(), message.body().to_vec()))
                }
                _ => None,
            },
        )
        .expect("Receive")
        .expect("Datagram")
        .expect("Ordinary body")
}
pub(super) fn receive_plan<'output>(
    channel: &mut ClientChannel<'_>,
    history: &LegacyClientCommands<'_>,
    private: &mut [ClientDataUpdateSlot],
    text: &'output mut [u8],
    updates: &'output mut [ClientDataUpdateSlot],
) -> Result<LegacyServicePlan<'output>, ServiceControlError> {
    let codec = channel.server_decoder().expect("Original decoder");
    let (sequence, mut body) = next(channel);
    let mut encrypted = [0; 1024];
    let mut service_bytes = [0; 1024];
    let result = {
        let payload = codec
            .decode(sequence, &body, history, &mut encrypted, &mut service_bytes)
            .expect("Complete transformed body");
        LegacyServicePlan::parse_with_client_data(
            &payload,
            ClientServiceBudget {
                max_records: 8,
                max_client_updates: 4,
            },
            &mut [0; 128],
            ClientServiceStorage {
                command_bytes: text,
                client_scratch: private,
                client_updates: updates,
            },
        )
    };
    body.fill(0);
    encrypted.fill(0);
    service_bytes.fill(0);
    result
}
pub(super) fn apply(
    plan: &LegacyServicePlan<'_>,
    client: &mut LegacyClientCommands<'_>,
    server: &mut ServerCommandHistory<'_>,
    identities: &mut ClientIdentityState<'_>,
    scratch: &mut [ClientIdentitySlot],
) -> Result<(ServerCommandUpdate, ClientIdentityChange), ClientCommandError> {
    let commands = server.prepare(plan).expect("Command resources");
    let updates = identities
        .prepare_service_plan(
            plan,
            ClientIdentityBudget {
                max_clients: 2,
                max_records: 3,
            },
            scratch,
        )
        .expect("Identity resources");
    client.commit_acknowledgement(plan.client_command_acknowledge())?;
    Ok((commands.commit(), updates.commit()))
}
