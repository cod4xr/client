// SPDX-License-Identifier: AGPL-3.0-or-later
//! Non-Unicode OS arguments reject without panicking or initiating network work.
#![allow(
    clippy::expect_used,
    reason = "The platform fixture starts one bounded failing process."
)]
use std::{ffi::OsString, process::Command};

#[cfg(windows)]
fn invalid_argument() -> OsString {
    use std::os::windows::ffi::OsStringExt;
    OsString::from_wide(&[0xd800])
}

#[cfg(unix)]
fn invalid_argument() -> OsString {
    use std::os::unix::ffi::OsStringExt;
    OsString::from_vec(vec![0xff])
}

#[test]
fn non_unicode_os_input_returns_a_fixed_error_before_logging_or_network_work() {
    let output = Command::new(env!("CARGO_BIN_EXE_cod4xr-client"))
        .arg("--help")
        .arg(invalid_argument())
        .output()
        .expect("Invalid OS argument process");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("arguments must be valid Unicode"));
    assert!(!error.contains("panicked"));
    assert!(!error.contains("client.started"));
}
