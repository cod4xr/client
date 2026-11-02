// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, OPTIONAL, SECTIONS, fixture, word};
use cod4xr_content::pe32::{Pe32Budget, Pe32Error, Pe32Image};

#[test]
fn preferred_addresses_map_headers_and_section_bytes_without_relocation_or_utf8() {
    let bytes = fixture();
    let image = Pe32Image::parse(&bytes, BUDGET).expect("Independent PE fixture");
    assert_eq!(image.image_base(), 0x0040_0000);
    assert_eq!(image.image_size(), 0x3000);
    assert_eq!(image.section_count(), 2);
    assert_eq!(image.file_span(0x0040_0000, 2), Ok(&b"MZ"[..]));
    assert_eq!(
        image.file_span(0x0040_1000, 15),
        Ok(&b"Private%Data\x92\0\0"[..])
    );
    assert_eq!(image.file_span(0x0040_11f0, 16), Ok(&[0xa5; 16][..])); // Raw alignment padding.
    assert_eq!(
        image.file_tail(0x0040_2000).expect("Second section").len(),
        0x100
    );
    assert_eq!(image.file_span(0x0040_20ff, 1), Ok(&[0x5a][..]));
    assert_eq!(image.file_span(0x0040_1000, 0), Ok(&[][..]));
    let clone = image;
    assert_eq!(clone.file_span(0x0040_2000, 4), Ok(&[0x5a; 4][..]));
    assert!(!format!("{image:?}").contains("Private"));
}

#[test]
fn gaps_zero_fill_cross_section_extents_and_unbacked_addresses_are_not_file_data() {
    let bytes = fixture();
    let image = Pe32Image::parse(&bytes, BUDGET).expect("PE view");
    for address in [
        0x003f_ffff,
        0x0040_0200,
        0x0040_1200,
        0x0040_1fff,
        0x0040_2100,
        0x0040_22ff,
        0x0040_2300,
        0x0040_3000,
        u32::MAX,
    ] {
        assert_eq!(image.file_tail(address), Err(Pe32Error::Address));
    }
    assert_eq!(image.file_span(0x0040_11ff, 2), Err(Pe32Error::Address));
    assert_eq!(image.file_span(0x0040_20ff, 2), Err(Pe32Error::Address));
    assert_eq!(
        image.file_span(0x0040_1000, usize::MAX),
        Err(Pe32Error::Address)
    );
}

#[test]
fn host_budgets_and_large_virtual_extents_do_not_allocate_or_expose_private_file_data() {
    let mut bytes = fixture();
    assert_eq!(
        Pe32Image::parse(
            &bytes,
            Pe32Budget {
                max_image_bytes: bytes.len() - 1,
                ..BUDGET
            }
        )
        .expect_err("Byte budget"),
        Pe32Error::ImageBudget
    );
    for maximum in [0, 1] {
        assert_eq!(
            Pe32Image::parse(
                &bytes,
                Pe32Budget {
                    max_sections: maximum,
                    ..BUDGET
                }
            )
            .expect_err("Section budget"),
            Pe32Error::SectionBudget
        );
    }
    word(&mut bytes, OPTIONAL + 28, 0);
    word(&mut bytes, OPTIONAL + 56, u32::MAX);
    let huge =
        Pe32Image::parse(&bytes, BUDGET).expect("Virtual size is metadata, never allocation");
    assert_eq!(huge.image_size(), u32::MAX);
    assert_eq!(huge.file_span(0x1000, 4), Ok(&b"Priv"[..]));
    word(&mut bytes, OPTIONAL + 28, 0xffff_d000);
    word(&mut bytes, OPTIONAL + 56, 0x3000);
    let end_at_address_space =
        Pe32Image::parse(&bytes, BUDGET).expect("Exclusive end may equal 2^32");
    assert_eq!(
        end_at_address_space.file_span(0xffff_f000, 1),
        Ok(&[0x5a][..])
    );
}

#[test]
fn raw_empty_bss_sections_remain_virtual_only_without_reading_unrelated_file_offsets() {
    let mut bytes = fixture();
    word(&mut bytes, SECTIONS + 40 + 16, 0);
    word(&mut bytes, SECTIONS + 40 + 20, u32::MAX);
    let image = Pe32Image::parse(&bytes, BUDGET).expect("No raw span belongs to BSS");
    assert_eq!(image.file_tail(0x0040_2000), Err(Pe32Error::Address));
    assert_eq!(image.file_span(0x0040_1000, 4), Ok(&b"Priv"[..]));
}
