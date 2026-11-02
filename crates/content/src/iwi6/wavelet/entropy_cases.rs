// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Finite entropy catalog vectors.")]
use super::{
    WaveletError,
    bits::Bits,
    entropy::{Book, Codeword},
    tables,
};

#[test]
fn every_codeword_suffix_and_escape_value_decodes() {
    for (book, words, escape, offset) in [
        (Book::Blue, tables::blue::WORDS.as_slice(), 9_u8, 255),
        (Book::RedGreen, tables::red_green::WORDS.as_slice(), 10, 510),
        (Book::Alpha, tables::alpha::WORDS.as_slice(), 9, 255),
    ] {
        let mut coverage = [false; 4096];
        for &Codeword(code, length, value) in words {
            for suffix in 0..(1_u32 << (12 - length)) {
                let prefix = u32::from(code) | (suffix << length);
                let index = usize::try_from(prefix).expect("12 bits");
                assert!(!coverage[index], "prefix overlap");
                coverage[index] = true;
                if value != i16::MIN {
                    let bytes = prefix.to_le_bytes();
                    let mut reader = Bits::new(&bytes);
                    assert_eq!(book.value(&mut reader), Ok(i32::from(value)));
                    assert_eq!(reader.consumed(), (usize::from(length / 8), length % 8));
                }
            }
            if value == i16::MIN {
                for escaped in 0..(1_u32 << escape) {
                    let bytes = (u32::from(code) | (escaped << length)).to_le_bytes();
                    let mut reader = Bits::new(&bytes);
                    assert_eq!(
                        book.value(&mut reader),
                        Ok(i32::try_from(escaped).expect("10 bits") - offset)
                    );
                }
            }
        }
        assert!(coverage.into_iter().all(|covered| covered));
    }
}

#[test]
fn short_lookahead_never_authorizes_missing_consumed_bits() {
    for length in 0..=2 {
        for value in 0..=255_u8 {
            let bytes = [value, 0];
            for book in [Book::Blue, Book::RedGreen, Book::Alpha] {
                let mut reader = Bits::new(&bytes[..length]);
                for _ in 0..20 {
                    if book.value(&mut reader).is_err() {
                        break;
                    }
                    let (byte, bit) = reader.consumed();
                    assert!(byte < length || (byte == length && bit == 0));
                }
            }
        }
    }
    let mut reader = Bits::new(&[0xff]);
    assert_eq!(reader.take(7), Ok(127));
    assert_eq!(Book::Alpha.value(&mut reader), Ok(0));
    assert_eq!(Book::Alpha.value(&mut reader), Err(WaveletError::Truncated));
    assert_eq!(reader.consumed(), (1, 0));
}

#[test]
fn raw_prefix_and_bit_cursor_are_bounded_and_monotone() {
    let mut reader = Bits::new(&[0x42, 0xa6, 0x51]);
    assert_eq!(reader.raw(), Ok(0x42));
    assert_eq!(reader.peek(12), 0x1a6);
    assert_eq!(reader.take(5), Ok(6));
    assert_eq!(reader.raw(), Err(WaveletError::RawAfterBits));
    assert_eq!(reader.take(9), Ok(141));
    assert_eq!(reader.consumed(), (2, 6));
    assert_eq!(reader.take(3), Err(WaveletError::Truncated));
    assert_eq!(reader.take(13), Err(WaveletError::BitCount));
    assert_eq!(reader.take(2), Ok(1));
    assert_eq!(reader.consumed(), (3, 0));
}
