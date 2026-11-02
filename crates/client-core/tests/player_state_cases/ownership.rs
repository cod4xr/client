// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, EntityDecodeContext, LegacyMapCenter, LivePlayerState, contexts, decode, encoded,
    mixed, position, sample, word,
};
use std::error::Error;

#[test]
fn committed_projection_outlives_wire_prior_and_prediction_sample() {
    for context in contexts() {
        let state = {
            let bytes = encoded(|writer| mixed(writer, false, 0, 123));
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
            let prepared = {
                let prior = decode(context, &[1, 0, 0], None);
                LivePlayerState::prepare(&mut reader, context, 0, Some(&prior)).expect("State")
            };
            let sample = sample(context.profile(), 123);
            prepared.commit_with_prediction(&sample).expect("Sample")
        };
        assert_eq!(state.scalars().word(word::session::COMMAND_TIME), 123);
        assert_eq!(state.scalars().word(word::motion::ORIGIN_X), 0x7fc0_1234);
        assert_eq!(state.models().get(127), Some(127 ^ 0xa5));
    }
}
#[test]
fn explicit_same_profile_center_replacement_retains_owned_words() {
    let old = EntityDecodeContext::Legacy6(LegacyMapCenter::new([1.0, 2.0, 3.0]).expect("Center"));
    let new = EntityDecodeContext::Legacy6(
        LegacyMapCenter::new([1000.0, -1.0, 0.0]).expect("New center"),
    );
    let prior = decode(old, &encoded(|writer| mixed(writer, true, 0, 100)), None);
    let next = decode(new, &[1, 0, 0], Some(&prior));
    assert_eq!(next.context(), new);
    assert_eq!(next.hud().context(), new);
    assert_eq!(next.scalars(), prior.scalars());
    assert_eq!(next.hud().archival(), prior.hud().archival());
    assert_eq!(prior.context(), old);
}
#[test]
fn debug_and_error_chains_expose_metadata_without_peer_or_prediction_words() {
    for context in contexts() {
        let sample = sample(context.profile(), 0x7654_3210);
        let bytes = encoded(|writer| mixed(writer, false, 0, sample.command_time()));
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let before = position(&reader);
        let prepared = LivePlayerState::prepare(&mut reader, context, 0, None).expect("Prepared");
        let request = prepared.prediction_request().expect("Request");
        let pending = format!("{prepared:?} {request:?} {sample:?} {:?}", sample.words());
        let state = prepared.commit_with_prediction(&sample).expect("Sample");
        let debug = format!("{pending} {state:?}");
        for secret in [
            "1985229328",
            "2143294004",
            "4294967168",
            "76543210",
            "7fc01234",
        ] {
            assert!(!debug.contains(secret), "{debug}");
        }
        assert_ne!(position(&reader), before);
        let mut truncated = BodyReader::try_new(&[1, 0], 2).expect("Short body");
        let error =
            LivePlayerState::prepare(&mut truncated, context, 0, None).expect_err("Failure");
        assert!(error.source().is_some());
        assert!(!format!("{error:?} {error}").contains("7fc01234"));
    }
}
