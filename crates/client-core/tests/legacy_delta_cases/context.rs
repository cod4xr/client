// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn explicit_context_replacement_preserves_order_and_reset_preserves_the_new_legacy_center() {
    assert!(matches!(
        EntityDeltaDecoder::new(BaselineProtocol::Legacy6),
        Err(EntityDeltaError::UnsupportedProfile)
    ));
    let first = fixture(17, [Coordinate::Full(0xffff); 3], false, false);
    let next = fixture(18, [Coordinate::Full(0xffff); 3], false, false);
    let mut stream = decoder([0.0; 3]);
    assert_eq!(stream.profile(), BaselineProtocol::Legacy6);
    let mut reader = BodyReader::try_new(&first, first.len()).expect("First");
    let _ = stream
        .prepare(&mut reader)
        .expect("Index")
        .decode(None, 0)
        .expect("First state");
    stream
        .set_legacy_map_center(LegacyMapCenter::new([1000.0; 3]).expect("New context"))
        .expect("Original legacy owner");
    let mut duplicate = BodyReader::try_new(&first, first.len()).expect("Duplicate");
    let before = position(&duplicate);
    assert!(stream.prepare(&mut duplicate).is_err());
    assert_eq!(position(&duplicate), before);
    let mut reader = BodyReader::try_new(&next, next.len()).expect("Next");
    assert_eq!(
        present(
            stream
                .prepare(&mut reader)
                .expect("Next index")
                .decode(None, 0)
                .expect("New center")
        )
        .word(W::PositionX),
        1999.0_f32.to_bits()
    );
    stream.reset();
    assert_eq!(
        present(
            stream
                .prepare(&mut duplicate)
                .expect("Explicit reset")
                .decode(None, 0)
                .expect("Context retained")
        )
        .word(W::PositionX),
        1999.0_f32.to_bits()
    );
}

#[test]
fn modern_context_rejection_leaves_the_original_profile_and_index_stream_unchanged() {
    let mut stream = EntityDeltaDecoder::new(BaselineProtocol::Cod4x21).expect("Modern");
    let mut reader = BodyReader::try_new(&[0xa1, 7], 2).expect("First");
    let _ = stream
        .prepare(&mut reader)
        .expect("Index")
        .decode(None, 0)
        .expect("State");
    assert_eq!(
        stream.set_legacy_map_center(LegacyMapCenter::new([1000.0; 3]).expect("Context")),
        Err(EntityDeltaError::UnsupportedProfile)
    );
    assert_eq!(stream.profile(), BaselineProtocol::Cod4x21);
    let mut next = BodyReader::try_new(&[1], 1).expect("Next shortcut");
    assert_eq!(
        present(
            stream
                .prepare(&mut next)
                .expect("Original next index")
                .decode(None, 0)
                .expect("State")
        )
        .number(),
        1
    );
}
