// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Spec, Zip32Directory, Zip32EntrySlot, Zip32Error, build, rejected, word};

#[test]
fn signed_and_unsigned_descriptors_validate_and_crc_signature_collisions_are_unambiguous() {
    for signed in [false, true] {
        for crc in [0x1234_5678, 0x0807_4b50] {
            let fixture = build(
                &[Spec {
                    flags: 8,
                    crc,
                    descriptor_signature: signed,
                    ..Spec::stored(b"stream", b"data")
                }],
                &[],
                &[],
            );
            let mut slots = [Zip32EntrySlot::default()];
            let catalog = Zip32Directory::parse(
                &fixture.bytes,
                BUDGET,
                &mut [Zip32EntrySlot::default()],
                &mut slots,
            )
            .expect("Streaming descriptor");
            assert_eq!(
                catalog.entries().next().expect("Entry").declared_crc32(),
                crc
            );
            for offset in 0..12 {
                let mut bytes = fixture.bytes.clone();
                let descriptor = fixture.central[0] - 12;
                bytes[descriptor + offset] ^= 1;
                assert_eq!(rejected(&bytes), Zip32Error::Descriptor);
            }
            let mut bytes = fixture.bytes.clone();
            word(&mut bytes, fixture.locals[0] + 14, crc);
            word(&mut bytes, fixture.locals[0] + 18, 4);
            word(&mut bytes, fixture.locals[0] + 22, 4);
            Zip32Directory::parse(
                &bytes,
                BUDGET,
                &mut [Zip32EntrySlot::default()],
                &mut [Zip32EntrySlot::default()],
            )
            .expect("Matching populated fields");
            word(&mut bytes, fixture.locals[0] + 18, 5);
            assert_eq!(rejected(&bytes), Zip32Error::LocalMismatch);
        }
    }
}

#[test]
fn declared_output_totals_are_wide_and_do_not_allocate_by_output_size() {
    let fixture = build(
        &[
            Spec {
                method: 8,
                unpacked: 3_000_000_000,
                ..Spec::stored(b"a", b"x")
            },
            Spec {
                method: 8,
                unpacked: 3_000_000_000,
                ..Spec::stored(b"b", b"y")
            },
        ],
        &[],
        &[],
    );
    let mut slots = [Zip32EntrySlot::default(); 2];
    let catalog = Zip32Directory::parse(
        &fixture.bytes,
        BUDGET,
        &mut [Zip32EntrySlot::default(); 2],
        &mut slots,
    )
    .expect("Metadata only, no decompression allocation");
    assert_eq!(catalog.stats().declared_uncompressed_bytes, 6_000_000_000);
    assert_eq!(catalog.stats().compressed_bytes, 2);
}
