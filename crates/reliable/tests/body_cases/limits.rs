// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{
    MAX_RELIABLE_BODY_BYTES, MAX_RELIABLE_PAYLOAD_BYTES, ReliableError, ReliableHeader,
    ReliablePacket, SelectiveAckRange, SelectiveAcks,
};

use crate::{ACK_ONLY, DATA, header};

#[test]
fn maximum_profile_and_every_short_output_are_transactional_including_all_three_ranges() {
    let selective = SelectiveAcks::new(&[
        SelectiveAckRange {
            offset: 0,
            length: 1,
        },
        SelectiveAckRange {
            offset: 7,
            length: 3,
        },
        SelectiveAckRange {
            offset: 17,
            length: 2,
        },
    ])
    .expect("Three ranges");
    let fields = ReliableHeader {
        sequence: Some(u32::try_from(i32::MAX).expect("Positive bound")),
        acknowledge: u32::try_from(i32::MAX).expect("Positive bound"),
        flags: u8::MAX,
        selective,
        sender_window: i16::MIN,
    };
    let payload = [0xff; MAX_RELIABLE_PAYLOAD_BYTES];
    let packet = ReliablePacket::try_new(fields, &payload).expect("Maximum complete body");
    assert_eq!(packet.encoded_len(), MAX_RELIABLE_BODY_BYTES);
    for capacity in 0..packet.encoded_len() {
        let mut output = [0xa5; MAX_RELIABLE_BODY_BYTES + 3];
        assert_eq!(
            packet.encode(&mut output[..capacity]),
            Err(ReliableError::OutputCapacity)
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
    }
    let mut output = [0xa5; MAX_RELIABLE_BODY_BYTES + 3];
    assert_eq!(packet.encode(&mut output), Ok(MAX_RELIABLE_BODY_BYTES));
    assert_eq!(
        ReliablePacket::parse(&output[..MAX_RELIABLE_BODY_BYTES]),
        Ok(packet)
    );
    assert_eq!(&output[MAX_RELIABLE_BODY_BYTES..], &[0xa5; 3]);
}

#[test]
fn constructors_reject_unrepresentable_domains_and_ack_only_data_before_any_encoding() {
    assert_eq!(
        ReliablePacket::try_new(
            ReliableHeader {
                sequence: Some(u32::MAX),
                ..header()
            },
            b""
        )
        .expect_err("Sequence"),
        ReliableError::Sequence
    );
    assert_eq!(
        ReliablePacket::try_new(
            ReliableHeader {
                acknowledge: u32::MAX,
                ..header()
            },
            b""
        )
        .expect_err("Ack"),
        ReliableError::Acknowledgement
    );
    assert_eq!(
        ReliablePacket::try_new(
            ReliableHeader {
                sequence: None,
                ..header()
            },
            b"x"
        )
        .expect_err("ACK-only data"),
        ReliableError::AckOnlyData
    );
    assert_eq!(
        ReliablePacket::try_new(header(), &[0; MAX_RELIABLE_PAYLOAD_BYTES + 1])
            .expect_err("Payload"),
        ReliableError::PayloadCapacity
    );
    assert!(ReliablePacket::try_new(header(), b"").is_ok());
}

#[test]
fn all_truncations_bad_lengths_and_hidden_suffixes_reject_complete_body_framing() {
    for length in 0..DATA.len() {
        assert!(ReliablePacket::parse(&DATA[..length]).is_err());
    }
    let mut trailing = DATA.to_vec();
    trailing.push(0);
    assert_eq!(
        ReliablePacket::parse(&trailing).expect_err("Hidden suffix"),
        ReliableError::LengthMismatch
    );
    for length in [-1_i16, 1201] {
        let mut packet = ACK_ONLY;
        packet[12..14].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            ReliablePacket::parse(&packet).expect_err("Bad fragment size"),
            ReliableError::PayloadCapacity
        );
    }
    let mut packet = ACK_ONLY.to_vec();
    packet[12..14].copy_from_slice(&1_i16.to_le_bytes());
    assert_eq!(
        ReliablePacket::parse(&packet).expect_err("Missing suffix"),
        ReliableError::LengthMismatch
    );
    packet.push(0);
    assert_eq!(
        ReliablePacket::parse(&packet).expect_err("ACK-only payload"),
        ReliableError::AckOnlyData
    );
    assert_eq!(
        ReliablePacket::parse(&[0; MAX_RELIABLE_BODY_BYTES + 1]).expect_err("Whole input budget"),
        ReliableError::InputCapacity
    );
    let mut packet = ACK_ONLY;
    packet[..4].copy_from_slice(&(-2_i32).to_le_bytes());
    assert_eq!(
        ReliablePacket::parse(&packet).expect_err("Other negative sequence"),
        ReliableError::Sequence
    );
    packet[..4].copy_from_slice(&0_i32.to_le_bytes());
    packet[4..8].copy_from_slice(&(-1_i32).to_le_bytes());
    assert_eq!(
        ReliablePacket::parse(&packet).expect_err("Negative ack"),
        ReliableError::Acknowledgement
    );
}

#[test]
fn selective_count_and_signed_short_fields_reject_without_window_side_effects() {
    assert_eq!(
        SelectiveAcks::new(
            &[SelectiveAckRange {
                offset: 0,
                length: 1
            }; 4]
        ),
        Err(ReliableError::RangeCount)
    );
    for range in [
        SelectiveAckRange {
            offset: 0,
            length: 0,
        },
        SelectiveAckRange {
            offset: 32768,
            length: 1,
        },
        SelectiveAckRange {
            offset: 0,
            length: 32768,
        },
    ] {
        assert_eq!(SelectiveAcks::new(&[range]), Err(ReliableError::Range));
    }
    for count in 4..=u8::MAX {
        let mut packet = ACK_ONLY;
        packet[9] = count;
        assert_eq!(
            ReliablePacket::parse(&packet).expect_err("Excess ranges"),
            ReliableError::RangeCount
        );
    }
    for (offset, length) in [(-1_i16, 1_i16), (0, -1), (0, 0)] {
        let mut packet = DATA;
        packet[10..12].copy_from_slice(&offset.to_le_bytes());
        packet[12..14].copy_from_slice(&length.to_le_bytes());
        assert_eq!(
            ReliablePacket::parse(&packet).expect_err("Signed range"),
            ReliableError::Range
        );
    }
}
