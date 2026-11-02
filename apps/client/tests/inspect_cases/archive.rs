// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Command, Fixture, OpenOptions, Output, fs};

fn run(fixture: &Fixture, format: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cod4xr-client-inspect"))
        .arg("--archive")
        .arg(&fixture.0)
        .args(["--log-format", format])
        .output()
        .expect("Archive inspector")
}

fn valid_bytes() -> Vec<u8> {
    // Independently generated Python stdlib ZIP containing a.txt / hello, plus a comment.
    let hex = concat!(
        "504b03041400000008000000215086a61036070000000500000005000000612e747874cb48cdc9c90700",
        "504b010214001400000008000000215086a610360700000005000000050000000000000000000000800100000000612e747874",
        "504b05060000000001000100330000002a0000000000",
    );
    let mut bytes: Vec<_> = hex
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16).expect("Byte"))
        .collect();
    let comment = b"PrivateArchiveComment";
    let offset = bytes.len() - 2;
    bytes[offset..].copy_from_slice(&u16::try_from(comment.len()).expect("Comment").to_le_bytes());
    bytes.extend_from_slice(comment);
    bytes
}

#[test]
fn archive_success_reports_counts_without_paths_names_comments_or_integrity_claims() {
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, valid_bytes()).expect("Synthetic archive");
    for format in ["json", "human"] {
        let output = run(&fixture, format);
        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Metadata JSON");
        assert_eq!(report["phase"], "zip32_archive_inspection");
        assert_eq!(report["entries"], 1);
        assert_eq!(report["deflate_entries"], 1);
        assert_eq!(report["declared_uncompressed_bytes"], 5);
        assert_eq!(report["payload_integrity"], "unverified");
        let text = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        for secret in ["Private", "a.txt", "hello"] {
            assert!(!text.contains(secret));
        }
        let stderr = String::from_utf8(output.stderr).expect("Logs");
        if format == "json" {
            let events: Vec<serde_json::Value> = stderr
                .lines()
                .map(|line| serde_json::from_str(line).expect("JSON event"))
                .collect();
            assert!(
                events
                    .iter()
                    .any(|event| event["fields"]["event"] == "content.inspect.completed")
            );
        } else {
            assert!(stderr.contains("Classic archive metadata cataloged"));
        }
    }
}

#[test]
fn invalid_archives_publish_no_report_or_private_input() {
    let fixture = Fixture::new(false);
    let output = run(&fixture, "json");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("Logs");
    assert!(!stderr.contains("Private"));
    assert!(stderr.contains("terminal end record missing"));
    for line in stderr.lines() {
        let _: serde_json::Value = serde_json::from_str(line).expect("JSON");
    }
}

#[test]
fn oversized_archive_files_reject_on_metadata_before_reading_payload() {
    let fixture = Fixture::new(false);
    OpenOptions::new()
        .write(true)
        .open(&fixture.0)
        .expect("Fixture")
        .set_len(512 * 1024 * 1024 + 1)
        .expect("Over-budget sparse extent");
    let output = run(&fixture, "json");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("Logs");
    assert!(stderr.contains("file byte budget exceeded"));
    assert!(!stderr.contains("Private"));
}
