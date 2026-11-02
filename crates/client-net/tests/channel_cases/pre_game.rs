// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    command_history::send_next,
    services::{response, send_control},
};
use crate::{
    channel_support::{accept, acknowledged},
    support::{receive, server},
};
use cod4x_protocol::{MessageReader, MessageStringLimit, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    pre_game::{PreGameAction, PreGameBudget, PreGameError},
    server_commands::{SERVER_COMMAND_SLOT_BYTES, ServerCommandHistory, ServerCommandSlot},
    service_controls::{LegacyServicePlan, ServiceControlBudget},
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};
use std::{
    net::UdpSocket,
    thread,
    time::{Duration, Instant},
};

fn serve(socket: &UdpSocket) {
    let client = accept(socket);
    let (first, peer) = receive(socket);
    assert_eq!(peer, client);
    assert_eq!(&first[..4], &1_u32.to_le_bytes());
    response(socket, client, 7, b"\x04\x01\0\0\0l \"Private%Map\" dm\0\x04\x02\0\0\0w Private%Reason PB\0\x04\x03\0\0\0m\0\x07");
    let (next, peer) = receive(socket);
    assert_eq!(peer, client);
    assert_eq!(&next[..4], &2_u32.to_le_bytes());
    assert_eq!(&next[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(
        &next[6..22],
        &[0, 0, 0, 0, 7, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0]
    );
    let mut encoded = next[22..].to_vec();
    obfuscation::apply_client_payload(
        &mut encoded,
        u32::from_le_bytes((-2_147_481_234_i32).to_le_bytes()),
        0,
        7,
        b"m",
    )
    .expect("Latest pre-game command remains key");
    let mut decoded = [0; 128];
    let length = compression::decompress(&encoded, &mut decoded).expect("Next actual UDP control");
    let mut reader = MessageReader::try_new(&decoded[..length]).expect("Control");
    assert_eq!(reader.read_bits(3).expect("Command"), 2);
    assert_eq!(reader.read_i32().expect("Sequence"), 2);
    let mut text = [0; 32];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Next text");
    assert_eq!(&text[..length], b"next");
    assert_eq!(reader.read_bits(3).expect("EOF"), 3);
}

fn receive_plan<'output>(
    channel: &mut ClientChannel<'_>,
    outgoing: &LegacyClientCommands<'_>,
    output: &'output mut [u8],
) -> LegacyServicePlan<'output> {
    let decoder = channel.server_decoder().expect("Original decoder");
    let (sequence, mut bytes) = channel
        .split()
        .receiver
        .receive_until(
            Instant::now() + Duration::from_secs(2),
            |event| match event {
                ChannelEvent::Message(message) => {
                    Some((message.sequence(), message.body().to_vec()))
                }
                _ => None,
            },
        )
        .expect("Receive I/O")
        .expect("Datagram")
        .expect("Ordinary body");
    let mut work = [0; 1024];
    let mut service_bytes = [0; 1024];
    let plan = {
        let payload = decoder
            .decode(sequence, &bytes, outgoing, &mut work, &mut service_bytes)
            .expect("Payload");
        LegacyServicePlan::parse(
            &payload,
            ServiceControlBudget::default(),
            &mut [0; 128],
            output,
        )
        .expect("Whole stream")
    };
    bytes.fill(0);
    work.fill(0);
    service_bytes.fill(0);
    plan
}

#[test]
fn typed_notice_preflight_cancels_then_commits_without_losing_next_udp_key_or_ack() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let responder = thread::spawn(move || serve(&socket));
    let mut reassembly = [0; 1300];
    let mut channel = acknowledged(peer)
        .into_channel(&mut reassembly)
        .expect("Original channel");
    let mut outgoing_bytes = [0; 3 * LEGACY_COMMAND_SLOT_BYTES];
    let mut outgoing = LegacyClientCommands::new(&mut outgoing_bytes).expect("Outgoing");
    outgoing.enqueue(b"a%").expect("Initial command");
    send_control(&mut channel, &mut outgoing);
    let mut output = [0; 128];
    let plan = receive_plan(&mut channel, &outgoing, &mut output);
    let mut retained_bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
    let mut slots = [ServerCommandSlot::default()];
    let mut history =
        ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut retained_bytes, &mut slots)
            .expect("One retained slot");
    let budget = PreGameBudget {
        max_command_bytes: 128,
        max_events: 3,
    };
    assert_eq!(
        history
            .prepare(&plan)
            .expect("Command preflight")
            .with_pre_game(PreGameBudget {
                max_events: 2,
                ..budget
            })
            .err(),
        Some(PreGameError::EventBudget)
    );
    assert_eq!(
        (
            history.last_message_sequence(),
            outgoing.acknowledged_sequence(),
            outgoing.pending_count()
        ),
        (0, 0, 1)
    );
    let typed = history
        .prepare(&plan)
        .expect("Retry")
        .with_pre_game(budget)
        .expect("Typed preflight");
    assert_eq!(typed.events().count(), 3);
    drop(typed);
    assert_eq!(
        (history.newest_sequence(), outgoing.acknowledged_sequence()),
        (0, 0)
    );
    let typed = history
        .prepare(&plan)
        .expect("Retry")
        .with_pre_game(budget)
        .expect("All notices admitted");
    outgoing
        .commit_acknowledgement(plan.client_command_acknowledge())
        .expect("ACK after resource checks");
    let committed = typed.commit();
    assert_eq!(committed.summary().new_commands, 3);
    let events: Vec<_> = committed.events().collect();
    assert_eq!(
        events
            .iter()
            .map(|event| event.sequence())
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    let PreGameAction::LoadMap { map, gametype } = events[0].action() else {
        unreachable!("Map fixture")
    };
    assert_eq!(
        (map.as_bytes(), gametype.as_bytes()),
        (b"Private.Map".as_slice(), b"dm".as_slice())
    );
    let PreGameAction::Disconnect { reason, punkbuster } = events[1].action() else {
        unreachable!("Disconnect fixture")
    };
    assert_eq!(reason.expect("Reason").as_bytes(), b"Private.Reason");
    assert!(punkbuster);
    assert_eq!(events[2].action(), PreGameAction::FastRestart);
    assert!(!format!("{committed:?} {events:?}").contains("Private"));
    outgoing.enqueue(b"next").expect("Next command");
    send_next(&mut channel, &mut outgoing, &history);
    responder.join().expect("Peer");
    assert_eq!(
        (outgoing.acknowledged_sequence(), outgoing.pending_count()),
        (1, 1)
    );
    assert_eq!(history.command(3).expect("Last slot").as_bytes(), b"m");
    assert!(history.command(1).is_err());
}
