// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{Buffers, ms, transmit};
use cod4xr_reliable::{
    ReliablePacket,
    endpoint::{DrainStatus, EndpointError, ReliableEndpoint},
    receive::ReceiveError,
};

fn inspect(endpoint: &mut ReliableEndpoint<'_>, bodies: &mut Vec<Vec<u8>>) {
    let run = endpoint.drain(&mut [0; 113], 400, 8);
    assert!(run.usage.bytes <= 400 && run.usage.examined <= 8);
    assert!(run.outcome.is_ok());
    if run.outcome == Ok(DrainStatus::Ready) {
        bodies.push(endpoint.frame().expect("Complete only").body().to_vec());
        assert!(endpoint.consume_frame());
    }
}

fn deliver(
    endpoint: &mut ReliableEndpoint<'_>,
    packet: ReliablePacket<'_>,
    now: std::time::Duration,
) -> bool {
    match endpoint.ingest(packet, now) {
        Ok(_) => true,
        Err(error) => {
            assert_eq!(error, EndpointError::Receive(ReceiveError::OutOfWindow));
            false // A smaller receive span applies backpressure until a later retry.
        }
    }
}

#[test]
fn two_bounded_endpoints_exchange_large_and_small_frames_under_loss_retransmission_and_duplicates()
{
    let mut left_buffers = Buffers::new(7, 3, 6000);
    let mut right_buffers = Buffers::new(7, 3, 6000);
    let mut left = left_buffers.endpoint();
    let mut right = right_buffers.endpoint();
    let expected: Vec<u8> = (0_usize..5000)
        .map(|index| index.to_le_bytes()[0])
        .collect();
    let _ = left
        .enqueue(&expected)
        .expect("Large frame within send budget");
    let _ = right.enqueue(b"reply").expect("Opposite direction");
    let mut left_bodies = Vec::new();
    let mut right_bodies = Vec::new();
    let mut lost = false;
    let mut duplicate = false;
    for tick in (0..50_000).step_by(50) {
        let now = ms(tick);
        if let Some(bytes) = transmit(&mut left, now).expect("Paced left") {
            let packet = ReliablePacket::parse(&bytes).expect("Complete body");
            if packet.header().sequence == Some(1) && !lost {
                lost = true;
            } else {
                let accepted = deliver(&mut right, packet, now);
                if accepted && packet.header().sequence == Some(2) && !duplicate {
                    assert!(deliver(&mut right, packet, now));
                    duplicate = true;
                }
            }
        }
        inspect(&mut right, &mut right_bodies);
        if let Some(bytes) = transmit(&mut right, now).expect("Paced right") {
            let _ = left
                .ingest(ReliablePacket::parse(&bytes).expect("Complete body"), now)
                .expect("Whole reverse receive");
        }
        inspect(&mut left, &mut left_bodies);
        if left.state().outgoing_fragments == 0
            && right.state().outgoing_fragments == 0
            && !left_bodies.is_empty()
            && !right_bodies.is_empty()
        {
            break;
        }
    }
    assert!(lost && duplicate);
    assert_eq!(left_bodies, [b"reply".to_vec()]);
    assert_eq!(right_bodies, [expected]);
    assert_eq!(
        (
            left.state().outgoing_fragments,
            right.state().outgoing_fragments
        ),
        (0, 0)
    );
    assert_eq!(
        (
            left.state().incoming_fragments,
            right.state().incoming_fragments
        ),
        (0, 0)
    );
}
