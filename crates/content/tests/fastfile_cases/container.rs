// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn stored_container_checks_zlib_and_copies_all_nine_block_declarations() {
    let input = stored(&inner(
        b"PrivateAssets",
        13,
        777,
        [1, 2, 3, 4, 5, 6, 7, 8, 9],
    ));
    let mut decoder = Fastfile5Decoder::new();
    let meta = decoder.inspect(&input, BUDGET).expect("size metadata");
    assert_eq!(meta.body_bytes(), 13);
    assert_eq!(meta.external_bytes(), 777);
    assert_eq!(meta.total_block_bytes(), 45);
    assert_eq!((meta.decoded_bytes(), meta.scratch_bytes()), (57, 58));
    for (index, block) in [
        Fastfile5Block::Temporary,
        Fastfile5Block::Runtime,
        Fastfile5Block::LargeRuntime,
        Fastfile5Block::PhysicalRuntime,
        Fastfile5Block::Virtual,
        Fastfile5Block::Large,
        Fastfile5Block::Physical,
        Fastfile5Block::Vertex,
        Fastfile5Block::Index,
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            meta.block_bytes(block),
            u32::try_from(index + 1).expect("nine sizes")
        );
    }
    let mut scratch = [0; 58];
    let mut output = [0xa5; 61];
    let checked = decoder
        .decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("complete container");
    assert_eq!(checked.metadata(), meta);
    assert_eq!(checked.contents(), b"PrivateAssets");
    assert_eq!(checked.decoded_bytes().len(), 57);
    assert_eq!(output[57..], [0xa5; 4]);
}

#[test]
fn independently_compressed_history_vector_has_exact_content_and_metadata() {
    let mut decoder = Fastfile5Decoder::default();
    let input = outer(HISTORY);
    let mut scratch = vec![0; 10045];
    let mut output = vec![0; 10044];
    let checked = decoder
        .decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("independent dynamic-Huffman stream");
    assert_eq!(checked.contents(), b"abcde".repeat(2000));
    assert_eq!(checked.metadata().body_bytes(), 10000);
    assert_eq!(checked.metadata().total_block_bytes(), 45);
}

#[test]
fn zero_body_is_a_valid_container_without_claiming_an_asset_root() {
    let input = stored(&inner(&[], 0, 0, [0; 9]));
    let mut scratch = [0; 45];
    let mut output = [0; 44];
    let checked = Fastfile5Decoder::new()
        .decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("empty container");
    assert!(checked.contents().is_empty());
    assert_eq!(checked.metadata().total_block_bytes(), 0);
}

#[test]
fn declared_short_or_long_body_and_checksum_fail_without_partial_publication() {
    let mut decoder = Fastfile5Decoder::new();
    let mut scratch = [0; 64];
    let mut output = [0x5a; 64];
    for declared in [0, 3, 5, 9] {
        let input = stored(&inner(b"body", declared, 0, [0; 9]));
        assert_eq!(
            decoder
                .decode(&input, BUDGET, &mut scratch, &mut output)
                .err(),
            Some(Fastfile5Error::OutputLength)
        );
        assert_eq!(output, [0x5a; 64]);
    }
    let mut input = stored(&inner(b"body", 4, 0, [0; 9]));
    let end = input.len() - 1;
    input[end] ^= 1;
    assert_eq!(
        decoder
            .decode(&input, BUDGET, &mut scratch, &mut output)
            .err(),
        Some(Fastfile5Error::InvalidZlib)
    );
    assert_eq!(output, [0x5a; 64]);
}

#[test]
fn trailing_and_concatenated_streams_and_preset_dictionaries_are_rejected() {
    let input = stored(&inner(b"body", 4, 0, [0; 9]));
    let mut decoder = Fastfile5Decoder::new();
    let mut scratch = [0; 49];
    let mut output = [0x77; 48];
    for tail in [&[0][..], &input[12..]] {
        let mut bytes = input.clone();
        bytes.extend_from_slice(tail);
        assert_eq!(
            decoder
                .decode(&bytes, BUDGET, &mut scratch, &mut output)
                .err(),
            Some(Fastfile5Error::TrailingInput)
        );
        assert_eq!(output, [0x77; 48]);
    }
    let mut dictionary = input;
    dictionary[13] = 0x20; // Valid FCHECK with FDICT set; no dictionary is authorized.
    dictionary.splice(14..14, [0, 0, 0, 1]);
    assert!(
        decoder
            .decode(&dictionary, BUDGET, &mut scratch, &mut output)
            .is_err()
    );
    assert_eq!(output, [0x77; 48]);
}
