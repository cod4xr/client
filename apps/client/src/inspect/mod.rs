// SPDX-License-Identifier: AGPL-3.0-or-later
mod arguments;
mod error;
mod file;
mod report;

use cod4x_observability::{LogConfig, LogRuntime};
use cod4xr_content::defaults::{DefaultSlot, DefaultsBudget, ImportedDefaults};
use std::{process::ExitCode, time::Instant};

pub(crate) use error::InspectError;

pub(crate) fn run() -> Result<ExitCode, InspectError> {
    let Some(options) = arguments::from_environment()? else {
        print!("{}", arguments::HELP);
        return Ok(ExitCode::SUCCESS);
    };
    let mut config = LogConfig::default();
    config.console_format = options.logs;
    let logs = LogRuntime::build(&config).map_err(InspectError::Logging)?;
    Ok(tracing::dispatcher::with_default(logs.dispatch(), || {
        tracing::info!(
            event = "content.inspect.started",
            project = "CoD4XR",
            "Inspecting installed client data"
        );
        let code = match execute(&options) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                tracing::warn!(event = "content.inspect.failed", error = %error, "Client data inspection failed");
                ExitCode::FAILURE
            }
        };
        tracing::info!(
            event = "content.inspect.stopped",
            dropped_logs = logs.dropped_records(),
            "Client data inspection stopped"
        );
        code
    }))
}

fn execute(options: &arguments::Options) -> Result<(), InspectError> {
    let started = Instant::now();
    let budget = DefaultsBudget {
        max_image_bytes: 8 * 1024 * 1024,
        max_sections: 64,
        max_records: 2442,
        max_text_bytes: 64 * 1024,
        max_value_bytes: 64 * 1024,
    };
    let input = file::read(&options.path, budget.max_image_bytes)?;
    let mut text = vec![0; budget.max_text_bytes];
    let mut slots = vec![DefaultSlot::default(); budget.max_records];
    let defaults = ImportedDefaults::import(&input, budget, &mut text, &mut slots)
        .map_err(InspectError::Import)?;
    tracing::info!(
        event = "content.inspect.completed",
        image_bytes = input.len(),
        defaults = defaults.record_count(),
        value_bytes = defaults.text_bytes(),
        elapsed_micros = started.elapsed().as_micros(),
        "Reviewed client defaults imported"
    );
    report::write(input.len(), &defaults)
}
