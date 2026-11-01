// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::{MAX_RELIABLE_BODY_BYTES, ReliablePacket, SelectiveAckRange, SelectiveAcks};

use crate::{ACK_ONLY, DATA, header};

#[test]
fn native_ack_only_vector_has_minus_one_sequence_and_no_payload_or_outer_envelope() {
    let packet = ReliablePacket::parse(&ACK_ONLY).expect("Native ACK-only body");
    let fields = packet.header();
    assert_eq!(fields.sequence, None);
    assert_eq!(fields.acknowledge, 0);
    assert!(!fields.force_ack());
    assert!(fields.selective.is_empty());
    assert_eq!(fields.sender_window, 4);
    assert!(packet.payload().is_empty());
    assert_eq!(packet.encoded_len(), 14);
    let mut output = [0xa5; 18];
    assert_eq!(packet.encode(&mut output), Ok(14));
    assert_eq!(&output[..14], &ACK_ONLY);
    assert_eq!(&output[14..], &[0xa5; 4]);
}

#[test]
fn native_data_vector_preserves_relative_selective_fields_and_exact_binary_suffix() {
    let packet = ReliablePacket::parse(&DATA).expect("Native data body");
    let fields = packet.header();
    assert_eq!(fields.sequence, Some(2));
    assert_eq!(fields.acknowledge, 4);
    assert!(fields.force_ack());
    let ranges = [
        SelectiveAckRange {
            offset: 1,
            length: 2,
        },
        SelectiveAckRange {
            offset: 5,
            length: 1,
        },
    ];
    assert_eq!(fields.selective.ranges(), ranges);
    assert_eq!(fields.selective.len(), 2);
    assert_eq!(packet.payload(), &[0xaa, 0, 0xff]);
    let rebuilt = ReliablePacket::try_new(fields, packet.payload())
        .expect("Owned metadata and borrowed data");
    let mut output = [0xa5; MAX_RELIABLE_BODY_BYTES];
    assert_eq!(rebuilt.encode(&mut output), Ok(DATA.len()));
    assert_eq!(&output[..DATA.len()], &DATA);
}

#[test]
fn unknown_flag_bits_and_ignored_sender_hints_remain_data_without_selecting_behavior() {
    for flags in 0..=u8::MAX {
        for sender_window in [i16::MIN, -1, 0, 4, i16::MAX] {
            let packet = ReliablePacket::try_new(
                cod4xr_reliable::ReliableHeader {
                    flags,
                    sender_window,
                    ..header()
                },
                b"",
            )
            .expect("Representable metadata");
            let mut output = [0; 14];
            packet.encode(&mut output).expect("Encode");
            let decoded = ReliablePacket::parse(&output).expect("Parse");
            assert_eq!(decoded.header().flags, flags);
            assert_eq!(decoded.header().sender_window, sender_window);
            assert_eq!(decoded.header().force_ack(), flags & 1 != 0);
        }
    }
}

#[test]
fn metadata_owns_selective_ranges_and_debug_excludes_private_payload_bytes() {
    let mut ranges = [SelectiveAckRange {
        offset: 1,
        length: 2,
    }];
    let selective = SelectiveAcks::new(&ranges).expect("Copy bounded metadata");
    ranges[0].offset = 9;
    assert_eq!(ranges[0].offset, 9);
    let packet = ReliablePacket::try_new(
        cod4xr_reliable::ReliableHeader {
            selective,
            ..header()
        },
        b"PrivateReliableData",
    )
    .expect("Private payload");
    assert_eq!(packet.header().selective.ranges()[0].offset, 1);
    assert_eq!(packet.payload(), b"PrivateReliableData");
    assert!(!format!("{packet:?}").contains("PrivateReliableData"));
}
