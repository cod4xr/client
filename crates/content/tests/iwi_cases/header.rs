// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Iwi6DirectImage, Iwi6Error, Iwi6Format, Iwi6Header, fixture, short};

#[test]
fn known_wavelet_headers_are_metadata_only_and_never_admitted_as_direct_pixels() {
    for (code, expected) in [
        (6, Iwi6Format::WaveletRgba),
        (7, Iwi6Format::WaveletRgb),
        (8, Iwi6Format::WaveletLuminanceAlpha),
        (9, Iwi6Format::WaveletLuminance),
        (10, Iwi6Format::WaveletAlpha),
    ] {
        let bytes = fixture(code, 2, [1, 1, 1], &[]);
        let header = Iwi6Header::parse(&bytes, BUDGET).expect("Known wavelet metadata");
        assert_eq!(header.format(), expected);
        assert!(header.format().is_wavelet());
        assert_eq!(
            Iwi6DirectImage::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::Wavelet)
        );
    }
}

#[test]
fn explicit_profile_unknown_formats_and_zero_interpreted_dimensions_reject() {
    let source = fixture(11, 2, [4, 4, 1], &[8]);
    for version in [0, 5, 8, 13, 27, 255] {
        let mut bytes = source.clone();
        bytes[3] = version;
        assert_eq!(
            Iwi6Header::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::Profile)
        );
    }
    for code in (0..=255).filter(|code| !(1..=14).contains(code)) {
        let mut bytes = source.clone();
        bytes[4] = code;
        assert_eq!(
            Iwi6Header::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::Format)
        );
    }
    let mut bad_magic = source.clone();
    bad_magic[0] = b'i';
    assert_eq!(
        Iwi6Header::parse(&bad_magic, BUDGET).err(),
        Some(Iwi6Error::Profile)
    );
    for (flags, offset) in [(2, 6), (2, 8), (10, 10)] {
        let mut bytes = source.clone();
        bytes[5] = flags;
        short(&mut bytes, offset, 0);
        assert_eq!(
            Iwi6Header::parse(&bytes, BUDGET).err(),
            Some(Iwi6Error::Dimensions)
        );
    }
}

#[test]
fn owned_headers_and_borrowed_mip_views_do_not_retain_parent_plan_or_private_pixels_in_debug() {
    let header = {
        let source = fixture(11, 2, [4, 4, 1], &[8]);
        Iwi6Header::parse(&source, BUDGET).expect("Owned metadata")
    };
    assert_eq!(header.dimensions().width, 4);
    let mut source = fixture(1, 2, [2, 2, 1], &[16]);
    source[28..].copy_from_slice(b"PrivatePixels123");
    let mip = {
        let plan = Iwi6DirectImage::parse(&source, BUDGET).expect("Complete image");
        assert!(!format!("{plan:?}").contains("Private"));
        plan.mip(0).expect("Independent view")
    };
    assert_eq!(mip.bytes(), b"PrivatePixels123");
    assert!(!format!("{mip:?}").contains("Private"));
    assert!(!format!("{header:?}").contains("Private"));
}
