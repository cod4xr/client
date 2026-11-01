// SPDX-License-Identifier: AGPL-3.0-or-later
//! Public command-history behavior under byte, sequence and retained-key pressure.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit storage and sequence fixtures."
)]
#[path = "command_cases/acknowledgement.rs"]
mod acknowledgement;
#[path = "command_cases/queue.rs"]
mod queue;

use cod4xr_client_core::commands::LegacyClientCommands;

#[derive(Debug, Eq, PartialEq)]
struct Snapshot {
    latest: u32,
    transmitted: u32,
    acknowledged: u32,
    commands: Vec<(u32, Vec<u8>)>,
}

fn snapshot(history: &LegacyClientCommands<'_>) -> Snapshot {
    Snapshot {
        latest: history.latest_sequence(),
        transmitted: history.transmitted_sequence(),
        acknowledged: history.acknowledged_sequence(),
        commands: (history.acknowledged_sequence()..=history.latest_sequence())
            .map(|sequence| {
                (
                    sequence,
                    history
                        .command(sequence)
                        .expect("Retained prefix")
                        .as_bytes()
                        .to_vec(),
                )
            })
            .collect(),
    }
}
