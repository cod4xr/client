// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete command budgets and private failures precede wire publication.
#![allow(
    clippy::expect_used,
    reason = "Finite caller fields exercise explicit valid/invalid profile boundaries."
)]
mod connect_support;
use cod4x_protocol::{InfoLimit, InfoString};
use cod4xr_client_core::connect::{
    ConnectPlanError, MAX_CONNECT_PACKET_BYTES, NoAuthConnectPlan, NoAuthConnectSettings,
};

fn plan(info: &[u8]) -> Result<NoAuthConnectPlan, ConnectPlanError> {
    NoAuthConnectPlan::new(
        connect_support::receipt(1234),
        NoAuthConnectSettings {
            protocol: 21,
            qport: 40_000,
            version: b"21.8",
            userinfo: InfoString::parse(info, InfoLimit::Normal).expect("Validated caller info"),
        },
    )
}

#[test]
fn every_short_output_prefix_is_unchanged_and_exact_capacity_succeeds() {
    let plan = plan(b"\\name\\PrivatePlayer").expect("Plan");
    for capacity in 0..plan.encoded_len() {
        let mut bytes = [0xa5; MAX_CONNECT_PACKET_BYTES];
        assert_eq!(
            plan.encode(&mut bytes[..capacity]),
            Err(ConnectPlanError::OutputCapacity)
        );
        assert_eq!(bytes, [0xa5; MAX_CONNECT_PACKET_BYTES]);
    }
    let mut exact = vec![0; plan.encoded_len()];
    assert_eq!(plan.encode(&mut exact), Ok(exact.len()));
}

#[test]
fn complete_legacy_command_boundary_includes_envelope_quotes_and_c_string_scratch() {
    let base = plan(b"\\padding\\x").expect("Base frame").encoded_len();
    let mut info = b"\\padding\\x".to_vec();
    info.resize(info.len() + MAX_CONNECT_PACKET_BYTES - base, b'x');
    let exact = plan(&info).expect("Exact command budget");
    assert_eq!(exact.encoded_len(), MAX_CONNECT_PACKET_BYTES);
    info.push(b'x');
    assert_eq!(
        plan(&info).expect_err("One extra wire byte"),
        ConnectPlanError::UserinfoEncoding
    );
}

#[test]
fn critical_case_aliases_and_duplicate_unknown_fields_reject_without_exposing_names() {
    for info in [
        b"\\PROTOCOL\\6".as_slice(),
        b"\\challenge\\1",
        b"\\QpOrT\\0",
        b"\\xver\\other",
    ] {
        assert_eq!(
            plan(info).expect_err("Typed fields cannot be overridden"),
            ConnectPlanError::ReservedField
        );
    }
    let error = plan(b"\\PrivateKey\\one\\PRIVATEKEY\\two").expect_err("Case aliases duplicate");
    assert_eq!(error, ConnectPlanError::DuplicateField);
    assert!(!format!("{error:?} {error}").contains("PrivateKey"));
}

#[test]
fn invalid_profile_values_and_version_delimiter_injection_reject() {
    let empty = InfoString::parse(b"", InfoLimit::Normal).expect("Info");
    for protocol in [-1, 0] {
        assert_eq!(
            NoAuthConnectPlan::new(
                connect_support::receipt(1),
                NoAuthConnectSettings {
                    protocol,
                    qport: 0,
                    version: b"21.8",
                    userinfo: empty,
                }
            )
            .expect_err("Invalid requested version"),
            ConnectPlanError::InvalidSettings
        );
    }
    for version in [
        b"".as_slice(),
        b"bad\\field",
        b"bad\"quote",
        b"bad\0suffix",
        b"bad\nline",
        b"bad;command",
    ] {
        let error = NoAuthConnectPlan::new(
            connect_support::receipt(1),
            NoAuthConnectSettings {
                protocol: 21,
                qport: 0,
                version,
                userinfo: empty,
            },
        )
        .expect_err("Version cannot inject extra command fields");
        assert!(matches!(
            error,
            ConnectPlanError::InvalidSettings | ConnectPlanError::UserinfoEncoding
        ));
        assert!(!format!("{error:?}").contains("bad"));
    }
}
