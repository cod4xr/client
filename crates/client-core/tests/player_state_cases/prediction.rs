// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, LivePlayerState, PlayerStateError, PredictionProvenance, PredictionWords, contexts,
    decode, encoded, mixed, position, sample, word,
};

fn assert_prediction(state: &LivePlayerState, words: PredictionWords) {
    let scalars = state.scalars();
    for (keys, values) in [
        (
            [
                word::motion::ORIGIN_X,
                word::motion::ORIGIN_Y,
                word::motion::ORIGIN_Z,
            ],
            words.origin(),
        ),
        (
            [
                word::motion::VELOCITY_X,
                word::motion::VELOCITY_Y,
                word::motion::VELOCITY_Z,
            ],
            words.velocity(),
        ),
        (
            [
                word::view::ANGLE_X,
                word::view::ANGLE_Y,
                word::view::ANGLE_Z,
            ],
            words.view_angles(),
        ),
    ] {
        for (key, value) in keys.into_iter().zip(values) {
            assert_eq!(scalars.word(key), value);
        }
    }
    assert_eq!(scalars.word(word::motion::BOB_CYCLE), words.bob_cycle());
    assert_eq!(
        scalars.word(word::motion::DIRECTION),
        words.movement_direction()
    );
}
#[test]
fn matching_prediction_replaces_exact_eleven_words_and_preserves_all_other_sections() {
    for context in contexts() {
        for time in [0, 1, u32::MAX] {
            let bytes = encoded(|writer| mixed(writer, false, 0, time));
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let prepared =
                LivePlayerState::prepare(&mut reader, context, 123, None).expect("Whole state");
            let request = prepared
                .prediction_request()
                .expect("Native lookup request");
            assert_eq!(request.profile(), context.profile());
            assert_eq!(request.command_time(), time);
            let sample = sample(context.profile(), time);
            let saved = sample;
            let state = prepared
                .commit_with_prediction(&sample)
                .expect("Matched sample");
            assert_eq!(sample, saved);
            assert_prediction(&state, sample.words());
            assert_eq!(state.prediction(), PredictionProvenance::CommandTimeMatched);
            assert_eq!(state.scalars().word(word::session::COMMAND_TIME), time);
            assert_eq!(state.scalars().word(word::weapons::TIME), 0xffff_fff9);
            assert_eq!(state.stats().get(0), Some(123));
            assert_eq!(state.ammunition().stored(0), Some(7));
            assert_eq!(state.models().get(127), Some(127 ^ 0xa5));
        }
    }
}
#[test]
fn bad_prediction_profile_and_time_cancel_all_sections_and_allow_valid_retry() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        let bytes = encoded(|writer| mixed(writer, false, 0, u32::MAX));
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        let prepared =
            LivePlayerState::prepare(&mut reader, context, 0, None).expect("Whole state");
        assert_eq!(
            prepared.commit_with_prediction(&sample(contexts[1 - index].profile(), u32::MAX)),
            Err(PlayerStateError::PredictionProfile)
        );
        assert_eq!(position(&reader), before);
        let prepared = LivePlayerState::prepare(&mut reader, context, 0, None).expect("Retry");
        assert_eq!(
            prepared.commit_with_prediction(&sample(context.profile(), 0)),
            Err(PlayerStateError::PredictionTime)
        );
        assert_eq!(position(&reader), before);
        LivePlayerState::prepare(&mut reader, context, 0, None)
            .expect("Retry")
            .commit_with_prediction(&sample(context.profile(), u32::MAX))
            .expect("Valid retry");
    }
}
#[test]
fn each_wire_mode_requires_its_own_explicit_commit_choice() {
    for context in contexts() {
        for transmitted in [false, true] {
            let bytes = [u8::from(transmitted), 0, 0];
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let before = position(&reader);
            if transmitted {
                let prepared =
                    LivePlayerState::prepare(&mut reader, context, 0, None).expect("State");
                assert_eq!(
                    prepared.commit_with_prediction(&sample(context.profile(), 0)),
                    Err(PlayerStateError::UnexpectedPrediction)
                );
                assert_eq!(position(&reader), before);
                let prepared =
                    LivePlayerState::prepare(&mut reader, context, 0, None).expect("Retry");
                assert_eq!(
                    prepared.commit_native_prior_fallback(),
                    Err(PlayerStateError::UnexpectedPrediction)
                );
                assert_eq!(position(&reader), before);
                LivePlayerState::prepare(&mut reader, context, 0, None)
                    .expect("Retry")
                    .commit_transmitted()
                    .expect("Correct mode");
            } else {
                let prepared =
                    LivePlayerState::prepare(&mut reader, context, 0, None).expect("State");
                assert_eq!(
                    prepared.commit_transmitted(),
                    Err(PlayerStateError::PredictionRequired)
                );
                assert_eq!(position(&reader), before);
                let prepared =
                    LivePlayerState::prepare(&mut reader, context, 0, None).expect("Retry");
                let request = prepared.prediction_request().expect("Request");
                assert_eq!(request.command_time(), 0);
                let state = prepared
                    .commit_native_prior_fallback()
                    .expect("Explicit fallback");
                assert_prediction(&state, PredictionWords::new([0; 3], [0; 3], [0; 3], 0, 0));
            }
        }
    }
}
#[test]
fn prior_fallback_overwrites_wire_prediction_words_but_keeps_new_time_and_arrays() {
    for context in contexts() {
        let bytes = encoded(|writer| {
            mixed(
                writer,
                false,
                sample(context.profile(), 0).words().origin()[0],
                700,
            );
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let prepared = {
            let mut prior_reader = BodyReader::try_new(&[0, 0, 0], 3).expect("Prior body");
            let prior = LivePlayerState::prepare(&mut prior_reader, context, 0, None)
                .expect("Prior")
                .commit_with_prediction(&sample(context.profile(), 0))
                .expect("Prior sample");
            LivePlayerState::prepare(&mut reader, context, 0, Some(&prior)).expect("Whole state")
        };
        let state = prepared
            .commit_native_prior_fallback()
            .expect("Explicit prior fallback");
        assert_prediction(&state, sample(context.profile(), 0).words());
        assert_eq!(
            state.prediction(),
            PredictionProvenance::ExplicitPriorFallback
        );
        assert_eq!(state.scalars().word(word::session::COMMAND_TIME), 700);
        assert_eq!(state.scalars().word(word::weapons::TIME), 0xffff_fff9);
        assert_eq!(state.stats().get(0), Some(123));
        assert_eq!(state.models().get(0), Some(0xa5));
    }
}
#[test]
fn transmitted_mode_retains_prediction_words_omitted_from_its_scalar_prefix() {
    for context in contexts() {
        let mut reader = BodyReader::try_new(&[0, 0, 0], 3).expect("Body");
        let prior = LivePlayerState::prepare(&mut reader, context, 0, None)
            .expect("State")
            .commit_with_prediction(&sample(context.profile(), 0))
            .expect("Sample");
        let next = decode(context, &[1, 0, 0], Some(&prior));
        assert_prediction(&next, sample(context.profile(), 0).words());
        assert_eq!(next.prediction(), PredictionProvenance::Transmitted);
    }
}
