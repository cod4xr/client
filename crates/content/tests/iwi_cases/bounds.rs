// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Iwi6Budget, Iwi6DirectImage, Iwi6Error, Iwi6Header, fixture, word};

#[test]
fn every_prefix_and_appended_byte_rejects_before_a_complete_image_can_escape() {
    let source = fixture(2, 1, [8, 4, 1], &[96, 24, 6, 3]);
    for length in 0..source.len() {
        assert!(Iwi6DirectImage::parse(&source[..length], BUDGET).is_err());
    }
    for byte in 0..=255 {
        let mut bytes = source.clone();
        bytes.push(byte);
        assert_eq!(
            Iwi6DirectImage::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::TrailingInput)
        );
    }
    assert!(Iwi6Header::parse(&source[..28], BUDGET).is_ok());
    assert!(Iwi6DirectImage::parse(&source[..28], BUDGET).is_err());
}

#[test]
fn used_picmip_extents_are_exact_but_inactive_table_entries_have_no_authority() {
    let source = fixture(2, 1, [8, 4, 1], &[96, 24, 6, 3]);
    for level in 0..4 {
        let mut bytes = source.clone();
        word(&mut bytes, 12 + level * 4, u32::MAX);
        assert_eq!(
            Iwi6DirectImage::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::PicmipSize)
        );
    }
    let mut no_mips = fixture(11, 2, [4, 4, 1], &[8]);
    for level in 1..4 {
        word(&mut no_mips, 12 + level * 4, u32::MAX);
    }
    assert!(Iwi6DirectImage::parse(&no_mips, BUDGET).is_ok());
}

#[test]
fn host_byte_mip_and_total_texel_budgets_are_independent_and_exact() {
    let source = fixture(2, 1, [8, 4, 1], &[96, 24, 6, 3]);
    let exact = Iwi6Budget {
        max_file_bytes: source.len(),
        max_texels: 43,
        max_mip_levels: 4,
    };
    assert!(Iwi6DirectImage::parse(&source, exact).is_ok());
    for (budget, expected) in [
        (
            Iwi6Budget {
                max_file_bytes: source.len() - 1,
                ..exact
            },
            Iwi6Error::FileBudget,
        ),
        (
            Iwi6Budget {
                max_texels: 42,
                ..exact
            },
            Iwi6Error::TexelBudget,
        ),
        (
            Iwi6Budget {
                max_mip_levels: 3,
                ..exact
            },
            Iwi6Error::MipBudget,
        ),
    ] {
        assert_eq!(
            Iwi6DirectImage::parse(&source, budget).err(),
            Some(expected)
        );
    }
    let cube = fixture(1, 6, [2, 3, 1], &[144]);
    assert_eq!(
        Iwi6Header::parse(
            &cube,
            Iwi6Budget {
                max_texels: 35,
                ..BUDGET
            }
        )
        .err(),
        Some(Iwi6Error::TexelBudget)
    );
    let large_volume = fixture(1, 8, [u16::MAX; 3], &[]);
    assert_eq!(
        Iwi6Header::parse(&large_volume, BUDGET).err(),
        Some(Iwi6Error::TexelBudget)
    );
}

#[test]
fn deterministic_header_mutations_never_expose_spans_outside_caller_image_bytes() {
    let source = fixture(11, 0, [7, 5, 1], &[32, 8, 8]);
    for offset in 0..28 {
        for mask in [1, 2, 4, 8, 16, 32, 64, 128, 255] {
            let mut bytes = source.clone();
            bytes[offset] ^= mask;
            if let Ok(image) = Iwi6DirectImage::parse(&bytes, BUDGET) {
                assert_eq!(image.mips().len(), image.header().mip_levels());
                for mip in image.mips() {
                    assert!(mip.bytes().len() <= bytes.len());
                    for face in 0..mip.face_count() {
                        assert!(mip.face(face).is_some());
                    }
                    assert!(mip.face(mip.face_count()).is_none());
                }
            }
        }
    }
}
