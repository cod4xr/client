// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::state;
use cod4x_protocol::{MessageError, MessageStringLimit};
use cod4xr_client_core::message_body::{BodyReadError, BodyReader};

#[test]
fn full_source_preflight_preserves_output_and_both_cursors_on_partial_byte_or_bit_fields() {
    let mut reader = BodyReader::try_new(&[0xdd, 0xaa], 2).expect("Body");
    assert_eq!(reader.read_bits(3), Ok(5));
    let before = state(&reader);
    let mut output = [0xa5; 2];
    assert_eq!(
        reader.read_bytes(&mut output),
        Err(BodyReadError::Message(MessageError::Truncated {
            needed: 2,
            remaining: 1
        }))
    );
    assert_eq!(output, [0xa5; 2]);
    for width in 33..=u8::MAX {
        assert_eq!(
            reader.read_bits(width),
            Err(BodyReadError::Message(MessageError::InvalidBitWidth {
                width
            }))
        );
        assert_eq!(state(&reader), before);
    }
    assert!(reader.read_i16().is_err());
    assert!(reader.read_i32().is_err());
    assert!(reader.read_i64().is_err());
    assert!(reader.read_f32().is_err());
    assert_eq!(state(&reader), before);
    assert_eq!(reader.read_bits(0), Ok(0));
    assert_eq!(state(&reader), before);
    assert_eq!(reader.read_u8(), Ok(0xaa));
    assert_eq!(reader.read_bits(5), Ok(27));
}

#[test]
fn explicit_budget_string_failures_and_debug_privacy_do_not_expose_partial_data() {
    let input = b"PrivateBody%\x92";
    assert!(
        matches!(BodyReader::try_new(input, input.len() - 1), Err(BodyReadError::Message(MessageError::MessageTooLarge { length, maximum })) if length == input.len() && maximum == input.len() - 1)
    );
    let mut reader = BodyReader::try_new(input, usize::MAX).expect("Caller-selected budget");
    let before = state(&reader);
    let mut output = [0xa5; 32];
    assert!(
        reader
            .read_legacy_string(&mut output, MessageStringLimit::Big)
            .is_err()
    );
    assert_eq!(state(&reader), before);
    assert_eq!(output, [0xa5; 32]);
    let debug = format!("{reader:?}");
    assert!(!debug.contains("PrivateBody"));
    assert!(!debug.contains('['));
    let mut empty = BodyReader::try_new(b"", 0).expect("Empty body is representable");
    assert_eq!(empty.read_bits(0), Ok(0));
    assert!(empty.read_u8().is_err());
    assert_eq!(state(&empty), (0, 0, 0));
}
