// SPDX-License-Identifier: AGPL-3.0-or-later
pub fn inner(body: &[u8], declared: u32, external: u32, blocks: [u32; 9]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for word in [declared, external].into_iter().chain(blocks) {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    bytes.extend_from_slice(body);
    bytes
}
pub fn outer(zlib: &[u8]) -> Vec<u8> {
    let mut input = b"IWffu100".to_vec();
    input.extend_from_slice(&5_u32.to_le_bytes());
    input.extend_from_slice(zlib);
    input
}
// Independent RFC1950/1951 stored-block encoder, not the production inflater.
pub fn stored(decoded: &[u8]) -> Vec<u8> {
    let mut zlib = vec![0x78, 0x01];
    let blocks = decoded.len().max(1).div_ceil(65535);
    for index in 0..blocks {
        let begin = index * 65535;
        let end = (begin + 65535).min(decoded.len());
        let part = &decoded[begin..end];
        let length = u16::try_from(part.len()).expect("Bounded stored block");
        zlib.push(u8::from(index + 1 == blocks));
        zlib.extend_from_slice(&length.to_le_bytes());
        zlib.extend_from_slice(&(!length).to_le_bytes());
        zlib.extend_from_slice(part);
    }
    let (mut a, mut b) = (1_u32, 0_u32);
    for &byte in decoded {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    zlib.extend_from_slice(&((b << 16) | a).to_be_bytes());
    outer(&zlib)
}
