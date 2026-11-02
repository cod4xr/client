// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{snapshot_support::with_snapshot, user_commands::zero};
use cod4xr_client_core::{
    BaselineProtocol,
    player_state::{PredictionSample, PredictionWords},
    user_commands::{
        UserCommandEncodeError,
        history::{InputCommandHistory, InputCommandSlot, InputHistoryError},
    },
};

fn prediction(profile: BaselineProtocol) -> PredictionSample {
    PredictionSample::new(
        profile,
        0,
        PredictionWords::new([0; 3], [0; 3], [0; 3], 0, 0),
    )
}
#[test]
fn original_input_world_binds_prediction_snapshot_and_stale_ids_without_time_guessing() {
    let profile = BaselineProtocol::Cod4x21;
    let mut cells = vec![InputCommandSlot::default(); 2];
    let mut owner = InputCommandHistory::new(7, profile, &mut cells).expect("Owner");
    let mut input = zero();
    input.server_time = i32::MIN;
    let id = owner
        .enqueue(input)
        .expect("Opaque wrapping native timestamp");
    let pending = owner.prepare_range(id, id).expect("Input range");
    assert_eq!(
        pending
            .movement(0, prediction(BaselineProtocol::Legacy6), None)
            .expect_err("Prediction binding"),
        InputHistoryError::PredictionProfile
    );
    for (source_profile, epoch) in [(BaselineProtocol::Legacy6, 7), (profile, 8)] {
        with_snapshot(source_profile, epoch, 7, |source| {
            assert_eq!(
                pending
                    .movement(0, prediction(profile), Some(source))
                    .expect_err("Snapshot world/profile binding"),
                InputHistoryError::SnapshotBinding
            );
        });
    }
    with_snapshot(profile, 7, 7, |source| {
        assert!(
            pending
                .movement(0, prediction(profile), Some(source))
                .is_ok()
        );
    });
    drop(pending);
    assert_eq!(owner.reset(7, profile), Err(InputHistoryError::EpochOrder));
    assert_eq!(owner.latest_id(), Some(id));
    owner
        .reset(8, BaselineProtocol::Legacy6)
        .expect("New world/profile");
    assert_eq!(
        owner
            .prepare_range(id, id)
            .expect_err("Stale range cannot alias"),
        InputHistoryError::Binding
    );
    assert_eq!(owner.retire_through(id), Err(InputHistoryError::Binding));
}
#[test]
fn invalid_inputs_and_full_storage_preserve_all_cells_and_private_debug() {
    assert_eq!(
        InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut [])
            .expect_err("No caller slots"),
        InputHistoryError::Storage
    );
    let mut cells = vec![InputCommandSlot::default(); 1];
    {
        let mut owner =
            InputCommandHistory::new(1_987_654_321, BaselineProtocol::Cod4x21, &mut cells)
                .expect("Owner");
        let mut invalid = zero();
        invalid.weapon = 128;
        assert_eq!(
            owner.enqueue(invalid),
            Err(InputHistoryError::Command(UserCommandEncodeError::Weapon))
        );
        assert!(owner.is_empty());
        let mut input = zero();
        input.server_time = 1_987_654_321;
        input.weapon = 119;
        let id = owner.enqueue(input).expect("Private native fields");
        assert_eq!(owner.enqueue(input), Err(InputHistoryError::Capacity));
        let prepared = owner.prepare_range(id, id).expect("Original owner lease");
        assert!(!format!("{prepared:?} {id:?}").contains("1987654321"));
        drop(prepared);
        assert!(!format!("{owner:?}").contains("1987654321"));
        assert_eq!(owner.command(id).expect("No overwritten input"), input);
    }
    assert!(!format!("{cells:?}").contains("119"));
}
