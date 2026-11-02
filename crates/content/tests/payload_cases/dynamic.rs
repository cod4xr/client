// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, ZipPayloadDecoder, entry, hex, source};

#[test]
fn independent_dynamic_deflate_crosses_history_windows_and_preserves_every_byte() {
    // Python zlib.compressobj(6, DEFLATED, -15) over bytes(range(256)) * 257.
    let packed = hex(concat!(
        "edcf030210060000c06cdbb66ddbe6b26ddbb66d6e59cbb66ddbb6eb21773fb8000103050e123458",
        "f0102143850e13365cf8081123458e12355af4183163c58e13375efc040913254e923459f2142953",
        "a54e93365dfa0c193365ce92355bf61c3973e5ce93375ffe02050b152e52b458f112254b952e53b6",
        "5cf90a152b55ae52b55af51a35ffa955bb4edd7af51b346cd4b849d366cd5bb46cd5ba4ddb76ed3b",
        "74ecd4b94bd76edd7bf4ecd5bb4fdf7efd070c1c3478c8d061c3478c1c357accd871e3274c9c3479",
        "cad469d367cc9c357bcedc79f3172c5cb478c9d265cbfffd6fc5ca55abd7ac5db77ec3c64dff6fde",
        "b275dbf61d3b77eddeb377dffe03070f1d3e72f4d8f113274f9d3e73f6dcf90b172f5dbe72f5daf5",
        "1b376fddbe73f7defd070f1f3d7ef2f4d9f3172f5fbd7ef3f6ddfb0f1f3f7dfef2f5dbf71f3f7ffd",
        "0ee0efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefefef",
        "efefefefefefefefefefefefefefeffff7ff07",
    ));
    assert_eq!((packed[0] >> 1) & 3, 2);
    let expected: Vec<u8> = (0..257).flat_map(|_| 0..=255).collect();
    let fixture = source(&packed, 65_792, 0x6d4d_0c00, 8);
    let mut output = vec![0xa5; expected.len() + 3];
    let mut scratch = vec![0x92; expected.len() + 1];
    let checked = ZipPayloadDecoder::new()
        .decode(entry(&fixture.bytes), BUDGET, &mut scratch, &mut output)
        .expect("Independent dynamic stream beyond 32 KiB");
    assert_eq!(checked.bytes(), expected);
    assert_eq!(&output[expected.len()..], &[0xa5; 3]);
}
