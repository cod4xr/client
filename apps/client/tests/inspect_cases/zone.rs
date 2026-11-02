// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Fixture, archive::run_mode, fs};
#[path = "../../../../crates/content/tests/fastfile_cases/fixture.rs"]
mod fastfile_fixture;

fn body() -> Vec<u8> {
    let mut bytes = Vec::new();
    // Three strings: inline, null, empty. Two asset records follow immediately.
    for value in [3, u32::MAX, 2, u32::MAX, u32::MAX, 0, u32::MAX] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes.extend_from_slice(b"Private\0\0");
    for value in [31, u32::MAX, 24, 0x1000_0101] {
        bytes.extend_from_slice(&value.to_le_bytes());
    }
    bytes
}
fn container(body: &[u8], virtual_bytes: u32) -> Vec<u8> {
    let mut blocks = [0; 9];
    blocks[4] = virtual_bytes;
    let decoded = fastfile_fixture::inner(
        body,
        u32::try_from(body.len()).expect("Synthetic root"),
        0,
        blocks,
    );
    fastfile_fixture::stored(&decoded)
}

#[test]
fn root_directory_report_keeps_strings_and_skipped_link_words_private() {
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, container(&body(), 40)).expect("Synthetic root container");
    for format in ["human", "json"] {
        let output = run_mode(&fixture, format, "--inspect-zone-directory");
        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("JSON metadata");
        assert_eq!(report["phase"], "zone5_directory_inspection");
        assert_eq!(report["container_integrity"], "zlib_adler32_verified");
        assert_eq!(report["asset_interpretation"], "root_directory_only");
        assert_eq!(report["child_asset_interpretation"], "unverified");
        let directory = &report["zone_directory"];
        assert_eq!(directory["script_strings"], 3);
        assert_eq!(directory["null_strings"], 1);
        assert_eq!(directory["asset_records"], 2);
        assert_eq!(directory["inline_string_bytes"], 9);
        assert_eq!(directory["body_prefix_bytes"], 53);
        assert_eq!(directory["virtual_prefix_bytes"], 40);
        assert_eq!(directory["asset_type_counts"][24], 1);
        assert_eq!(directory["asset_type_counts"][31], 1);
        assert_eq!(directory["asset_link_counts"]["following"], 1);
        assert_eq!(directory["asset_link_counts"]["offset"], 1);
        assert_eq!(
            directory["script_bytes_sha256"],
            serde_json::json!([
                151, 189, 253, 5, 83, 179, 81, 136, 73, 93, 14, 40, 230, 212, 49, 212, 81, 15, 87,
                5, 241, 119, 81, 2, 182, 222, 247, 146, 122, 57, 154, 144
            ])
        );
        let all = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!all.contains("Private"));
        assert!(!all.contains("10000101"));
    }
}

#[test]
fn zlib_valid_but_invalid_root_catalog_has_no_partial_report_or_completed_event() {
    for mutation in 0..5 {
        let mut bytes = body();
        let mut extent = 40;
        match mutation {
            0 => bytes[4..8].copy_from_slice(&0_u32.to_le_bytes()),
            1 => bytes[37..41].copy_from_slice(&33_u32.to_le_bytes()),
            2 => bytes[16..20].copy_from_slice(&1_u32.to_le_bytes()),
            3 => extent = 39,
            _ => bytes[..4].copy_from_slice(&u32::MAX.to_le_bytes()),
        }
        let fixture = Fixture::new(false);
        fs::write(&fixture.0, container(&bytes, extent))
            .expect("Invalid root with valid zlib integrity");
        let output = run_mode(&fixture, "json", "--inspect-zone-directory");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let logs = String::from_utf8(output.stderr).expect("JSON logs");
        assert!(!logs.contains("Private"));
        assert!(!logs.contains("content.inspect.completed"));
        for line in logs.lines() {
            let _: serde_json::Value = serde_json::from_str(line).expect("JSON record");
        }
    }
}
