// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, BaselineProtocol, Error, Finals, SERVICES, carrier, encoded, finish, parse};
use cod4xr_client_core::{gamestate::GameStateError, service_controls::ServiceControlError};

#[test]
fn every_truncated_carrier_and_late_failure_preserves_all_six_final_output_areas() {
    let source = carrier(7);
    // Empty payloads reject in ServerPayloadDecoder before this carrier is reachable.
    for length in 1..source.len() {
        let mut finals = Finals::new();
        let saved = finals.clone();
        parse(
            &source[..length],
            BaselineProtocol::Cod4x21,
            SERVICES,
            BUDGET,
            &mut finals,
        )
        .expect_err("Complete inner and outer framing required");
        assert_eq!(finals, saved, "Prefix length {length}");
    }
    for (source, expected) in [
        (vec![7], Error::MissingGameState),
        (
            carrier(0xfe),
            Error::Controls(ServiceControlError::UnsupportedService(0xfe)),
        ),
        (
            carrier(1),
            Error::Controls(ServiceControlError::UnsupportedService(1)),
        ),
        (
            encoded(|writer| {
                writer.write_u8(1).expect("Gamestate");
                writer.write_i32(0).expect("Sequence");
                finish(writer, 64);
                writer.write_u8(7).expect("Outer EOF");
            }),
            Error::GameState(GameStateError::ClientIndex),
        ),
    ] {
        let mut finals = Finals::new();
        let saved = finals.clone();
        assert_eq!(
            parse(
                &source,
                BaselineProtocol::Cod4x21,
                SERVICES,
                BUDGET,
                &mut finals
            )
            .expect_err("Atomic framing rejection"),
            expected
        );
        assert_eq!(finals, saved);
    }
}

#[test]
fn actual_legacy_payload_rejects_before_interpreting_modern_gamestate_or_writing_outputs() {
    let mut finals = Finals::new();
    let saved = finals.clone();
    assert_eq!(
        parse(
            &carrier(7),
            BaselineProtocol::Legacy6,
            SERVICES,
            BUDGET,
            &mut finals
        )
        .expect_err("No implicit profile expansion"),
        Error::Profile
    );
    assert_eq!(finals, saved);
}
