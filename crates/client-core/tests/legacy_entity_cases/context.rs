// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn map_center_preserves_native_extended_half_bias_then_truncation_and_hides_values() {
    for (center, expected) in [
        ([1000.75, -1000.75, 0.0], [1001, -1000, 0]),
        ([0.5, -0.5, -0.51], [1, 0, 0]),
        ([-1.5, -1.49, -1.51], [-1, 0, -1]),
        ([8_388_609.0, 8_388_608.0, -0.0], [8_388_609, 8_388_608, 0]),
        (
            [-2_147_483_648.0, 2_147_483_520.0, 0.0],
            [i32::MIN + 1, 2_147_483_520, 0],
        ),
        ([f32::from_bits(0x3eff_ffff), 0.5, -0.5], [0, 1, 0]),
    ] {
        let context = LegacyMapCenter::new(center).expect("Native bias");
        assert_eq!(context.biases(), expected);
        assert_eq!(format!("{context:?}"), "LegacyMapCenter { .. }");
    }
    for value in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        f32::MAX,
        f32::MIN,
        2_147_483_648.0,
    ] {
        for axis in 0..3 {
            let mut center = [0.0; 3];
            center[axis] = value;
            assert_eq!(
                LegacyMapCenter::new(center),
                Err(EntityBaselineError::InvalidMapCenter)
            );
        }
    }
}

#[test]
fn legacy_context_updates_keep_index_order_and_reset_keeps_the_explicit_current_center() {
    assert!(matches!(
        EntityBaselineDecoder::new(BaselineProtocol::Legacy6),
        Err(EntityBaselineError::UnsupportedProfile)
    ));
    let coordinates = [Coordinate::Full(0xffff); 3];
    let first = fixture(17, coordinates, false, false);
    let next = fixture(18, coordinates, false, false);
    let mut stream = decoder([0.0; 3]);
    assert_eq!(stream.profile(), BaselineProtocol::Legacy6);
    let mut reader = BodyReader::try_new(&first, first.len()).expect("First");
    assert_eq!(
        stream
            .parse(&mut reader)
            .expect("First state")
            .word(W::PositionX),
        (-1.0_f32).to_bits()
    );
    stream
        .set_legacy_map_center(LegacyMapCenter::new([1000.0; 3]).expect("Replacement"))
        .expect("Same profile");
    let mut duplicate = BodyReader::try_new(&first, first.len()).expect("Duplicate");
    let before = position(&duplicate);
    assert!(stream.parse(&mut duplicate).is_err());
    assert_eq!(position(&duplicate), before);
    let mut reader = BodyReader::try_new(&next, next.len()).expect("Next");
    assert_eq!(
        stream
            .parse(&mut reader)
            .expect("New center")
            .word(W::PositionX),
        1999.0_f32.to_bits()
    );
    stream.reset();
    assert_eq!(
        stream
            .parse(&mut duplicate)
            .expect("Explicit reset, same center")
            .word(W::PositionX),
        1999.0_f32.to_bits()
    );
}

#[test]
fn modern_owners_reject_center_reconfiguration_without_changing_their_coordinate_profile() {
    let mut stream = EntityBaselineDecoder::new(BaselineProtocol::Cod4x21).expect("Modern");
    let mut reader = BodyReader::try_new(&[0xa1, 7], 2).expect("First no-delta physical record");
    assert_eq!(stream.parse(&mut reader).expect("First").number(), 0);
    assert_eq!(
        stream.set_legacy_map_center(LegacyMapCenter::new([1000.0; 3]).expect("Context")),
        Err(EntityBaselineError::UnsupportedProfile)
    );
    assert_eq!(stream.profile(), BaselineProtocol::Cod4x21);
    // The original next-number shortcut still means entity 1; context rejection did
    // not silently reset ordering or select a different dialect.
    let mut reader = BodyReader::try_new(&[1], 1).expect("Next no-delta record");
    assert_eq!(
        stream.parse(&mut reader).expect("Original stream").number(),
        1
    );
}
