// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{MAX_MESSAGE_BYTES, padded, skip_padding, state};
use cod4x_protocol::{MessageReader, MessageStringLimit};
use cod4xr_client_core::message_body::BodyReader;

#[test]
fn independent_cached_bit_byte_vector_survives_rebasing_beyond_one_megabyte_and_byte_reads() {
    for prefix in [
        0,
        MAX_MESSAGE_BYTES - 1,
        MAX_MESSAGE_BYTES,
        MAX_MESSAGE_BYTES + 1,
        MAX_MESSAGE_BYTES * 8 + 11,
    ] {
        let input = padded(&[0xdd, 0xaa], prefix);
        let mut reader =
            BodyReader::try_new(&input, input.len()).expect("Explicit full-body budget");
        skip_padding(&mut reader, prefix);
        assert_eq!(reader.read_bits(3), Ok(5));
        assert_eq!(reader.read_u8(), Ok(0xaa));
        assert_eq!(state(&reader), (prefix + 2, 0, 5));
        let before = state(&reader);
        assert!(reader.read_bits(6).is_err());
        assert_eq!(state(&reader), before);
        assert!(reader.read_u8().is_err());
        assert_eq!(state(&reader), before);
        assert_eq!(reader.read_bits(5), Ok(27));
        assert_eq!(reader.read_bits(0), Ok(0));
        assert_eq!(state(&reader), (prefix + 2, 0, 0));
    }
}

#[test]
fn mixed_primitive_trace_matches_the_pinned_shared_reader_after_a_large_input_prefix() {
    let payload: Vec<u8> = (0_usize..8192)
        .map(|index| index.wrapping_mul(37).wrapping_add(11).to_le_bytes()[0])
        .collect();
    let input = padded(&payload, MAX_MESSAGE_BYTES + 7);
    let mut actual = BodyReader::try_new(&input, input.len()).expect("Body");
    skip_padding(&mut actual, MAX_MESSAGE_BYTES + 7);
    let mut reference = MessageReader::try_new(&payload).expect("Shared field trace");
    for _ in 0..10 {
        for width in 0..=32 {
            assert_eq!(
                actual.read_bits(width).expect("Bits"),
                reference.read_bits(width).expect("Shared bits")
            );
            match width % 5 {
                0 => assert_eq!(
                    actual.read_u8().expect("Byte"),
                    reference.read_u8().expect("Shared byte")
                ),
                1 => assert_eq!(
                    actual.read_i16().expect("Short"),
                    reference.read_i16().expect("Shared short")
                ),
                2 => assert_eq!(
                    actual.read_i32().expect("Integer"),
                    reference.read_i32().expect("Shared integer")
                ),
                3 => assert_eq!(
                    actual.read_i64().expect("Wide"),
                    reference.read_i64().expect("Shared wide")
                ),
                _ => assert_eq!(
                    actual.read_f32().expect("Float").to_bits(),
                    reference.read_f32().expect("Shared float").to_bits()
                ),
            }
            assert_eq!(actual.remaining_bytes(), reference.remaining_bytes());
        }
    }
}

#[test]
fn cached_bits_survive_a_megabyte_byte_copy_and_shared_big_string_normalization() {
    let padding = MAX_MESSAGE_BYTES * 8;
    let mut input = vec![0xdd];
    input.extend_from_slice(&vec![0xfe; padding]);
    input.extend_from_slice(&vec![b'%'; 8190]);
    input.extend_from_slice(&[0x92, 0]);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Explicit body budget");
    assert_eq!(reader.read_bits(3), Ok(5));
    skip_padding(&mut reader, padding);
    let before = state(&reader);
    let mut output = [0xa5; 8191];
    assert!(
        reader
            .read_legacy_string(&mut output, MessageStringLimit::Normal)
            .is_err()
    );
    assert_eq!(state(&reader), before);
    assert!(output.iter().all(|&byte| byte == 0xa5));
    assert!(
        reader
            .read_legacy_string(&mut output[..8190], MessageStringLimit::Big)
            .is_err()
    );
    assert_eq!(state(&reader), before);
    assert!(output.iter().all(|&byte| byte == 0xa5));
    assert_eq!(
        reader.read_legacy_string(&mut output, MessageStringLimit::Big),
        Ok(8191)
    );
    assert!(output[..8190].iter().all(|&byte| byte == b'.'));
    assert_eq!(output[8190], 0x92);
    assert_eq!(reader.read_bits(5), Ok(27));
    assert_eq!(reader.remaining_bytes(), 0);
}
