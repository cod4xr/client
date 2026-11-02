// SPDX-License-Identifier: AGPL-3.0-or-later
mod carriers;
mod direct;
mod mixed;
mod mixed_failures;

use super::{
    BodyReader, SnapshotError, SnapshotPlan, contexts, dirty, encoded, input, output, position,
    sample, word,
};
use cod4xr_client_core::player_state::{
    PredictionProvenance, PredictionSample,
    history::{PredictionHistory, PredictionHistoryError, PredictionSlot},
};

fn retain(history: &mut PredictionHistory<'_>, time: u32, origin: u32) {
    let original = sample(history.profile()).words();
    let words = cod4xr_client_core::player_state::PredictionWords::new(
        [origin, 0x7fc0_1234, 0x8000_0000],
        original.velocity(),
        original.view_angles(),
        original.bob_cycle(),
        original.movement_direction(),
    );
    history
        .prepare_insert(
            history.epoch(),
            PredictionSample::new(history.profile(), time, words),
        )
        .expect("Exact sample")
        .commit();
}
fn timed_body(distance: u8, command_time: Option<u32>) -> Vec<u8> {
    assert_eq!(
        cod4xr_client_core::player_scalars::word::ALL[0],
        word::session::COMMAND_TIME
    );
    encoded(|writer| {
        writer
            .write_i32(1234)
            .expect("Server time differs from input time");
        writer.write_u8(distance).expect("Delta distance");
        writer.write_u8(0).expect("Flags");
        writer.write_bits(0, 1).expect("Native prediction required");
        writer
            .write_bits(u32::from(command_time.is_some()), 8)
            .expect("Scalar count");
        if let Some(time) = command_time {
            writer
                .write_bits(3, 2)
                .expect("Changed absolute command time");
            writer
                .write_i32(time.cast_signed())
                .expect("Raw command time");
        }
        writer.write_bits(0, 13).expect("Unchanged arrays");
        writer.write_bits(0xffe, 12).expect("Entity terminator");
        writer.write_bits(0, 1).expect("Client terminator");
    })
}
