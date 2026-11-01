// SPDX-License-Identifier: AGPL-3.0-or-later
//! Independent finite discovery wire vectors and malformed-input boundaries.
#![allow(
    clippy::expect_used,
    reason = "Tests construct explicit finite wire vectors."
)]
use cod4xr_client_core::{
    DiscoveryError, INFO_REQUEST_BYTES, InfoResponse, MAX_INFO_DATAGRAM_BYTES, QueryChallenge,
};

fn challenge() -> QueryChallenge {
    QueryChallenge::from_entropy([0; 16])
}

fn response(extra: &[u8]) -> Vec<u8> {
    let mut bytes =
        b"\xff\xff\xff\xffinfoResponse\n\\challenge\\00000000000000000000000000000000".to_vec();
    bytes.extend_from_slice(extra);
    bytes
}

#[test]
fn request_has_exact_native_oob_shape_and_hex_entropy() {
    let token = QueryChallenge::from_entropy(std::array::from_fn(|index| {
        u8::try_from(index).expect("16 bytes")
    }));
    let mut output = [0xaa; INFO_REQUEST_BYTES + 3];
    assert_eq!(token.write_request(&mut output), Ok(INFO_REQUEST_BYTES));
    assert_eq!(
        &output[..INFO_REQUEST_BYTES],
        b"\xff\xff\xff\xffgetinfo 000102030405060708090a0b0c0d0e0f"
    );
    assert_eq!(&output[INFO_REQUEST_BYTES..], &[0xaa; 3]);
    assert!(!format!("{token:?}").contains("00010203"));
}

#[test]
fn every_short_request_storage_prefix_is_unchanged() {
    for length in 0..INFO_REQUEST_BYTES {
        let mut storage = [0xaa; INFO_REQUEST_BYTES];
        assert_eq!(
            challenge().write_request(&mut storage[..length]),
            Err(DiscoveryError::OutputCapacity)
        );
        assert_eq!(storage, [0xaa; INFO_REQUEST_BYTES]);
    }
}

#[test]
fn response_preserves_remote_bytes_unknown_fields_and_legacy_advertisement() {
    let bytes = response(b"\\hostname\\^1Server\xff\\future\\value\\protocol\\6");
    let info = InfoResponse::parse(&bytes, challenge()).expect("Native server advertisement");
    assert_eq!(info.field(b"HOSTNAME"), Some(b"^1Server\xff".as_slice()));
    assert_eq!(info.field(b"future"), Some(b"value".as_slice()));
    assert_eq!(info.advertised_protocol(), 6);
    assert!(info.info().as_bytes().ends_with(b"\\protocol\\6"));
    let debug = format!("{info:?}");
    assert!(!debug.contains("Server"));
    assert!(!debug.contains("00000000"));
}

#[test]
fn every_required_field_truncation_and_embedded_terminator_rejects() {
    let bytes = response(b"\\protocol\\6");
    for length in 0..bytes.len() {
        assert!(
            InfoResponse::parse(&bytes[..length], challenge()).is_err(),
            "length {length}"
        );
    }
    let mut terminated = bytes.clone();
    terminated.push(0);
    assert!(InfoResponse::parse(&terminated, challenge()).is_ok());
    terminated.push(0);
    assert!(InfoResponse::parse(&terminated, challenge()).is_err());
    let mut appended = bytes;
    appended.extend_from_slice(b"\0unparsed");
    assert!(InfoResponse::parse(&appended, challenge()).is_err());
}

#[test]
fn stale_missing_and_ambiguous_case_insensitive_challenges_reject() {
    let bytes = response(b"\\protocol\\6");
    assert_eq!(
        InfoResponse::parse(&bytes, QueryChallenge::from_entropy([1; 16]))
            .expect_err("Wrong token"),
        DiscoveryError::ChallengeMismatch
    );
    for key in [b"challenge".as_slice(), b"CHALLENGE"] {
        let mut duplicate = response(b"\\protocol\\6\\");
        duplicate.extend_from_slice(key);
        duplicate.extend_from_slice(b"\\00000000000000000000000000000000");
        assert_eq!(
            InfoResponse::parse(&duplicate, challenge()).expect_err("Ambiguous token"),
            DiscoveryError::AmbiguousChallenge
        );
    }
    assert_eq!(
        InfoResponse::parse(b"\xff\xff\xff\xffinfoResponse\n\\protocol\\6", challenge())
            .expect_err("Missing token"),
        DiscoveryError::ChallengeMismatch
    );
}

#[test]
fn all_single_byte_protocols_and_decimal_overflow_are_checked_without_panics() {
    for byte in u8::MIN..=u8::MAX {
        let mut bytes = response(b"\\protocol\\");
        bytes.push(byte);
        if byte.is_ascii_digit() {
            assert_eq!(
                InfoResponse::parse(&bytes, challenge())
                    .expect("Digit")
                    .advertised_protocol(),
                u32::from(byte - b'0')
            );
        } else {
            assert!(
                InfoResponse::parse(&bytes, challenge()).is_err(),
                "byte {byte}"
            );
        }
    }
    for value in [b"".as_slice(), b"-1", b"+6", b"6 ", b"4294967296"] {
        let mut bytes = response(b"\\protocol\\");
        bytes.extend_from_slice(value);
        assert!(InfoResponse::parse(&bytes, challenge()).is_err());
    }
    let bytes = response(b"\\protocol\\4294967295");
    assert_eq!(
        InfoResponse::parse(&bytes, challenge())
            .expect("u32 domain")
            .advertised_protocol(),
        u32::MAX
    );
}

#[test]
fn unrelated_envelopes_commands_and_over_limit_datagrams_reject() {
    for bytes in [
        b"\xff\xff\xff\xffstatusResponse\n".as_slice(),
        b"\xfe\xff\xff\xffinfoResponse\n",
        b"\x01\0\0\0infoResponse\n",
    ] {
        assert!(InfoResponse::parse(bytes, challenge()).is_err());
    }
    let oversized = vec![b'a'; MAX_INFO_DATAGRAM_BYTES + 1];
    assert_eq!(
        InfoResponse::parse(&oversized, challenge()).expect_err("Profile bound"),
        DiscoveryError::InvalidEnvelope
    );
    let lowercase = response(b"\\protocol\\6");
    let mut case = lowercase;
    case[4..16].copy_from_slice(b"INFORESPONSE");
    assert!(InfoResponse::parse(&case, challenge()).is_ok());
}

#[test]
fn complete_normal_info_limit_is_admitted_but_one_more_byte_rejects() {
    let mut bytes = response(b"\\protocol\\6\\padding\\");
    bytes.resize(4 + 13 + 1023, b'x');
    assert!(InfoResponse::parse(&bytes, challenge()).is_ok());
    bytes.push(b'x');
    assert!(InfoResponse::parse(&bytes, challenge()).is_err());
}
