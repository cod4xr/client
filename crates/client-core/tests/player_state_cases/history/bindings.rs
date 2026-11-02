// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PredictionHistory, PredictionHistoryError as Error, PredictionSlot, insert, prediction,
    profiles,
};
use crate::player_state::{
    BodyReader, LivePlayerState, PredictionProvenance, contexts, encoded, mixed, position, word,
};

#[test]
fn decoded_exact_request_resolves_only_current_world_profile_and_time_before_commit() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        for time in [0, 1, 0x8000_0000, u32::MAX] {
            let bytes = encoded(|writer| mixed(writer, false, 0, time));
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let before = position(&reader);
            let mut slots = vec![PredictionSlot::default(); 3];
            let mut history =
                PredictionHistory::new(7, context.profile(), &mut slots).expect("Owner");
            // A nearby sample is never silently selected, including at wraparound.
            insert(&mut history, time.wrapping_sub(1), 111);
            let prepared =
                LivePlayerState::prepare(&mut reader, context, 123, None).expect("Whole state");
            let request = prepared.prediction_request().expect("Actual request");
            assert_eq!(history.resolve(8, request), Err(Error::Epoch));
            assert_eq!(history.resolve(7, request), Err(Error::Missing));
            let mut foreign_slots = [PredictionSlot::default()];
            let foreign =
                PredictionHistory::new(7, contexts[1 - index].profile(), &mut foreign_slots)
                    .expect("Other profile");
            assert_eq!(foreign.resolve(7, request), Err(Error::Profile));
            drop(prepared);
            assert_eq!(position(&reader), before);
            insert(&mut history, time, 999);
            let prepared =
                LivePlayerState::prepare(&mut reader, context, 123, None).expect("Retry");
            let state = prepared
                .commit_with_prediction(history.resolve(7, request).expect("Exact current sample"))
                .expect("Commit");
            assert_eq!(state.prediction(), PredictionProvenance::CommandTimeMatched);
            assert_eq!(state.scalars().word(word::motion::ORIGIN_X), 999);
            assert_eq!(state.scalars().word(word::session::COMMAND_TIME), time);
            assert_eq!(state.stats().get(0), Some(123));
        }
    }
}

#[test]
fn stale_producers_failed_reset_and_reconstruction_cannot_reuse_old_world_samples() {
    let profiles = profiles();
    let mut slots = vec![PredictionSlot::default(); 3];
    {
        let mut history = PredictionHistory::new(7, profiles[0], &mut slots).expect("Owner");
        insert(&mut history, 123, 456);
        assert_eq!(history.reset(7, profiles[1]), Err(Error::EpochOrder));
        assert_eq!(history.reset(6, profiles[1]), Err(Error::EpochOrder));
        assert_eq!(
            history.sample_at(7, 123),
            Ok(&prediction(profiles[0], 123, 456))
        );
        assert_eq!(
            history
                .prepare_insert(8, prediction(profiles[0], 124, 0))
                .expect_err("World"),
            Error::Epoch
        );
        assert_eq!(
            history
                .prepare_insert(7, prediction(profiles[1], 124, 0))
                .expect_err("Profile"),
            Error::Profile
        );
        history.reset(8, profiles[1]).expect("New world");
        assert_eq!(history.epoch(), 8);
        assert_eq!(history.profile(), profiles[1]);
        assert!(history.is_empty());
        assert_eq!(history.sample_at(8, 123), Err(Error::Missing));
        assert_eq!(history.sample_at(7, 123), Err(Error::Epoch));
        assert_eq!(history.remove(7, 123), Err(Error::Epoch));
        assert_eq!(
            history
                .prepare_replace(7, prediction(profiles[1], 123, 0))
                .expect_err("Stale producer"),
            Error::Epoch
        );
        insert(&mut history, 123, 789);
    }
    let history = PredictionHistory::new(9, profiles[0], &mut slots).expect("Fresh owner");
    assert!(history.is_empty());
    assert_eq!(history.sample_at(9, 123), Err(Error::Missing));
    let mut empty = [];
    assert_eq!(
        PredictionHistory::new(1, profiles[0], &mut empty).expect_err("Storage"),
        Error::Storage
    );
}

#[test]
fn debug_and_errors_do_not_publish_times_labels_or_raw_player_words() {
    let profile = profiles()[0];
    let mut slots = [PredictionSlot::default()];
    {
        let mut history = PredictionHistory::new(987_654_321, profile, &mut slots).expect("Owner");
        let prepared = history
            .prepare_insert(987_654_321, prediction(profile, 123_456_789, 555_555_555))
            .expect("Preflight");
        assert_eq!(
            format!("{prepared:?}"),
            "PreparedPredictionUpdate { replacement: false, .. }"
        );
        prepared.commit();
        for text in [
            format!("{history:?}"),
            format!("{}", Error::Missing),
            format!("{:?}", Error::Missing),
        ] {
            for private in ["987654321", "123456789", "555555555", "7fc01234"] {
                assert!(!text.contains(private));
            }
        }
    }
    assert_eq!(format!("{:?}", slots[0]), "PredictionSlot { .. }");
}
