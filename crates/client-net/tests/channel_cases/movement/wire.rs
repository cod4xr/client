// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{channel_support::accept, services::response_with_key, support::receive};
use cod4x_protocol::{
    MessageReader, MessageStringLimit, compression, obfuscation,
    user_command::{self, LegacyUserCommand},
};
use std::net::UdpSocket;

pub(super) const FEED: u32 = 0x8765_4321;
pub(super) const ORIGIN: [u32; 3] = [0x3f80_0000, 0xc000_0000, 0x8000_0000];
pub(super) fn serve(socket: &UdpSocket) {
    let client = accept(socket);
    response_with_key(socket, client, 7, 0, b"", b"\x04\x09\0\0\0\x92\0\x07");
    let (packet, peer) = receive(socket);
    assert_eq!(peer, client);
    assert_eq!(&packet[..4], &1_u32.to_le_bytes());
    assert_eq!(&packet[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(&packet[6..10], &0x1122_3344_u32.to_le_bytes());
    assert_eq!(&packet[10..14], &7_u32.to_le_bytes());
    assert_eq!(&packet[14..18], &9_u32.to_le_bytes());
    assert_eq!(&packet[18..22], &0_u32.to_le_bytes());
    let mut compressed = packet[22..].to_vec();
    obfuscation::apply_client_payload(
        &mut compressed,
        (-2_147_481_234_i32).cast_unsigned(),
        0x1122_3344,
        7,
        b"\x92",
    )
    .expect("Actual connection and retained incoming key");
    let mut plain = [0; 2048];
    let length =
        compression::decompress(&compressed, &mut plain).expect("Complete actual movement packet");
    let mut reader = MessageReader::try_new(&plain[..length]).expect("Native payload");
    assert_eq!(reader.read_bits(3).expect("Reliable command"), 2);
    assert_eq!(reader.read_i32().expect("Reliable sequence"), 1);
    let mut text = [0; 32];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Reliable text");
    assert_eq!(&text[..length], b"reliable");
    assert_eq!(reader.read_bits(3).expect("No-delta movement"), 1);
    assert_eq!(reader.read_u8().expect("Count"), 3);
    // The high-byte hash is an independently compiler-evaluated public C vector.
    let key = FEED ^ 7 ^ 0xffff_ccd2;
    let mut prior = LegacyUserCommand::default();
    for index in 0_u8..3 {
        let command =
            user_command::read_delta(&mut reader, key, &prior).expect("Shared command decoder");
        assert_eq!(command.server_time, 100 + i32::from(index) * 16);
        assert_eq!(command.buttons, 1);
        assert_eq!(command.angles, [0x1234, 0x8000, 0xffff]);
        assert_eq!((command.weapon, command.offhand), (31 + index, 0));
        assert_eq!((command.forward_move, command.right_move), (-127, 127));
        prior = command;
    }
    for expected in ORIGIN {
        assert_eq!(
            reader
                .read_i32()
                .expect("Predicted origin bits")
                .cast_unsigned(),
            expected
        );
    }
    assert_eq!(reader.read_i32().expect("Prediction time"), 132);
    assert_eq!(reader.read_bits(3).expect("EOF"), 3);
    response_with_key(socket, client, 8, 1, b"reliable", b"\x07");
}
