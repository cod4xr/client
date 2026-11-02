// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    channel_support::accept,
    services::{response, response_with_key},
    support::receive,
};
use cod4x_protocol::{MessageReader, MessageStringLimit, MessageWriter, compression, obfuscation};
use std::{
    net::{SocketAddr, UdpSocket},
    sync::mpsc::Receiver,
    time::Duration,
};

fn string(writer: &mut MessageWriter<'_>, text: &[u8]) {
    for &byte in text {
        writer.write_u8(byte).expect("Text byte");
    }
    writer.write_u8(0).expect("String terminator");
}
fn command(writer: &mut MessageWriter<'_>, sequence: i32, text: &[u8]) {
    writer.write_u8(4).expect("Command service");
    writer.write_i32(sequence).expect("Command sequence");
    string(writer, text);
}
fn identity(writer: &mut MessageWriter<'_>, sequence: i32, name: &[u8]) {
    writer.write_u8(11).expect("Identity service");
    writer.write_i32(sequence).expect("Identity sequence");
    writer.write_u8(3).expect("Client index");
    string(writer, name);
    string(writer, b"Tag%");
}
fn source(valid: bool, distance: u8) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Wire fixture");
    command(&mut writer, 3, b"Private%Before");
    identity(&mut writer, 10, b"Before%");
    writer.write_u8(6).expect("Snapshot service");
    writer.write_i32(123).expect("Server time");
    writer
        .write_u8(distance)
        .expect("Actual reference distance");
    writer.write_u8(0).expect("Flags");
    writer.write_bits(0, 1).expect("Exact prediction required");
    writer.write_bits(0, 8).expect("Unchanged scalar prefix");
    writer.write_bits(0, 13).expect("Unchanged player arrays");
    writer.write_bits(6, 12).expect("Absolute entity one");
    writer.write_bits(0, 1).expect("Not removed");
    writer
        .write_bits(u32::from(distance == 0), 1)
        .expect("Entity delta");
    if distance == 0 {
        writer.write_bits(1, 6).expect("Kind prefix");
        writer.write_bits(1, 1).expect("Changed kind");
        writer.write_bits(1, 1).expect("Nonzero kind");
        writer.write_u8(13).expect("Kind XOR");
    }
    writer.write_bits(0xffe, 12).expect("Entity terminator");
    writer
        .write_bits(253, 8)
        .expect("More and absolute client 63");
    writer.write_bits(0, 1).expect("Not removed");
    writer
        .write_bits(u32::from(distance == 0), 1)
        .expect("Client delta");
    if distance == 0 {
        writer.write_bits(1, 5).expect("Model prefix");
        writer.write_bits(1, 1).expect("Changed model");
        writer.write_bits(1, 1).expect("Nonzero model");
        writer.write_bits(1, 1).expect("Model residual bit");
        writer.write_u8(255).expect("Remaining model XOR");
    }
    writer.write_bits(0, 1).expect("Client stream terminator");
    identity(&mut writer, 11, b"After");
    command(&mut writer, 4, b"Private%After");
    writer.write_u8(if valid { 7 } else { 0xfe }).expect("Tail");
    writer.as_bytes().to_vec()
}
pub(super) fn serve(socket: &UdpSocket, proceed: &Receiver<()>) {
    let client = accept(socket);
    let (first, peer) = receive(socket);
    assert_eq!(peer, client);
    assert_eq!(&first[..4], &1_u32.to_le_bytes());
    assert_eq!(&first[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(&first[6..18], &[0; 12]);
    assert_eq!(&first[18..22], &9_u32.to_le_bytes());
    response(socket, client, 7, &source(false, 0));
    proceed
        .recv_timeout(Duration::from_secs(4))
        .expect("Rejected malformed tail");
    response(socket, client, 8, &source(true, 0));
    control(socket, client, 2, 8, b"next");
    response_with_key(socket, client, 11, 2, b"next", &source(true, 3));
    control(socket, client, 3, 11, b"last");
}
fn control(
    socket: &UdpSocket,
    client: SocketAddr,
    sequence: u32,
    message_acknowledge: u32,
    expected_text: &[u8],
) {
    let (next, peer) = receive(socket);
    assert_eq!(peer, client);
    assert_eq!(&next[..4], &sequence.to_le_bytes());
    assert_eq!(&next[4..6], &40_000_u16.to_le_bytes());
    assert_eq!(&next[6..10], &0_u32.to_le_bytes());
    assert_eq!(&next[10..14], &message_acknowledge.to_le_bytes());
    assert_eq!(&next[14..18], &4_u32.to_le_bytes());
    assert_eq!(&next[18..22], &11_u32.to_le_bytes());
    let mut body = next[22..].to_vec();
    obfuscation::apply_client_payload(
        &mut body,
        (-2_147_481_234_i32).cast_unsigned(),
        0,
        message_acknowledge,
        b"Private.After",
    )
    .expect("Original connection and retained command key");
    let mut plain = [0; 128];
    let length = compression::decompress(&body, &mut plain).expect("Actual next control");
    let mut reader = MessageReader::try_new(&plain[..length]).expect("Control body");
    assert_eq!(reader.read_bits(3).expect("Command opcode"), 2);
    assert_eq!(
        reader.read_i32().expect("Command sequence").cast_unsigned(),
        sequence
    );
    let mut text = [0; 32];
    let length = reader
        .read_legacy_string(&mut text, MessageStringLimit::Normal)
        .expect("Command text");
    assert_eq!(&text[..length], expected_text);
    assert_eq!(reader.read_bits(3).expect("Control EOF"), 3);
}
