// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, SystemInfo, SystemInfoBudget, SystemInfoError};
use cod4x_protocol::InfoError;

#[test]
fn only_the_critical_identifier_rejects_duplicates_and_missing_is_explicit() {
    for bytes in [
        &b"\\sv_serverid\\1\\SV_SERVERID\\1"[..],
        b"\\sv_serverid\\invalid\\sv_serverid\\2",
    ] {
        assert_eq!(
            SystemInfo::parse(bytes, BUDGET).err(),
            Some(SystemInfoError::DuplicateServerId)
        );
    }
    for bytes in [&b""[..], b"\\other\\1", b"\\sv_serveridx\\1"] {
        assert_eq!(
            SystemInfo::parse(bytes, BUDGET).err(),
            Some(SystemInfoError::MissingServerId)
        );
    }
}

#[test]
fn shared_validation_checks_the_whole_value_before_metadata_can_escape() {
    for suffix in [&b"\\dangling"[..], b"\\", b"\\\\value"] {
        let mut bytes = b"\\sv_serverid\\1".to_vec();
        bytes.extend_from_slice(suffix);
        assert!(matches!(
            SystemInfo::parse(&bytes, BUDGET),
            Err(SystemInfoError::Info(_))
        ));
    }
    for byte in (0..=31).chain([127, b'"', b';']) {
        let mut bytes = b"\\sv_serverid\\1\\unknown\\".to_vec();
        bytes.push(byte);
        assert_eq!(
            SystemInfo::parse(&bytes, BUDGET).err(),
            Some(SystemInfoError::Info(InfoError::InvalidCharacter))
        );
    }
}

#[test]
fn host_limits_and_big_info_wire_bounds_are_independent_and_exact() {
    let mut bytes = b"\\sv_serverid\\1\\payload\\".to_vec();
    bytes.resize(8191, b'x');
    let exact = SystemInfoBudget {
        max_encoded_bytes: bytes.len(),
        max_fields: 2,
    };
    let info = SystemInfo::parse(&bytes, exact).expect("Big info above normal-info limit");
    assert_eq!(info.field_count(), 2);
    assert_eq!(info.field(b"payload").expect("Payload").len(), 8191 - 23);
    for max_encoded_bytes in [0, 1023, 8190] {
        assert_eq!(
            SystemInfo::parse(
                &bytes,
                SystemInfoBudget {
                    max_encoded_bytes,
                    ..exact
                }
            )
            .err(),
            Some(SystemInfoError::ByteBudget)
        );
    }
    for max_fields in [0, 1] {
        assert_eq!(
            SystemInfo::parse(
                &bytes,
                SystemInfoBudget {
                    max_fields,
                    ..exact
                }
            )
            .err(),
            Some(SystemInfoError::FieldBudget)
        );
    }
    bytes.push(b'x');
    assert_eq!(
        SystemInfo::parse(
            &bytes,
            SystemInfoBudget {
                max_encoded_bytes: usize::MAX,
                max_fields: usize::MAX,
            }
        )
        .err(),
        Some(SystemInfoError::Info(InfoError::TooLong))
    );
}

#[test]
fn errors_are_fixed_categories_without_echoing_remote_contents() {
    let error = SystemInfo::parse(b"\\sv_serverid\\sensitive-id", BUDGET).expect_err("Invalid ID");
    assert!(!format!("{error:?}: {error}").contains("sensitive"));
    let error = SystemInfo::parse(b"\\sv_serverid\\1\\sensitive-key\\bad;value", BUDGET)
        .expect_err("Invalid shared syntax");
    assert!(std::error::Error::source(&error).is_some());
    assert!(!format!("{error:?}: {error}").contains("sensitive"));
}
