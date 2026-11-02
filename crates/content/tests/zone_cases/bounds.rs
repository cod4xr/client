// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn every_root_prefix_truncation_rejects_before_a_catalog_exists() {
    let (body, extent) = body(
        &[Some(b"ab"), Some(b"%\xff")],
        &[(3, u32::MAX), (31, u32::MAX)],
    );
    for end in 0..body.len() {
        let mut cooked = Vec::new();
        let container = checked(&body[..end], extent, &mut cooked);
        let mut strings = [Zone5StringSlot::default(); 2];
        let mut assets = [Zone5AssetSlot::default(); 2];
        assert!(
            PreparedZone5Directory::parse(&container, BUDGET, &mut strings, &mut assets).is_err()
        );
    }
}

#[test]
fn invalid_counts_root_links_string_aliases_and_unknown_types_fail_explicitly() {
    let (source, extent) = body(&[Some(b"a")], &[(3, u32::MAX)]);
    for (offset, value, error) in [
        (0, 0x8000_0000, Zone5Error::Count),
        (8, u32::MAX, Zone5Error::Count),
        (4, 0, Zone5Error::RootLink),
        (12, 0, Zone5Error::RootLink),
        (4, 0xffff_fffe, Zone5Error::RootLink),
        (16, 1, Zone5Error::StringLink),
        (16, 0xffff_fffe, Zone5Error::StringLink),
        (22, 33, Zone5Error::AssetKind),
    ] {
        let mut bytes = source.clone();
        word(&mut bytes, offset, value);
        let mut cooked = Vec::new();
        let container = checked(&bytes, extent, &mut cooked);
        let mut strings = [Zone5StringSlot::default(); 1];
        let mut assets = [Zone5AssetSlot::default(); 1];
        assert_eq!(
            PreparedZone5Directory::parse(&container, BUDGET, &mut strings, &mut assets).err(),
            Some(error)
        );
    }
    let (mut bytes, extent) = body(&[Some(b"Private")], &[]);
    bytes[20..].fill(b'X');
    let mut cooked = Vec::new();
    let container = checked(&bytes, extent, &mut cooked);
    assert_eq!(
        PreparedZone5Directory::parse(
            &container,
            BUDGET,
            &mut [Zone5StringSlot::default(); 1],
            &mut []
        )
        .err(),
        Some(Zone5Error::Truncated)
    );
}

#[test]
fn every_host_budget_and_private_capacity_is_independent() {
    let (body, extent) = body(&[Some(b"a"), Some(b"b")], &[(3, u32::MAX)]);
    let exact = Zone5Budget {
        max_script_strings: 2,
        max_assets: 1,
        max_string_bytes: 4,
        max_prefix_bytes: 36,
    };
    let mut cooked = Vec::new();
    let container = checked(&body, extent, &mut cooked);
    let mut strings = [Zone5StringSlot::default(); 2];
    let mut assets = [Zone5AssetSlot::default(); 1];
    assert!(PreparedZone5Directory::parse(&container, exact, &mut strings, &mut assets).is_ok());
    for (budget, error) in [
        (
            Zone5Budget {
                max_script_strings: 1,
                ..exact
            },
            Zone5Error::StringCountBudget,
        ),
        (
            Zone5Budget {
                max_assets: 0,
                ..exact
            },
            Zone5Error::AssetCountBudget,
        ),
        (
            Zone5Budget {
                max_string_bytes: 3,
                ..exact
            },
            Zone5Error::StringByteBudget,
        ),
        (
            Zone5Budget {
                max_prefix_bytes: 35,
                ..exact
            },
            Zone5Error::PrefixBudget,
        ),
    ] {
        assert_eq!(
            PreparedZone5Directory::parse(&container, budget, &mut strings, &mut assets).err(),
            Some(error)
        );
    }
    for length in 0..2 {
        assert_eq!(
            PreparedZone5Directory::parse(&container, exact, &mut strings[..length], &mut assets)
                .err(),
            Some(Zone5Error::ScratchCapacity)
        );
    }
    assert_eq!(
        PreparedZone5Directory::parse(&container, exact, &mut strings, &mut []).err(),
        Some(Zone5Error::ScratchCapacity)
    );
    let mut other = Vec::new();
    let too_small = checked(&body, extent - 1, &mut other);
    assert_eq!(
        PreparedZone5Directory::parse(&too_small, exact, &mut strings, &mut assets).err(),
        Some(Zone5Error::VirtualExtent)
    );
}

#[test]
fn empty_asset_root_controls_logical_alignment_without_physical_padding() {
    let (mut bytes, extent) = body(&[Some(b"a")], &[]);
    assert_eq!((bytes.len(), extent), (22, 6));
    let mut cooked = Vec::new();
    let container = checked(&bytes, 6, &mut cooked);
    let mut strings = [Zone5StringSlot::default(); 1];
    let plan = PreparedZone5Directory::parse(&container, BUDGET, &mut strings, &mut [])
        .expect("No asset allocation means no alignment");
    assert_eq!(plan.metadata().virtual_prefix_bytes, 6);
    word(&mut bytes, 12, u32::MAX);
    let mut other = Vec::new();
    let container = checked(&bytes, 8, &mut other);
    let plan = PreparedZone5Directory::parse(&container, BUDGET, &mut strings, &mut [])
        .expect("Zero-count nonnull allocation still aligns logically");
    assert_eq!(plan.metadata().virtual_prefix_bytes, 8);
    assert_eq!(plan.metadata().body_prefix_bytes, 22);
}

#[test]
fn root_counts_alone_do_not_prove_complete_strings_or_asset_records() {
    let (mut bytes, extent) = body(&[Some(b"a")], &[(3, u32::MAX)]);
    // Keep root/minimum arrays present, but corrupt the directory's type.
    word(&mut bytes, 22, u32::MAX);
    let mut cooked = Vec::new();
    let container = checked(&bytes, extent, &mut cooked);
    assert_eq!(
        PreparedZone5Directory::counts(&container, BUDGET)
            .expect("Counts only")
            .assets,
        1
    );
    assert_eq!(
        PreparedZone5Directory::parse(
            &container,
            BUDGET,
            &mut [Zone5StringSlot::default(); 1],
            &mut [Zone5AssetSlot::default(); 1]
        )
        .err(),
        Some(Zone5Error::AssetKind)
    );
}
