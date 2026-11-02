// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{WaveletError, bits::Bits, tables};

pub(super) struct Codeword(pub u16, pub u8, pub i16);

#[derive(Clone, Copy)]
struct Entry {
    value: i16,
    bits: u8,
}

// The importer independently rejects gaps, overlaps and out-of-range words.
// Building every slot from its prefix also prevents a mutable runtime registry.
const fn lookup(words: &[Codeword]) -> [Entry; 4096] {
    let mut result = [Entry { value: 0, bits: 0 }; 4096];
    let mut index = 0;
    while index < words.len() {
        let Codeword(code, bits, value) = words[index];
        let mut slot = code as usize;
        while slot < 4096 {
            result[slot] = Entry { value, bits };
            slot += 1 << bits;
        }
        index += 1;
    }
    result
}
static BLUE: [Entry; 4096] = lookup(&tables::blue::WORDS);
static RED_GREEN: [Entry; 4096] = lookup(&tables::red_green::WORDS);
static ALPHA: [Entry; 4096] = lookup(&tables::alpha::WORDS);

#[derive(Clone, Copy)]
pub(super) enum Book {
    Blue,
    RedGreen,
    Alpha,
}
impl Book {
    pub fn value(self, bits: &mut Bits<'_>) -> Result<i32, WaveletError> {
        let (table, escape_bits, escape_offset) = match self {
            Self::Blue => (&BLUE, 9, 255),
            Self::RedGreen => (&RED_GREEN, 10, 510),
            Self::Alpha => (&ALPHA, 9, 255),
        };
        let entry = table[usize::from(bits.peek(12))];
        if entry.bits == 0 {
            return Err(WaveletError::Codeword);
        }
        bits.take(entry.bits)?;
        if entry.value == i16::MIN {
            Ok(i32::from(bits.take(escape_bits)?) - escape_offset)
        } else {
            Ok(i32::from(entry.value))
        }
    }
    pub fn coefficients(self, bits: &mut Bits<'_>) -> Result<[i32; 3], WaveletError> {
        Ok([self.value(bits)?, self.value(bits)?, self.value(bits)?])
    }
}
