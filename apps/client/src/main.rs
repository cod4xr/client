// SPDX-License-Identifier: AGPL-3.0-or-later
//! `CoD4XR` development client. Discovery only; no gameplay connection is claimed.

mod app;
mod arguments;
mod error;
mod query;
mod report;

use std::process::ExitCode;

fn main() -> ExitCode {
    match app::run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("cod4xr-client: {error}");
            ExitCode::FAILURE
        }
    }
}
