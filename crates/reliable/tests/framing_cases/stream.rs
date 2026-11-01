// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::framing::{FrameAssembler, FrameError, FrameProgress};

use crate::NATIVE_SHAPED;

#[test]
fn every_three_chunk_partition_retains_partial_headers_and_hides_incomplete_body_storage() {
    for first in 0..=NATIVE_SHAPED.len() {
        for second in first..=NATIVE_SHAPED.len() {
            let mut storage = [0xa5; 32];
            let mut stream = FrameAssembler::new(&mut storage).expect("Host budget");
            assert!(stream.frame().is_none());
            assert!(!stream.consume_ready());
            let mut total = 0;
            for chunk in [
                &NATIVE_SHAPED[..first],
                &NATIVE_SHAPED[first..second],
                &NATIVE_SHAPED[second..],
            ] {
                let progress = stream.push(chunk).expect("Chunk boundary");
                assert_eq!(progress.consumed, chunk.len());
                total += progress.consumed;
                assert_eq!(progress.complete, total == NATIVE_SHAPED.len());
                assert_eq!(stream.frame().is_some(), progress.complete);
                assert_eq!(stream.buffered_bytes(), total);
            }
            assert_eq!(
                stream.frame().expect("Complete").body(),
                &NATIVE_SHAPED[4..]
            );
            assert!(stream.consume_ready());
            assert_eq!(stream.buffered_bytes(), 0);
            assert!(stream.frame().is_none());
            assert!(!stream.is_failed());
        }
    }
}

#[test]
fn retained_frames_consume_no_new_input_and_concatenation_advances_exactly_one_frame_per_call() {
    let source = [0, 0, 0, 0, 1, 0, 0, 0, 42, 3, 0, 0, 0, 0xfe, 0, 0xff];
    let mut storage = [0xa5; 8];
    let mut stream = FrameAssembler::new(&mut storage).expect("Budget");
    let mut offset = 0;
    for expected in [b"".as_slice(), &[42], &[0xfe, 0, 0xff]] {
        let progress = stream.push(&source[offset..]).expect("One frame");
        assert_eq!(
            progress,
            FrameProgress {
                consumed: 4 + expected.len(),
                complete: true
            }
        );
        offset += progress.consumed;
        assert_eq!(stream.frame().expect("Body").body(), expected);
        assert_eq!(
            stream.push(&source[offset..]),
            Ok(FrameProgress {
                consumed: 0,
                complete: true
            })
        );
        assert_eq!(stream.frame().expect("Still retained").body(), expected);
        assert!(stream.consume_ready());
    }
    assert_eq!(offset, source.len());
    assert_eq!(stream.push(b""), Ok(FrameProgress::default()));
}

#[test]
fn invalid_length_poisoning_occurs_only_after_a_complete_prefix_without_copying_claimed_body() {
    for declared in [-1_i32, i32::MIN, 9, i32::MAX] {
        for split in 1..4 {
            let mut storage = [0xa5; 8];
            let mut stream = FrameAssembler::new(&mut storage).expect("Budget");
            let prefix = declared.to_le_bytes();
            assert_eq!(
                stream.push(&prefix[..split]),
                Ok(FrameProgress {
                    consumed: split,
                    complete: false
                })
            );
            let mut suffix = prefix[split..].to_vec();
            suffix.extend_from_slice(b"PrivateClaimedBody");
            let expected = if declared < 0 {
                FrameError::Length
            } else {
                FrameError::BodyCapacity
            };
            assert_eq!(stream.push(&suffix), Err(expected));
            assert!(stream.is_failed());
            assert!(stream.frame().is_none());
            assert!(!stream.consume_ready());
            assert_eq!(stream.push(&NATIVE_SHAPED), Err(FrameError::Failed));
            assert_eq!(storage, [0xa5; 8]);
        }
    }
}

#[test]
fn caller_storage_owns_complete_bytes_and_reuse_never_exposes_previous_or_incomplete_tails() {
    let mut storage = [0xa5; 32];
    let mut stream = FrameAssembler::new(&mut storage).expect("Budget");
    let mut input = [6, 0, 0, 0, b's', b'e', b'c', b'r', b'e', b't'];
    assert!(stream.push(&input).expect("Full").complete);
    input.fill(0xff);
    assert_eq!(stream.frame().expect("Owned body").body(), b"secret");
    assert!(!format!("{stream:?}").contains("secret"));
    assert!(stream.consume_ready());
    assert!(
        !stream
            .push(&[3, 0, 0, 0, b'n', b'e'])
            .expect("Partial next")
            .complete
    );
    assert!(stream.frame().is_none());
    assert!(stream.push(b"w").expect("Complete next").complete);
    assert_eq!(stream.frame().expect("Exact new body").body(), b"new");
    assert_eq!(stream.capacity(), 32);
}

#[test]
fn zero_body_and_empty_storage_still_require_a_full_prefix_and_one_explicit_consume() {
    let mut storage = [];
    let mut stream = FrameAssembler::new(&mut storage).expect("Zero-body-only budget");
    for index in 0..4 {
        assert_eq!(
            stream.push(&[0]),
            Ok(FrameProgress {
                consumed: 1,
                complete: index == 3
            })
        );
    }
    assert_eq!(stream.frame().expect("Empty frame").body(), b"");
    assert!(stream.consume_ready());
    assert_eq!(stream.push(&[1, 0, 0, 0]), Err(FrameError::BodyCapacity));
}
