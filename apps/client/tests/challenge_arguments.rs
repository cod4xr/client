// SPDX-License-Identifier: AGPL-3.0-or-later
//! Retry settings have explicit modes, ranges and duplicate rejection.
#![allow(
    clippy::expect_used,
    reason = "Finite invalid argument processes never begin network work."
)]
use std::process::Command;

#[test]
fn retry_options_are_bounded_unique_and_only_accepted_for_the_challenge_command() {
    for options in [
        vec!["--retry-ms", "0"],
        vec!["--retry-ms", "30001"],
        vec!["--max-attempts", "0"],
        vec!["--max-attempts", "17"],
        vec!["--retry-ms", "1", "--retry-ms", "2"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_cod4xr-client"))
            .args(["probe-challenge", "127.0.0.1:28960"])
            .args(options)
            .output()
            .expect("Invalid probe options");
        assert!(!output.status.success());
        assert!(!String::from_utf8_lossy(&output.stderr).contains("client.started"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_cod4xr-client"))
        .args(["query", "127.0.0.1:28960", "--retry-ms", "20"])
        .output()
        .expect("Query option rejection");
    assert!(!output.status.success());
    assert!(!String::from_utf8_lossy(&output.stderr).contains("client.started"));
}
