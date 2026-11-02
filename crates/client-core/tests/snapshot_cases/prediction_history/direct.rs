// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, PredictionHistory, PredictionHistoryError as Error, PredictionProvenance,
    PredictionSlot, SnapshotError, SnapshotPlan, contexts, dirty, input, output, position, retain,
    timed_body, word,
};

#[test]
fn actual_and_inherited_delta_command_times_select_exact_samples_without_retaining_owner_borrows() {
    for context in contexts() {
        for time in [0, 1, 0x8000_0000, u32::MAX] {
            let mut slots = vec![PredictionSlot::default(); 3];
            let mut history =
                PredictionHistory::new(7, context.profile(), &mut slots).expect("Owner");
            retain(&mut history, time.wrapping_sub(1), 111);
            retain(&mut history, time, 999);
            let (mut entities, mut clients) = dirty();
            let prior = {
                let bytes = timed_body(0, Some(time));
                let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
                SnapshotPlan::prepare(
                    &mut reader,
                    input(context, 5, None),
                    output(&mut entities, &mut clients),
                )
                .expect("Whole body")
                .commit_from_history(history.source())
                .expect("Actual decoded exact time")
            };
            assert_eq!(
                prior.player().scalars().word(word::session::COMMAND_TIME),
                time
            );
            assert_eq!(prior.player().scalars().word(word::motion::ORIGIN_X), 999);
            assert_eq!(
                prior.player().prediction(),
                PredictionProvenance::CommandTimeMatched
            );
            assert_eq!(prior.header().server_time(), 1234);
            // Correct that exact sample while the old cooked snapshot stays alive.
            let corrected = cod4xr_client_core::player_state::PredictionSample::new(
                context.profile(),
                time,
                cod4xr_client_core::player_state::PredictionWords::new(
                    [555, 0, 0],
                    [0; 3],
                    [0; 3],
                    0,
                    0,
                ),
            );
            history
                .prepare_replace(7, corrected)
                .expect("Replay update after decode")
                .commit();
            let bytes = timed_body(2, None);
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Delta body");
            let (mut next_entities, mut next_clients) = dirty();
            let next = SnapshotPlan::prepare(
                &mut reader,
                input(context, 7, Some(prior.as_source())),
                output(&mut next_entities, &mut next_clients),
            )
            .expect("Coherent delta")
            .commit_from_history(history.source())
            .expect("Inherited exact time");
            history
                .reset(8, context.profile())
                .expect("Borrow released");
            assert_eq!(
                next.player().scalars().word(word::session::COMMAND_TIME),
                time
            );
            assert_eq!(next.player().scalars().word(word::motion::ORIGIN_X), 555);
            assert_eq!(prior.player().scalars().word(word::motion::ORIGIN_X), 999);
            assert_eq!(
                next.player().prediction(),
                PredictionProvenance::CommandTimeMatched
            );
        }
    }
}

#[test]
fn wrong_world_profile_and_missing_exact_sample_preserve_reader_and_both_complete_outputs() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        for (epoch, profile, expected) in [
            (8, context.profile(), Error::Epoch),
            (7, contexts[1 - index].profile(), Error::Profile),
            (7, context.profile(), Error::Missing),
        ] {
            let mut slots = [PredictionSlot::default()];
            let mut history = PredictionHistory::new(epoch, profile, &mut slots).expect("Owner");
            retain(&mut history, 1233, 111);
            let bytes = timed_body(0, Some(1234));
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let before = position(&reader);
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let error = SnapshotPlan::prepare(
                &mut reader,
                input(context, 7, None),
                output(&mut entities, &mut clients),
            )
            .expect("Complete candidate")
            .commit_from_history(history.source())
            .expect_err("No substitution");
            assert_eq!(error, SnapshotError::PredictionHistory(expected));
            assert_eq!(position(&reader), before);
            assert_eq!((entities, clients), saved);
            assert_eq!(history.len(), 1);
        }
    }
}
