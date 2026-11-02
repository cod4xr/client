// SPDX-License-Identifier: AGPL-3.0-or-later
//! Read-only installed-client data inspection; reports metadata, never game values.

#[path = "../inspect/mod.rs"]
mod inspect;

use std::process::ExitCode;

fn main() -> ExitCode {
    match inspect::run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("cod4xr-client-inspect: {error}");
            ExitCode::FAILURE
        }
    }
}
