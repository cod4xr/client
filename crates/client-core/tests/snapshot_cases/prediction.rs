// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, Change, PlayerStateError, PredictionSample, SnapshotError, SnapshotPlan, body,
    contexts, dirty, input, output, position, sample, values, word,
};

#[test]
fn prediction_resolution_precedes_every_final_entity_and_client_write() {
    let contexts = contexts();
    for (index, &context) in contexts.iter().enumerate() {
        let bytes = body(
            0,
            false,
            &[Change::Set(1, 0, 13)],
            &[Change::Set(63, 0, 511)],
        );
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let (mut entities, mut clients) = dirty();
        let saved = (entities.clone(), clients.clone());
        let before = position(&reader);
        for case in 0..3 {
            let prepared = SnapshotPlan::prepare(
                &mut reader,
                input(context, 1, None),
                output(&mut entities, &mut clients),
            )
            .expect("Whole snapshot");
            let request = prepared.prediction_request().expect("Request");
            assert_eq!(request.command_time(), 0);
            assert_eq!(prepared.header().profile(), context.profile());
            assert_eq!(prepared.message().epoch(), 7);
            let failure = match case {
                0 => prepared
                    .commit_transmitted()
                    .expect_err("Unresolved prediction"),
                1 => prepared
                    .commit_with_prediction(&sample(contexts[1 - index].profile()))
                    .expect_err("Wrong profile"),
                _ => prepared
                    .commit_with_prediction(&PredictionSample::new(
                        context.profile(),
                        1,
                        sample(context.profile()).words(),
                    ))
                    .expect_err("Wrong time"),
            };
            let expected = match case {
                0 => PlayerStateError::PredictionRequired,
                1 => PlayerStateError::PredictionProfile,
                _ => PlayerStateError::PredictionTime,
            };
            assert_eq!(failure, SnapshotError::Player(expected));
            assert_eq!(position(&reader), before);
            assert_eq!((&entities, &clients), (&saved.0, &saved.1));
        }
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 1, None),
            output(&mut entities, &mut clients),
        )
        .expect("Retry")
        .commit_with_prediction(&sample(context.profile()))
        .expect("Matched prediction");
        assert_eq!(values(&next), (vec![(1, 13)], vec![(63, 511)]));
        assert_eq!(next.player().scalars().word(word::motion::ORIGIN_X), 1);
        assert_eq!(
            next.player().scalars().word(word::motion::DIRECTION),
            u32::MAX
        );
    }
}
#[test]
fn explicit_prior_prediction_fallback_preserves_coherent_whole_snapshot_reference() {
    for context in contexts() {
        let bytes = body(
            0,
            false,
            &[Change::Set(1, 0, 13)],
            &[Change::Set(63, 0, 511)],
        );
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Prior body");
        let (mut old_entities, mut old_clients) = dirty();
        let old = SnapshotPlan::prepare(
            &mut reader,
            input(context, 10, None),
            output(&mut old_entities, &mut old_clients),
        )
        .expect("Prior")
        .commit_with_prediction(&sample(context.profile()))
        .expect("Prior sample");
        let bytes = body(1, false, &[Change::Keep(1)], &[Change::Keep(63)]);
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Delta body");
        let (mut entities, mut clients) = dirty();
        let next = SnapshotPlan::prepare(
            &mut reader,
            input(context, 11, Some(old.as_source())),
            output(&mut entities, &mut clients),
        )
        .expect("Delta")
        .commit_native_prior_fallback()
        .expect("Explicit fallback");
        assert_eq!(next.player().scalars(), old.player().scalars());
        assert_eq!(values(&next), values(&old));
    }
}
#[test]
fn transmitted_mode_rejects_sample_and_fallback_without_touching_either_output() {
    for context in contexts() {
        for fallback in [false, true] {
            let bytes = body(
                0,
                true,
                &[Change::Set(1, 0, 13)],
                &[Change::Set(63, 0, 511)],
            );
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let (mut entities, mut clients) = dirty();
            let saved = (entities.clone(), clients.clone());
            let prepared = SnapshotPlan::prepare(
                &mut reader,
                input(context, 1, None),
                output(&mut entities, &mut clients),
            )
            .expect("Snapshot");
            let error = if fallback {
                prepared
                    .commit_native_prior_fallback()
                    .expect_err("Unexpected fallback")
            } else {
                prepared
                    .commit_with_prediction(&sample(context.profile()))
                    .expect_err("Unexpected sample")
            };
            assert_eq!(
                error,
                SnapshotError::Player(PlayerStateError::UnexpectedPrediction)
            );
            assert_eq!(reader.byte_position(), 0);
            assert_eq!((entities, clients), saved);
        }
    }
}
