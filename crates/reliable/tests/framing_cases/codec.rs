// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::framing::{FrameError, ReliableFrame, required_frame_bytes};

use crate::NATIVE_SHAPED;

#[test]
fn independent_native_length_excludes_prefix_and_every_short_encode_preserves_output() {
    let frame = ReliableFrame::parse(&NATIVE_SHAPED).expect("Native-shaped frame");
    assert_eq!(frame.body(), &[1, 0, 0, 0, 1, 0xfe, 0, 0x92, 0xff]);
    assert_eq!(frame.encoded_len(), 13);
    for capacity in 0..13 {
        let mut output = [0xa5; 16];
        assert_eq!(
            frame.encode(&mut output[..capacity]),
            Err(FrameError::OutputCapacity)
        );
        assert_eq!(output, [0xa5; 16]);
    }
    let mut output = [0xa5; 16];
    assert_eq!(frame.encode(&mut output), Ok(13));
    assert_eq!(&output[..13], &NATIVE_SHAPED);
    assert_eq!(&output[13..], &[0xa5; 3]);
    assert!(
        !format!("{:?}", ReliableFrame::new(b"PrivateCommand").expect("Body"))
            .contains("PrivateCommand")
    );
    let empty = ReliableFrame::new(b"").expect("Empty framing, no command authority");
    assert_eq!(empty.encode(&mut output), Ok(4));
    assert_eq!(&output[..4], &[0; 4]);
}

#[test]
fn whole_frame_parser_rejects_missing_or_hidden_bytes_and_negative_length_domains() {
    for length in 0..NATIVE_SHAPED.len() {
        assert!(ReliableFrame::parse(&NATIVE_SHAPED[..length]).is_err());
    }
    let mut trailing = NATIVE_SHAPED.to_vec();
    trailing.push(0);
    assert_eq!(
        ReliableFrame::parse(&trailing).expect_err("Hidden suffix"),
        FrameError::LengthMismatch
    );
    for declared in [-1_i32, i32::MIN] {
        assert_eq!(
            ReliableFrame::parse(&declared.to_le_bytes()).expect_err("Negative"),
            FrameError::Length
        );
    }
    assert_eq!(required_frame_bytes(usize::MAX), Err(FrameError::Length));
    assert_eq!(required_frame_bytes(0), Ok(4));
    assert_eq!(
        ReliableFrame::parse(&[0; 4]).expect("Zero body").body(),
        b""
    );
}
