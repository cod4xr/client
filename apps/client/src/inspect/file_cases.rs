// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Owned ephemeral file fixtures.")]
use super::read;
use crate::inspect::InspectError;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(bytes: &[u8]) -> Self {
        let path = std::env::temp_dir().join(format!(
            "cod4xr-private-file-{}-{}.tmp",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .expect("Exclusive temporary fixture");
        file.write_all(bytes).expect("Fixture bytes");
        Self(path)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

#[test]
fn regular_files_admit_exact_budgets_and_empty_input_without_unbounded_reads() {
    let file = Fixture::new(b"Private%Fixture");
    assert_eq!(
        read(&file.0, 15).expect("Exact file budget"),
        b"Private%Fixture"
    );
    for maximum in 0..15 {
        assert!(matches!(
            read(&file.0, maximum),
            Err(InspectError::FileBudget)
        ));
    }
    let empty = Fixture::new(&[]);
    assert!(read(&empty.0, 0).expect("Empty regular file").is_empty());
}

#[test]
fn missing_paths_and_nonregular_inputs_fail_without_retaining_private_path_data() {
    let file = Fixture::new(b"x");
    fs::remove_file(&file.0).expect("Missing fixture");
    let error = read(&file.0, 1).expect_err("No such file");
    assert!(!format!("{error:?} {error}").contains("cod4xr-private-file"));
    assert!(read(&std::env::temp_dir(), 64).is_err());
}
