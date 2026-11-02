// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent native snapshot-header vectors and constituent cursor boundaries.
#![allow(clippy::expect_used, reason = "Finite native snapshot-header vectors.")]
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entities::EntityDecodeContext,
    message_body::BodyReader,
    player_state::LivePlayerState,
    snapshots::{SNAPSHOT_HEADER_BYTES, SNAPSHOT_OPCODE, SnapshotDelta, SnapshotHeader},
};

const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
#[test]
fn physical_header_vectors_preserve_time_bits_and_full_delta_selection() {
    assert_eq!(SNAPSHOT_OPCODE, 6);
    assert_eq!(SNAPSHOT_HEADER_BYTES, 6);
    for profile in PROFILES {
        for (bytes, time) in [
            ([0, 0, 0, 0, 0, 0], 0),
            ([255, 255, 255, 255, 0, 255], u32::MAX),
            ([0, 0, 0, 128, 0, 128], 0x8000_0000),
            ([0x78, 0x56, 0x34, 0x12, 0, 0xa5], 0x1234_5678),
        ] {
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Header body");
            let header = SnapshotHeader::parse(&mut reader, profile).expect("Header");
            assert_eq!(header.profile(), profile);
            assert_eq!(header.server_time(), time);
            assert_eq!(header.flags(), bytes[5]);
            assert_eq!(header.delta(), SnapshotDelta::Full);
            assert_eq!(header.delta().distance(), 0);
            assert_eq!(reader.remaining_bytes(), 0);
        }
    }
}
#[test]
fn every_nonzero_distance_and_flag_pattern_is_preserved_without_native_ring_limits() {
    for profile in PROFILES {
        for flags in 0..=255_u8 {
            let distance = flags.max(1);
            let bytes = [0xff, 0xff, 0xff, 0xff, distance, flags];
            let mut reader = BodyReader::try_new(&bytes, 6).expect("Body");
            let header = SnapshotHeader::parse(&mut reader, profile).expect("Raw header");
            assert!(
                matches!(header.delta(), SnapshotDelta::Previous(value) if value.get() == distance)
            );
            assert_eq!(header.delta().distance(), distance);
            assert_eq!(header.flags(), flags);
            assert_eq!(header.server_time(), u32::MAX);
        }
    }
}
#[test]
fn byte_header_operations_preserve_cached_bits_and_following_bytes() {
    for profile in PROFILES {
        let bytes = [0b1101_0101, 0x78, 0x56, 0x34, 0x12, 255, 0xa5, 0x9d];
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        assert_eq!(reader.read_bits(3).expect("Leading bits"), 5);
        let header = SnapshotHeader::parse(&mut reader, profile).expect("Header");
        assert_eq!(header.server_time(), 0x1234_5678);
        assert_eq!(header.delta().distance(), 255);
        assert_eq!(reader.byte_position(), 7);
        assert_eq!(reader.remaining_cached_bits(), 5);
        assert_eq!(reader.read_u8().expect("Following byte"), 0x9d);
        assert_eq!(reader.read_bits(5).expect("Retained cached bits"), 0b11010);
    }
}
#[test]
fn every_truncated_header_preserves_both_cursors_and_private_cached_value() {
    for profile in PROFILES {
        let bytes = [0xd5, 0x78, 0x56, 0x34, 0x12, 255, 0xa5];
        for length in 1..bytes.len() {
            let mut reader = BodyReader::try_new(&bytes[..length], length).expect("Truncated body");
            assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
            let before = position(&reader);
            assert!(SnapshotHeader::parse(&mut reader, profile).is_err());
            assert_eq!(position(&reader), before);
            assert_eq!(reader.read_bits(5).expect("Original cached value"), 0b11010);
        }
        for length in 0..6 {
            let mut reader =
                BodyReader::try_new(&bytes[1..][..length], length).expect("Short body");
            let before = position(&reader);
            assert!(SnapshotHeader::parse(&mut reader, profile).is_err());
            assert_eq!(position(&reader), before);
        }
    }
}
#[test]
fn following_player_state_starts_immediately_without_an_area_mask() {
    let bytes = [0x78, 0x56, 0x34, 0x12, 0, 0xa5, 1, 0, 0];
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    let header = SnapshotHeader::parse(&mut reader, BaselineProtocol::Cod4x21).expect("Header");
    let state = LivePlayerState::prepare(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        header.server_time(),
        None,
    )
    .expect("Immediate player state")
    .commit_transmitted()
    .expect("Transmitted state");
    assert_eq!(state.profile(), header.profile());
    assert_eq!(reader.byte_position(), 9);
    assert_eq!(reader.remaining_cached_bits(), 2);
}
#[test]
fn cached_player_state_bits_can_straddle_intervening_header_bytes() {
    let mut bytes = [0; 128];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    writer.write_bits(5, 3).expect("Unrelated leading bits");
    writer.write_i32(123).expect("Header time");
    writer.write_u8(0).expect("Full snapshot");
    writer.write_u8(255).expect("Raw flags");
    writer
        .write_bits(1, 1)
        .expect("Player origin mode in cached byte");
    writer.write_bits(0, 8).expect("No scalar fields");
    writer.write_bits(0, 13).expect("Player arrays");
    writer.write_u8(0x9d).expect("Following byte");
    let input = writer.as_bytes();
    let mut reader = BodyReader::try_new(input, input.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    let header = SnapshotHeader::parse(&mut reader, BaselineProtocol::Cod4x21).expect("Header");
    LivePlayerState::prepare(
        &mut reader,
        EntityDecodeContext::Cod4x21,
        header.server_time(),
        None,
    )
    .expect("Straddling state")
    .commit_transmitted()
    .expect("Transmitted");
    assert_eq!(reader.read_u8().expect("Following byte"), 0x9d);
}
#[test]
fn owned_header_outlives_source_and_debug_excludes_raw_time_and_flags() {
    for profile in PROFILES {
        let header = {
            let bytes = [0x10, 0x32, 0x54, 0x76, 1, 0xff];
            let mut reader = BodyReader::try_new(&bytes, 6).expect("Body");
            SnapshotHeader::parse(&mut reader, profile).expect("Header")
        };
        assert_eq!(header.server_time(), 0x7654_3210);
        let copy = header;
        assert_eq!(copy, header);
        let debug = format!("{header:?}");
        assert!(!debug.contains("1985229328"));
        assert!(!debug.contains("76543210"));
        assert!(!debug.contains("255"));
    }
}
