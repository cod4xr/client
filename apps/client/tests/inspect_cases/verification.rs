// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    Fixture,
    archive::{run_mode, valid_bytes},
    fs,
};

#[test]
fn verification_reports_only_complete_checked_aggregate_metadata() {
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, valid_bytes()).expect("Independent ZIP");
    for format in ["human", "json"] {
        let output = run_mode(&fixture, format, "--verify-archive");
        assert!(output.status.success());
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("Report");
        assert_eq!(report["phase"], "zip32_archive_verification");
        assert_eq!(report["payload_integrity"], "crc32_verified");
        assert_eq!(report["verified_entries"], 1);
        assert_eq!(report["verified_uncompressed_bytes"], 5);
        assert_eq!(report["gameplay_compatibility"], "unverified");
        let text = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for secret in ["Private", "a.txt", "hello"] {
            assert!(!text.contains(secret));
        }
    }
}

#[test]
fn false_payload_crc_and_over_budget_declarations_publish_no_success_report() {
    let fixture = Fixture::new(false);
    for over_budget in [false, true] {
        let mut bytes = valid_bytes();
        if over_budget {
            let declared = 64_u32 * 1024 * 1024 + 1;
            bytes[22..26].copy_from_slice(&declared.to_le_bytes());
            bytes[66..70].copy_from_slice(&declared.to_le_bytes());
        } else {
            bytes[14..18].fill(0);
            bytes[58..62].fill(0);
        }
        fs::write(&fixture.0, &bytes).expect("Structurally valid false metadata");
        // Cataloging alone must not report these unverified declarations as integrity.
        let catalog = run_mode(&fixture, "json", "--archive");
        assert!(catalog.status.success());
        let output = run_mode(&fixture, "json", "--verify-archive");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("JSON logs");
        assert!(!stderr.contains("Private"));
        assert!(!stderr.contains("content.inspect.completed"));
        assert!(stderr.contains(if over_budget {
            "output budget exhausted"
        } else {
            "checksum disagrees"
        }));
    }
}
