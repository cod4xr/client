// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Iwi6Dimensions, Iwi6DirectImage, Iwi6Format, Iwi6Shape, fixture};

#[test]
fn all_nine_direct_formats_have_exact_dense_or_block_rounded_storage() {
    for (code, format, size) in [
        (1, Iwi6Format::Bgra8, 24),
        (2, Iwi6Format::Bgr8, 18),
        (3, Iwi6Format::LuminanceAlpha8, 12),
        (4, Iwi6Format::Luminance8, 6),
        (5, Iwi6Format::Alpha8, 6),
        (11, Iwi6Format::Bc1, 8),
        (12, Iwi6Format::Bc2, 16),
        (13, Iwi6Format::Bc3, 16),
        (14, Iwi6Format::Bc5, 16),
    ] {
        let bytes = fixture(code, 2, [3, 2, 1], &[size]);
        let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("Independent format vector");
        assert_eq!(image.header().format(), format);
        assert_eq!(image.header().shape(), Iwi6Shape::Image2D);
        assert_eq!(image.header().mip_levels(), 1);
        assert_eq!(image.header().texels(), 6);
        let mip = image.mip(0).expect("Base mip");
        assert_eq!(mip.bytes(), vec![1; size]);
        assert_eq!(mip.face(0), Some(mip.bytes()));
        assert_eq!(mip.face(1), None);
        assert!(image.mip(1).is_none());
    }
}

#[test]
fn reverse_physical_mips_have_logical_base_first_views_and_no_picmip_keeps_all_levels() {
    let bytes = fixture(2, 1, [8, 4, 1], &[96, 24, 6, 3]);
    assert_eq!(&bytes[28..31], &[4; 3]);
    let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("Reverse physical mip vector");
    assert_eq!(image.header().mip_levels(), 4);
    assert_eq!(image.header().texels(), 43);
    assert_eq!(image.mips().len(), 4);
    for (mip, (width, height, size)) in
        image
            .mips()
            .zip([(8, 4, 96), (4, 2, 24), (2, 1, 6), (1, 1, 3)])
    {
        assert_eq!(
            mip.dimensions(),
            Iwi6Dimensions {
                width,
                height,
                depth: 1
            }
        );
        assert_eq!(
            mip.bytes(),
            vec![u8::try_from(mip.level() + 1).expect("Marker"); size]
        );
    }
}

#[test]
fn non_power_of_two_bc_mips_round_blocks_and_ignore_inactive_stored_depth() {
    let bytes = fixture(11, 0, [7, 5, u16::MAX], &[32, 8, 8]);
    let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("NPOT block sizes");
    assert_eq!(
        image.header().stored_dimensions().depth,
        u32::from(u16::MAX)
    );
    assert_eq!(image.header().dimensions().depth, 1);
    assert_eq!(image.header().mip_levels(), 3);
    assert_eq!(image.header().texels(), 42);
    assert_eq!(
        image
            .mips()
            .map(|mip| mip.bytes().len())
            .collect::<Vec<_>>(),
        [32, 8, 8]
    );
}

#[test]
fn cube_faces_are_contiguous_and_cube_flag_has_native_priority_over_volume() {
    let mut bytes = fixture(1, 0xfe, [2, 3, 99], &[144]);
    for (face, data) in bytes[28..].chunks_exact_mut(24).enumerate() {
        data.fill(u8::try_from(face + 10).expect("Face marker"));
    }
    let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("Six-face vector");
    assert_eq!(image.header().shape(), Iwi6Shape::Cube);
    assert_eq!(image.header().flags(), 0xfe);
    assert_eq!(image.header().texels(), 36);
    let mip = image.mip(0).expect("Cube base");
    assert_eq!(mip.face_count(), 6);
    assert_eq!(mip.dimensions().depth, 1);
    for face in 0..6 {
        assert_eq!(mip.face(face), Some(&vec![face + 10; 24][..]));
    }
    assert_eq!(mip.face(6), None);
}

#[test]
fn volume_mips_shrink_depth_and_contain_all_slices_in_one_face() {
    let bytes = fixture(4, 8, [4, 2, 3], &[24, 2, 1]);
    let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("Volume vector");
    assert_eq!(image.header().shape(), Iwi6Shape::Volume3D);
    assert_eq!(image.header().texels(), 27);
    assert_eq!(
        image
            .mips()
            .map(|mip| mip.dimensions().depth)
            .collect::<Vec<_>>(),
        [3, 1, 1]
    );
    assert!(image.mips().all(|mip| mip.face_count() == 1));
}

#[test]
fn complete_u16_dimension_profile_supports_sixteen_levels_without_native_arenas() {
    let sizes = [
        65535, 32767, 16383, 8191, 4095, 2047, 1023, 511, 255, 127, 63, 31, 15, 7, 3, 1,
    ];
    let bytes = fixture(4, 0, [u16::MAX, 1, 1], &sizes);
    let image = Iwi6DirectImage::parse(&bytes, BUDGET).expect("Full profile dimension range");
    assert_eq!(image.header().mip_levels(), 16);
    assert_eq!(image.header().texels(), 131_054);
    assert_eq!(image.mip(15).expect("Final level").bytes(), &[16]);
}
