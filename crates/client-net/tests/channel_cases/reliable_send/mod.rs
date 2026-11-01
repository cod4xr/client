// SPDX-License-Identifier: AGPL-3.0-or-later
mod acknowledgements;
mod data;
mod limits;

use std::{
    io,
    net::UdpSocket,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use cod4xr_client_net::{ChannelEvent, ClientReceiver};
use cod4xr_reliable::{
    ReliablePacket,
    receive::ReceiveWindow,
    send::{AcknowledgementUpdate, SendSlot, SendWindow, required_send_bytes},
};

use crate::channel_support::accept;

fn wire(
    sequence: i32,
    acknowledge: i32,
    ranges: &[(u16, u16)],
    hint: i16,
    payload: &[u8],
) -> Vec<u8> {
    let mut bytes = vec![0xf0, 0xff, 0xff, 0xff];
    bytes.extend_from_slice(&40_000_u16.to_le_bytes());
    bytes.extend_from_slice(&sequence.to_le_bytes());
    bytes.extend_from_slice(&acknowledge.to_le_bytes());
    bytes.extend_from_slice(&[0, u8::try_from(ranges.len()).expect("Fixture count")]);
    for (offset, length) in ranges {
        bytes.extend_from_slice(&offset.to_le_bytes());
        bytes.extend_from_slice(&length.to_le_bytes());
    }
    bytes.extend_from_slice(&hint.to_le_bytes());
    bytes.extend_from_slice(
        &i16::try_from(payload.len())
            .expect("Fixture payload")
            .to_le_bytes(),
    );
    bytes.extend_from_slice(payload);
    bytes
}

fn storage(capacity: usize) -> (Vec<u8>, Vec<SendSlot>) {
    (
        vec![0; required_send_bytes(capacity).expect("Send budget")],
        vec![SendSlot::default(); capacity],
    )
}

fn admit(
    packet: ReliablePacket<'_>,
    outgoing: &mut SendWindow<'_>,
    incoming: &mut ReceiveWindow<'_>,
) -> AcknowledgementUpdate {
    let received = incoming.prepare(packet).expect("Incoming preflight");
    let acknowledged = outgoing
        .prepare_acknowledgement(packet)
        .expect("Outgoing preflight");
    let update = acknowledged.commit();
    let _ = received.commit();
    update
}

fn receive_auxiliary(
    receiver: &mut ClientReceiver<'_, '_>,
    deadline: Instant,
    outgoing: &mut SendWindow<'_>,
    incoming: &mut ReceiveWindow<'_>,
) -> AcknowledgementUpdate {
    receiver
        .receive_until(deadline, |event| {
            let ChannelEvent::Auxiliary(message) = event else {
                return None;
            };
            Some(admit(
                message.reliable_packet().expect("Exact body"),
                outgoing,
                incoming,
            ))
        })
        .expect("Receive")
        .expect("One datagram")
        .expect("Auxiliary family")
}

fn quiet_peer(socket: UdpSocket) -> (thread::JoinHandle<()>, mpsc::SyncSender<()>) {
    let (finished, completion) = mpsc::sync_channel(0);
    let responder = thread::spawn(move || {
        let client = accept(&socket);
        completion
            .recv_timeout(Duration::from_secs(5))
            .expect("Sender finished");
        socket
            .set_nonblocking(true)
            .expect("Inspect pending traffic");
        for _ in 0..8 {
            let mut bytes = [0; 1400];
            match socket.recv_from(&mut bytes) {
                Ok((length, source)) => {
                    assert_eq!(source, client);
                    // A slow scheduler may leave a duplicate connect attempt queued.
                    assert!(bytes[..length].starts_with(b"\xff\xff\xff\xffconnect \""));
                }
                Err(error) => {
                    assert_eq!(error.kind(), io::ErrorKind::WouldBlock);
                    return;
                }
            }
        }
        assert_eq!(
            socket
                .peek_from(&mut [0; 1])
                .expect_err("Bounded pending traffic")
                .kind(),
            io::ErrorKind::WouldBlock
        );
    });
    (responder, finished)
}
