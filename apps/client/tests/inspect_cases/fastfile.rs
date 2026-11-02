// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Fixture, archive::run_mode, fs};

// Fixed Python zlib vector: 10000-byte synthetic body b'abcde'*2000,
// external size 777, nine block declarations 1..=9. No game assets.
const ZLIB: &[u8] = &[
    0x78, 0xda, 0xed, 0xc6, 0xb9, 0x0d, 0x83, 0x00, 0x00, 0x04, 0xc1, 0x33, 0x3f, 0x99, 0xbb, 0xa0,
    0x2e, 0x1e, 0xf7, 0xdf, 0x02, 0xd7, 0x86, 0xa5, 0x09, 0x76, 0xb4, 0xdf, 0x23, 0xd9, 0xc7, 0xe4,
    0x93, 0x64, 0x68, 0xdd, 0x4c, 0x6d, 0x6e, 0x4b, 0x5b, 0xdb, 0xd6, 0xf6, 0x76, 0x5e, 0xf7, 0xf3,
    0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe0, 0xcf,
    0x79, 0x01, 0xd8, 0xfe, 0x1c, 0x82,
];
fn container() -> Vec<u8> {
    let mut bytes = b"IWffu100".to_vec();
    bytes.extend_from_slice(&5_u32.to_le_bytes());
    bytes.extend_from_slice(ZLIB);
    bytes
}

#[test]
fn fastfile_report_separates_checked_container_from_uninterpreted_assets() {
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, container()).expect("Synthetic fastfile");
    for format in ["human", "json"] {
        let output = run_mode(&fixture, format, "--inspect-fastfile");
        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Metadata JSON");
        assert_eq!(report["phase"], "fastfile5_inspection");
        assert_eq!(report["encoded_bytes"], 82);
        assert_eq!(report["decoded_bytes"], 10044);
        assert_eq!(report["serialized_body_bytes"], 10000);
        assert_eq!(report["declared_external_bytes"], 777);
        assert_eq!(report["declared_total_block_bytes"], 45);
        assert_eq!(report["container_integrity"], "zlib_adler32_verified");
        assert_eq!(report["asset_interpretation"], "unverified");
        assert_eq!(report["gameplay_compatibility"], "unverified");
        let all = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!all.contains("Private"));
        assert!(!all.contains("abcde"));
    }
}

#[test]
fn checksum_trailing_bytes_and_wrong_profile_fail_with_no_partial_report() {
    for mutation in 0..3 {
        let mut bytes = container();
        match mutation {
            0 => {
                let end = bytes.len() - 1;
                bytes[end] ^= 1;
            }
            1 => bytes.push(0),
            _ => bytes[8] = 6,
        }
        let fixture = Fixture::new(false);
        fs::write(&fixture.0, bytes).expect("Invalid container");
        let output = run_mode(&fixture, "json", "--inspect-fastfile");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("JSON errors");
        assert!(!stderr.contains("Private"));
        assert!(!stderr.contains("abcde"));
        assert!(!stderr.contains("content.inspect.completed"));
        for line in stderr.lines() {
            let _: serde_json::Value = serde_json::from_str(line).expect("JSON log");
        }
    }
}
