// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native system metadata is validated without granting remote configuration authority.
#![allow(clippy::expect_used, reason = "Finite reviewed metadata fixtures.")]

#[path = "system_info_cases/snapshot.rs"]
mod snapshot;
#[path = "system_info_cases/validation.rs"]
mod validation;

use cod4xr_client_core::system_info::{SystemInfo, SystemInfoBudget, SystemInfoError};

const BUDGET: SystemInfoBudget = SystemInfoBudget {
    max_encoded_bytes: 8191,
    max_fields: 64,
};

#[test]
fn signed_and_unsigned_decimal_forms_preserve_every_server_id_bit() {
    for (text, expected) in [
        ("0", 0),
        ("+0", 0),
        ("-0", 0),
        ("00000000001", 1),
        ("2147483647", 0x7fff_ffff),
        ("2147483648", 0x8000_0000),
        ("4294967295", u32::MAX),
        ("+4294967295", u32::MAX),
        ("-1", u32::MAX),
        ("-2147483648", 0x8000_0000),
    ] {
        let bytes = format!("\\SV_SERVERID\\{text}");
        let parsed = SystemInfo::parse(bytes.as_bytes(), BUDGET).expect("Complete decimal ID");
        assert_eq!(parsed.server_id(), expected);
        assert_eq!(parsed.field_count(), 1);
    }
}

#[test]
fn identifiers_must_be_complete_checked_decimal_values() {
    for text in [
        "",
        "+",
        "-",
        "--1",
        "+-1",
        " 1",
        "1 ",
        "1x",
        "0x01",
        "1.0",
        "4294967296",
        "-2147483649",
        "99999999999999999999999999999",
        "∞",
        "١",
    ] {
        let bytes = format!("\\sv_serverid\\{text}");
        assert_eq!(
            SystemInfo::parse(bytes.as_bytes(), BUDGET).err(),
            Some(SystemInfoError::InvalidServerId),
            "{text}",
        );
    }
    assert_eq!(
        SystemInfo::parse(b"\\sv_serverid\\\x92", BUDGET).err(),
        Some(SystemInfoError::InvalidServerId)
    );
    let mut zeros = b"\\sv_serverid\\".to_vec();
    zeros.extend_from_slice(&vec![b'0'; 7000]);
    zeros.push(b'1');
    assert_eq!(
        SystemInfo::parse(&zeros, BUDGET)
            .expect("Bounded leading zeros")
            .server_id(),
        1
    );
}

#[test]
fn unknown_fields_remain_raw_inert_data_and_debug_never_prints_them() {
    let info = SystemInfo::parse(
        b"sv_serverid\\4294967295\\fs_game\\private/path\\opaque\\\x92%secret\\OPAQUE\\later",
        BUDGET,
    )
    .expect("Unknown fields and non-UTF8 data");
    assert_eq!(info.field(b"FS_GAME"), Some(&b"private/path"[..]));
    assert_eq!(info.field(b"opaque"), Some(&b"\x92%secret"[..]));
    assert_eq!(info.field(b"missing"), None);
    assert_eq!(info.field_count(), 4);
    let debug = format!("{info:?}");
    assert!(debug.contains("fields: 4"));
    for secret in [
        "private",
        "secret",
        "opaque",
        "later",
        "4294967295",
        "sv_serverid",
    ] {
        assert!(!debug.contains(secret));
    }
}
