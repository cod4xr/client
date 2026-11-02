// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned no-auth request plans use shared native-compatible wire codecs.
#![allow(
    clippy::expect_used,
    reason = "Finite userinfo/receipt fixtures use explicit compatibility settings."
)]
use crate::connect_support;
use cod4x_protocol::connection::ConnectRequest;
use cod4x_protocol::{CommandLine, InfoLimit, InfoString};
use cod4xr_client_core::connect::{
    MAX_CONNECT_PACKET_BYTES, NoAuthConnectPlan, NoAuthConnectSettings,
};

#[test]
fn plan_owns_unknown_userinfo_bytes_and_exact_connect_framing_after_inputs_are_dropped() {
    let mut userinfo =
        b"\\name\\PrivatePlayer\\future\\raw\xff\\password\\PrivatePassword".to_vec();
    let settings = NoAuthConnectSettings {
        protocol: 21,
        qport: 40_000,
        version: b"21.8",
        userinfo: InfoString::parse(&userinfo, InfoLimit::Normal).expect("Caller fields"),
    };
    let receipt = connect_support::receipt(i32::MIN);
    let plan = NoAuthConnectPlan::new(receipt, settings).expect("Owned plan");
    let settings_debug = format!("{settings:?}");
    userinfo.fill(0);
    drop(userinfo);
    let mut output = [0xa5; MAX_CONNECT_PACKET_BYTES + 3];
    let length = plan.encode(&mut output).expect("Wire packet");
    assert_eq!(length, plan.encoded_len());
    assert_eq!(&output[..13], b"\xff\xff\xff\xffconnect \"");
    assert_eq!(output[length - 1], b'"');
    assert!(output[length..].iter().all(|&byte| byte == 0xa5));
    let request =
        ConnectRequest::parse(CommandLine::parse(&output[4..length]).expect("Complete command"))
            .expect("Shared native field decoder");
    assert_eq!(request.protocol(), 21);
    assert_eq!(request.qport(), 40_000);
    assert_eq!(request.challenge(), i32::MIN);
    assert_eq!(
        request.userinfo().get(b"name"),
        Some(b"PrivatePlayer".as_slice())
    );
    assert_eq!(
        request.userinfo().get(b"future"),
        Some(b"raw\xff".as_slice())
    );
    assert_eq!(request.userinfo().get(b"xver"), Some(b"21.8".as_slice()));
    assert_eq!(request.userinfo().get(b"qport"), Some(b"-25536".as_slice()));
    assert!(request.userinfo().get(b"pbguid").is_none());
    assert_eq!(plan.peer(), receipt.peer());
    for debug in [settings_debug, format!("{plan:?}")] {
        for private in [
            "PrivatePlayer",
            "PrivatePassword",
            "2147483648",
            "127.0.0.1",
            "raw",
        ] {
            assert!(!debug.contains(private));
        }
    }
}

#[test]
fn full_qport_and_cookie_domains_preserve_native_signed_short_spelling() {
    let empty = InfoString::parse(b"", InfoLimit::Normal).expect("Empty caller fields");
    for cookie in [i32::MIN, -1, 0, 1, i32::MAX] {
        for qport in [0, 1, 32767, 32768, u16::MAX] {
            let plan = NoAuthConnectPlan::new(
                connect_support::receipt(cookie),
                NoAuthConnectSettings {
                    protocol: 18,
                    qport,
                    version: b"explicit",
                    userinfo: empty,
                },
            )
            .expect("Signed domains");
            let mut bytes = [0; MAX_CONNECT_PACKET_BYTES];
            let length = plan.encode(&mut bytes).expect("Packet");
            let request =
                ConnectRequest::parse(CommandLine::parse(&bytes[4..length]).expect("Line"))
                    .expect("Fields");
            assert_eq!((request.qport(), request.challenge()), (qport, cookie));
            let signed = i16::from_le_bytes(qport.to_le_bytes()).to_string();
            assert_eq!(request.userinfo().get(b"qport"), Some(signed.as_bytes()));
        }
    }
}

#[test]
fn explicit_wire_protocol_is_never_inferred_from_the_challenge_marker() {
    let empty = InfoString::parse(b"", InfoLimit::Normal).expect("Empty info");
    for protocol in [6, 18, 21, i32::MAX] {
        let plan = NoAuthConnectPlan::new(
            connect_support::receipt(42),
            NoAuthConnectSettings {
                protocol,
                qport: 1,
                version: b"ExplicitVersion",
                userinfo: empty,
            },
        )
        .expect("Explicit syntactic profile, not admission");
        let mut bytes = [0; MAX_CONNECT_PACKET_BYTES];
        let length = plan.encode(&mut bytes).expect("Encode");
        let request =
            ConnectRequest::parse(CommandLine::parse(&bytes[4..length]).expect("Command"))
                .expect("Typed fields");
        assert_eq!(request.protocol(), protocol);
        assert_eq!(
            request.userinfo().get(b"xver"),
            Some(b"ExplicitVersion".as_slice())
        );
    }
}

#[test]
fn empty_caller_fields_follow_legacy_setter_deletion_without_changing_other_values() {
    let info = InfoString::parse(
        b"\\future\\kept\\deleted\\\\other\\value",
        InfoLimit::Normal,
    )
    .expect("Empty caller value");
    let plan = NoAuthConnectPlan::new(
        connect_support::receipt(1),
        NoAuthConnectSettings {
            protocol: 21,
            qport: 1,
            version: b"21.8",
            userinfo: info,
        },
    )
    .expect("Canonical setter behavior");
    let mut bytes = [0; MAX_CONNECT_PACKET_BYTES];
    let length = plan.encode(&mut bytes).expect("Packet");
    let request = ConnectRequest::parse(CommandLine::parse(&bytes[4..length]).expect("Line"))
        .expect("Fields");
    assert_eq!(request.userinfo().get(b"future"), Some(b"kept".as_slice()));
    assert_eq!(request.userinfo().get(b"other"), Some(b"value".as_slice()));
    assert!(request.userinfo().get(b"deleted").is_none());
}
