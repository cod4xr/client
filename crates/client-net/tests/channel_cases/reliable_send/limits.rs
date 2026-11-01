// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::{Duration, Instant};

use cod4xr_client_net::{NetworkError, SendUsage};
use cod4xr_reliable::{
    ReliableError, ReliablePacket, SelectiveAcks,
    receive::{ReceiveAcknowledgement, ReceiveSlot, ReceiveWindow},
    send::{SendError, SendWindow},
};

use super::{quiet_peer, storage, wire};
use crate::{channel_support::acknowledged_for, support::server};

#[test]
fn legacy_and_unknown_original_profiles_reject_auxiliary_sends_before_any_io_or_transmit_admission()
{
    for protocol in [6, 22] {
        let socket = server();
        let peer = socket.local_addr().expect("Peer");
        let (responder, finished) = quiet_peer(socket);
        let mut ordinary_storage = [0; 1300];
        let mut channel = acknowledged_for(peer, protocol)
            .into_channel(&mut ordinary_storage)
            .expect("Opaque channel");
        let (mut bytes, mut slots) = storage(4);
        let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
        let _ = outgoing.enqueue(b"x").expect("Unsent");
        let invalid = ReceiveAcknowledgement {
            cumulative: u32::MAX,
            selective: SelectiveAcks::default(),
            next_offset: 0,
        };
        let deadline = Instant::now() + Duration::from_secs(4);
        {
            let mut sender = channel.split().sender;
            let tx = outgoing.prepare_transmission(0).expect("Prepare");
            assert!(matches!(
                sender.send_reliable_fragment(tx, invalid, deadline),
                Err(NetworkError::AuxiliaryProfile)
            ));
            assert!(matches!(
                sender.send_reliable_ack(&outgoing, invalid, deadline),
                Err(NetworkError::AuxiliaryProfile)
            ));
            assert_eq!(sender.next_sequence(), 1);
        }
        assert_eq!(channel.send_usage(), SendUsage::default());
        let ack = wire(-1, 1, &[], 4, b"");
        assert_eq!(
            outgoing
                .prepare_acknowledgement(ReliablePacket::parse(&ack[6..]).expect("Body"))
                .expect_err("Never sent"),
            SendError::AckUnsent
        );
        finished.send(()).expect("Release quiet peer");
        responder.join().expect("No unexpected packet");
    }
}

#[test]
fn invalid_receive_metadata_on_the_reviewed_profile_cannot_mark_data_or_charge_datagram_io() {
    let socket = server();
    let peer = socket.local_addr().expect("Peer");
    let (responder, finished) = quiet_peer(socket);
    let mut ordinary_storage = [0; 1300];
    let mut channel = acknowledged_for(peer, 21)
        .into_channel(&mut ordinary_storage)
        .expect("Channel");
    let (mut bytes, mut slots) = storage(4);
    let mut outgoing = SendWindow::new(&mut bytes, &mut slots, 4).expect("Outgoing");
    let _ = outgoing.enqueue(b"PrivateRPC").expect("Unsent");
    let mut receive_bytes = [0; 1200];
    let mut receive_slots = [ReceiveSlot::default(); 1];
    let incoming = ReceiveWindow::new(&mut receive_bytes, &mut receive_slots).expect("Incoming");
    let invalid = ReceiveAcknowledgement {
        cumulative: u32::MAX,
        ..incoming.acknowledgement(0)
    };
    let deadline = Instant::now() + Duration::from_secs(4);
    {
        let mut sender = channel.split().sender;
        let tx = outgoing.prepare_transmission(0).expect("Prepare");
        assert!(matches!(
            sender.send_reliable_fragment(tx, invalid, deadline),
            Err(NetworkError::Reliable(ReliableError::Acknowledgement))
        ));
        assert!(matches!(
            sender.send_reliable_ack(&outgoing, invalid, deadline),
            Err(NetworkError::Reliable(ReliableError::Acknowledgement))
        ));
        assert_eq!(sender.next_sequence(), 1);
    }
    assert_eq!(channel.send_usage(), SendUsage::default());
    assert_eq!(
        (outgoing.queued_fragments(), outgoing.buffered_bytes()),
        (1, 10)
    );
    let ack = wire(-1, 1, &[], 4, b"");
    assert_eq!(
        outgoing
            .prepare_acknowledgement(ReliablePacket::parse(&ack[6..]).expect("Body"))
            .expect_err("Never sent"),
        SendError::AckUnsent
    );
    finished.send(()).expect("Release quiet peer");
    responder.join().expect("No unexpected packet");
}
