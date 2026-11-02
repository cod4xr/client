// SPDX-License-Identifier: AGPL-3.0-or-later
mod bindings;
mod model;
mod retention;

use super::{BaselineProtocol, PredictionSample, PredictionWords};
use cod4xr_client_core::player_state::history::{
    PredictionHistory, PredictionHistoryError, PredictionSlot,
};

fn profiles() -> [BaselineProtocol; 2] {
    [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21]
}
fn prediction(profile: BaselineProtocol, time: u32, value: u32) -> PredictionSample {
    PredictionSample::new(
        profile,
        time,
        PredictionWords::new(
            [value, 0x7fc0_1234, 0x8000_0000],
            [1, 2, 3],
            [4, 5, 6],
            u32::MAX,
            0xffff_ff80,
        ),
    )
}
fn insert(history: &mut PredictionHistory<'_>, time: u32, value: u32) {
    let sample = prediction(history.profile(), time, value);
    history
        .prepare_insert(history.epoch(), sample)
        .expect("New exact sample")
        .commit();
}
