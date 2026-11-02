// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, OPTIONAL, SECTIONS, fixture, short, word};
use cod4xr_content::pe32::{Pe32Budget, Pe32Image};

#[test]
fn every_required_file_prefix_rejects_but_unused_overlay_bytes_are_not_required() {
    let bytes = fixture();
    for end in 0..0x500 {
        assert!(
            Pe32Image::parse(&bytes[..end], BUDGET).is_err(),
            "Required prefix {end}"
        );
    }
    assert!(Pe32Image::parse(&bytes[..0x500], BUDGET).is_ok());
    assert!(Pe32Image::parse(&bytes, BUDGET).is_ok());
}

#[test]
fn invalid_signatures_formats_optional_directories_and_header_counts_reject() {
    let source = fixture();
    for (offset, value) in [(0, b'X'), (0x80, b'X')] {
        let mut bytes = source.clone();
        bytes[offset] = value;
        assert!(Pe32Image::parse(&bytes, BUDGET).is_err());
    }
    for (offset, value) in [
        (0x84, 0x8664),
        (OPTIONAL, 0x20b),
        (0x94, 64),
        (0x94, 0),
        (0x86, 0),
    ] {
        let mut bytes = source.clone();
        short(&mut bytes, offset, value);
        assert!(Pe32Image::parse(&bytes, BUDGET).is_err());
    }
    for (offset, value) in [
        (0x3c, 0),
        (0x3c, u32::MAX),
        (OPTIONAL + 92, 17),
        (OPTIONAL + 92, u32::MAX),
    ] {
        let mut bytes = source.clone();
        word(&mut bytes, offset, value);
        assert!(Pe32Image::parse(&bytes, BUDGET).is_err());
    }
    let mut bytes = source;
    short(&mut bytes, 0x86, u16::MAX);
    assert!(
        Pe32Image::parse(
            &bytes,
            Pe32Budget {
                max_sections: u16::MAX,
                ..BUDGET
            }
        )
        .is_err()
    );
}

#[test]
fn unordered_overlapping_truncated_and_overflowing_declared_geometry_rejects() {
    let source = fixture();
    for (offset, value) in [
        (SECTIONS + 12, 0x100),
        (SECTIONS + 20, 0x100),
        (SECTIONS + 16, u32::MAX),
        (SECTIONS + 20, u32::MAX),
        (SECTIONS + 12, u32::MAX),
        (SECTIONS + 40 + 12, 0x1100),
        (SECTIONS + 40 + 12, 0x1000),
        (SECTIONS + 40 + 8, u32::MAX),
        (OPTIONAL + 28, 0xffff_f000),
        (OPTIONAL + 56, 0x2100),
        (OPTIONAL + 56, 0),
        (OPTIONAL + 60, 0x1c0),
        (OPTIONAL + 60, 0x601),
    ] {
        let mut bytes = source.clone();
        word(&mut bytes, offset, value);
        assert!(
            Pe32Image::parse(&bytes, BUDGET).is_err(),
            "Geometry field {offset}"
        );
    }
}

#[test]
fn deterministic_header_mutations_and_address_workloads_never_escape_checked_slices() {
    let source = fixture();
    let mut random = 0x1234_5678_u32;
    for case in 0..4096 {
        let mut bytes = source.clone();
        for _ in 0..=case % 4 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let position = usize::try_from(random).expect("Host index") % bytes.len();
            bytes[position] ^= random.to_le_bytes()[3];
        }
        if let Ok(image) = Pe32Image::parse(&bytes, BUDGET) {
            for address in [
                random,
                image.image_base().wrapping_add(random & 0x3fff),
                0x0040_1000,
                u32::MAX,
            ] {
                let _ = image.file_tail(address);
                let _ = image.file_span(address, usize::from(random.to_le_bytes()[0]));
            }
        }
    }
}
