// SPDX-License-Identifier: AGPL-3.0-or-later
//! Connect acknowledgements and messages are data without path/command authority.
#![allow(
    clippy::expect_used,
    reason = "Independent complete reply fixtures have explicit byte budgets."
)]
use cod4xr_client_core::connect::{
    ConnectReply, ConnectReplyError, ConnectReplyKind, MAX_CONNECT_REPLY_BYTES,
};

#[test]
fn bare_and_labeled_acknowledgements_preserve_bytes_without_executing_or_resolving_them() {
    for (packet, label) in [
        (
            b"\xff\xff\xff\xffconnectResponse".as_slice(),
            b"".as_slice(),
        ),
        (
            b"\xff\xff\xff\xffCONNECTRESPONSE mods/PrivateMod",
            b"mods/PrivateMod",
        ),
        (
            b"\xff\xff\xff\xffconnectResponse \"../PrivateMod\"",
            b"../PrivateMod",
        ),
        (b"\xff\xff\xff\xffconnectResponse raw\xff\0", b"raw\xff"),
    ] {
        let reply = ConnectReply::parse(packet).expect("Wire acknowledgement");
        assert_eq!(reply.kind(), ConnectReplyKind::Accepted);
        assert_eq!(reply.payload(), label);
        assert!(!format!("{reply:?}").contains("PrivateMod"));
    }
}

#[test]
fn refusal_and_notice_payloads_remain_opaque_even_with_native_markup_and_newlines() {
    let text = b"{ OOBErrorParser protocolmismatch private 21 }\nPrivateReason\xff";
    for (command, kind) in [
        (b"error\n".as_slice(), ConnectReplyKind::Refused),
        (b"print\n", ConnectReplyKind::Notice),
    ] {
        let mut packet = vec![0xff; 4];
        packet.extend_from_slice(command);
        packet.extend_from_slice(text);
        let reply = ConnectReply::parse(&packet).expect("Opaque text");
        assert_eq!(reply.kind(), kind);
        assert_eq!(reply.payload(), text);
        assert!(!format!("{reply:?}").contains("PrivateReason"));
        assert!(!format!("{reply:?}").contains("OOBErrorParser"));
    }
}

#[test]
fn extra_fields_and_hidden_trailing_bytes_reject_instead_of_becoming_commands() {
    for packet in [
        b"\xff\xff\xff\xffconnectResponse one two".as_slice(),
        b"\xff\xff\xff\xffconnectResponse\nignored",
        b"\xff\xff\xff\xffconnectResponse\0ignored",
        b"\xff\xff\xff\xfferror inline text",
        b"\xff\xff\xff\xfferror\nreason\0ignored",
        b"\xff\xff\xff\xffprint\ntext\0\0",
    ] {
        assert!(ConnectReply::parse(packet).is_err());
    }
}

#[test]
fn named_error_and_print_destination_limits_include_optional_c_string_termination() {
    for (header, maximum) in [(b"error\n".as_slice(), 1023), (b"print\n", 8191)] {
        let mut packet = vec![0xff; 4];
        packet.extend_from_slice(header);
        packet.resize(packet.len() + maximum, b'x');
        assert_eq!(
            ConnectReply::parse(&packet)
                .expect("Exact text budget")
                .payload()
                .len(),
            maximum
        );
        packet.push(0);
        assert!(ConnectReply::parse(&packet).is_ok());
        packet.pop();
        packet.push(b'x');
        assert_eq!(
            ConnectReply::parse(&packet).expect_err("One extra text byte"),
            ConnectReplyError::Capacity
        );
    }
    assert_eq!(
        ConnectReply::parse(&vec![b'x'; MAX_CONNECT_REPLY_BYTES + 1]).expect_err("Envelope bound"),
        ConnectReplyError::Capacity
    );
}

#[test]
fn malformed_and_unrelated_packets_never_become_admission() {
    for packet in [
        b"".as_slice(),
        b"\xff\xff\xff",
        b"\xfe\xff\xff\xffconnectResponse",
        b"\xff\xff\xff\xffchallengeResponse 1 2 0 xproto 18",
        b"\xff\xff\xff\xffdisconnect",
    ] {
        assert!(ConnectReply::parse(packet).is_err());
    }
    for packet in [
        b"\xff\xff\xff\xfferror\n".as_slice(),
        b"\xff\xff\xff\xffprint\n",
    ] {
        assert_ne!(
            ConnectReply::parse(packet).expect("Empty message").kind(),
            ConnectReplyKind::Accepted
        );
    }
}
