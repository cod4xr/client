// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn content_and_metadata_outlive_encoded_input_work_and_decoder_without_debug_leaks() {
    let mut output = [0; 57];
    let (metadata, contents) = {
        let input = stored(&inner(b"PrivateAssets", 13, 0, [0; 9]));
        let mut scratch = [0; 58];
        let mut decoder = Fastfile5Decoder::new();
        let checked = decoder
            .decode(&input, BUDGET, &mut scratch, &mut output)
            .expect("owned output");
        assert!(!format!("{checked:?}").contains("PrivateAssets"));
        assert!(!format!("{decoder:?}").contains("PrivateAssets"));
        (checked.metadata(), checked.contents())
    };
    assert_eq!(contents, b"PrivateAssets");
    assert_eq!(metadata.decoded_bytes(), 57);
}

#[test]
fn inspection_is_only_metadata_and_decoder_reuses_clean_state_after_failure() {
    let input = stored(&inner(b"body", 4, 0, [0; 9]));
    let incomplete = &input[..12 + 7 + 45];
    let mut decoder = Fastfile5Decoder::new();
    let metadata = decoder
        .inspect(incomplete, BUDGET)
        .expect("prefix requirements without complete body");
    assert_eq!(metadata.decoded_bytes(), 48);
    let mut scratch = [0; 49];
    let mut output = [0xa5; 48];
    assert!(
        decoder
            .decode(incomplete, BUDGET, &mut scratch, &mut output)
            .is_err()
    );
    assert_eq!(output, [0xa5; 48]);
    let checked = decoder
        .decode(&input, BUDGET, &mut scratch, &mut output)
        .expect("reset after failure");
    assert_eq!(checked.contents(), b"body");
    let other = stored(&inner(b"next", 4, 0, [0; 9]));
    let checked = decoder
        .decode(&other, BUDGET, &mut scratch, &mut output)
        .expect("reset after success");
    assert_eq!(checked.contents(), b"next");
}
