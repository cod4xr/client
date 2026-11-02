// SPDX-License-Identifier: AGPL-3.0-or-later
/// Exact SHA-256 of the locally audited installed multiplayer image.
/// This identifies one reviewed data layout; it is not publisher authentication.
pub const REVIEWED_MULTIPLAYER_SHA256: [u8; 32] = [
    0xf5, 0x0f, 0x8a, 0x52, 0x05, 0x81, 0x75, 0x4e, 0x73, 0xcb, 0x7c, 0xae, 0x67, 0xb0, 0x2a, 0xdc,
    0xba, 0xd5, 0xf2, 0xe7, 0x1a, 0xaa, 0x41, 0x2a, 0x01, 0x3b, 0xb3, 0xef, 0x57, 0xa3, 0x91, 0xa5,
];

#[derive(Clone, Copy)]
pub(super) struct Profile {
    pub digest: [u8; 32],
    pub image_bytes: usize,
    pub image_base: u32,
    pub table_address: u32,
    pub records: usize,
    pub text_bytes: usize,
    pub longest_value: usize,
    pub last_index: u32,
    pub index_limit: u32,
}
pub(super) const MULTIPLAYER: Profile = Profile {
    digest: REVIEWED_MULTIPLAYER_SHA256,
    image_bytes: 3_330_048,
    image_base: 0x0040_0000,
    table_address: 0x0072_09b0,
    records: 832,
    text_bytes: 18_813,
    longest_value: 1506,
    last_index: 2273,
    index_limit: 2442,
};
