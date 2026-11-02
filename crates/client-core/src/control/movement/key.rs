// SPDX-License-Identifier: AGPL-3.0-or-later

// Native Com_HashKey uses signed x86 char and arithmetic signed shifts. The fold
// reads the original sum in all three terms. Native maxlen here is exactly 32.
pub(super) fn hash(bytes: &[u8]) -> u32 {
    let sum = bytes
        .iter()
        .take(32)
        .take_while(|&&byte| byte != 0)
        .zip(119_i32..)
        .fold(0_i32, |sum, (&byte, weight)| {
            sum + i32::from(i8::from_ne_bytes([byte])) * weight
        });
    (sum ^ (sum >> 10) ^ (sum >> 20)).cast_unsigned()
}

#[cfg(test)]
mod tests {
    #![allow(
        clippy::expect_used,
        reason = "Compiler-evaluated immutable C hash vectors."
    )]

    #[test]
    fn signed_bytes_arithmetic_fold_nul_and_first_thirty_two_match_compiler_constants() {
        let source = include_str!("../../../tests/fixtures/movement_key_vectors.txt");
        let mut count = 0;
        for row in source
            .lines()
            .filter(|row| !row.starts_with('#') && !row.is_empty())
        {
            let fields: Vec<_> = row.split_whitespace().collect();
            assert_eq!(fields.len(), 3);
            let bytes: Vec<_> = if fields[1] == "-" {
                Vec::new()
            } else {
                fields[1]
                    .as_bytes()
                    .chunks_exact(2)
                    .map(|pair| {
                        u8::from_str_radix(std::str::from_utf8(pair).expect("Hex"), 16)
                            .expect("Byte")
                    })
                    .collect()
            };
            let expected = fields[2].parse::<u32>().expect("Compiler constant");
            assert_eq!(super::hash(&bytes), expected, "{}", fields[0]);
            count += 1;
        }
        assert_eq!(count, 9);
    }
}
