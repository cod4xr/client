// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded installed-image process failures and separated metadata-only logging.
#![allow(
    clippy::expect_used,
    reason = "Finite process and ephemeral input fixtures."
)]
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
#[path = "inspect_cases/archive.rs"]
mod archive;
#[path = "inspect_cases/fastfile.rs"]
mod fastfile;
#[path = "inspect_cases/images.rs"]
mod images;
#[path = "inspect_cases/verification.rs"]
mod verification;
struct Fixture(PathBuf);
impl Fixture {
    fn new(oversized: bool) -> Self {
        let path = std::env::temp_dir().join(format!(
            "cod4xr-PrivateImageFixture-{}-{}.bin",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
            .expect("Exclusive fixture");
        file.write_all(b"PrivateDefaultFixture%\x92")
            .expect("Unknown bytes");
        if oversized {
            file.set_len(8 * 1024 * 1024 + 1)
                .expect("Over-budget extent");
        }
        Self(path)
    }
    fn run(&self, format: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_cod4xr-client-inspect"))
            .arg(&self.0)
            .args(["--log-format", format])
            .output()
            .expect("Inspector process")
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn help_and_invalid_options_do_not_open_or_echo_image_inputs() {
    let help = Command::new(env!("CARGO_BIN_EXE_cod4xr-client-inspect"))
        .arg("--help")
        .output()
        .expect("Help");
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("PATH_TO_IW3MP_EXE"));
    assert!(help.stderr.is_empty());
    let invalid = Command::new(env!("CARGO_BIN_EXE_cod4xr-client-inspect"))
        .args(["PrivatePath", "--log-format", "PrivateFormat"])
        .output()
        .expect("Bad arguments");
    assert!(!invalid.status.success());
    assert!(invalid.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&invalid.stderr).contains("Private"));
}

#[test]
fn unknown_images_and_file_budgets_emit_json_events_without_paths_or_game_values() {
    for oversized in [false, true] {
        let fixture = Fixture::new(oversized);
        let output = fixture.run("json");
        assert!(!output.status.success());
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 logs");
        assert!(!stderr.contains("Private"));
        let records: Vec<serde_json::Value> = stderr
            .lines()
            .map(|line| serde_json::from_str(line).expect("JSON event"))
            .collect();
        assert!(
            records
                .iter()
                .any(|record| record["fields"]["event"] == "content.inspect.failed")
        );
        assert!(
            records
                .iter()
                .any(|record| record["fields"]["event"] == "content.inspect.stopped")
        );
        assert!(stderr.contains(if oversized {
            "byte budget exceeded"
        } else {
            "no reviewed data profile"
        }));
    }
}

#[test]
fn human_development_logs_keep_failure_readable_and_metadata_separate() {
    let fixture = Fixture::new(false);
    let output = fixture.run("human");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("Human logs");
    assert!(stderr.contains("Client data inspection failed"));
    assert!(!stderr.contains("Private"));
    assert!(
        !stderr
            .lines()
            .any(|line| line.trim_start().starts_with('{'))
    );
}
