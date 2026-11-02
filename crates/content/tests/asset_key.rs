// SPDX-License-Identifier: AGPL-3.0-or-later
//! Strict virtual keys, byte fidelity, stable lookup semantics and atomic publication.
#![allow(clippy::expect_used, reason = "Finite virtual-key vectors.")]
use cod4xr_content::asset_key::{AssetKey, AssetKeyBudget, AssetKeyError};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

const BUDGET: AssetKeyBudget = AssetKeyBudget {
    max_bytes: 1024 * 1024,
    max_components: 1024,
};

#[test]
fn native_ascii_case_and_separator_equivalence_produce_the_same_virtual_key() {
    let mut first = [0xa5; 32];
    let mut second = [0xa5; 32];
    let a = AssetKey::parse(b"IMAGES\\Private%\x92.IWI", BUDGET, &mut first).expect("Raw key");
    let b =
        AssetKey::parse(b"images/private%\x92.iwi", BUDGET, &mut second).expect("Same virtual key");
    assert_eq!(a, b);
    assert_eq!(a.bytes(), b"images/private%\x92.iwi");
    assert_eq!(a.component_count(), 2);
    let mut ah = DefaultHasher::new();
    let mut bh = DefaultHasher::new();
    a.hash(&mut ah);
    b.hash(&mut bh);
    assert_eq!(ah.finish(), bh.finish());
    assert!(!format!("{a:?}").contains("private"));
    assert!(!format!("{a:?}").contains("images"));
    assert!(first[20..].iter().all(|&byte| byte == 0xa5));
}

#[test]
fn invalid_relative_file_names_fail_without_rewriting_any_output() {
    for (bytes, expected) in [
        (&b""[..], AssetKeyError::Empty),
        (b"/root", AssetKeyError::Rooted),
        (b"\\root", AssetKeyError::Rooted),
        (b"file/", AssetKeyError::Directory),
        (b"file\\", AssetKeyError::Directory),
        (b"a//b", AssetKeyError::EmptyComponent),
        (b"a\\/b", AssetKeyError::EmptyComponent),
        (b".", AssetKeyError::DotComponent),
        (b"a/./b", AssetKeyError::DotComponent),
        (b"a/../b", AssetKeyError::DotComponent),
        (b"file..ext", AssetKeyError::DotComponent),
        (b"C:/file", AssetKeyError::Character),
        (b"a:b", AssetKeyError::Character),
        (b"a\0b", AssetKeyError::Character),
    ] {
        let mut output = [0xa5; 64];
        assert_eq!(
            AssetKey::parse(bytes, BUDGET, &mut output).err(),
            Some(expected)
        );
        assert_eq!(output, [0xa5; 64]);
        assert!(!format!("{expected:?}: {expected}").contains("file..ext"));
    }
    for byte in (0..=31).chain([127]) {
        let mut output = [0xa5; 4];
        assert_eq!(
            AssetKey::parse(&[b'a', b'/', byte], BUDGET, &mut output).err(),
            Some(AssetKeyError::Character)
        );
        assert_eq!(output, [0xa5; 4]);
    }
}

#[test]
fn budgets_and_every_short_output_are_exact_without_native_path_array_limits() {
    let input = b"folder/file";
    for capacity in 0..input.len() {
        let mut output = vec![0xa5; capacity];
        assert_eq!(
            AssetKey::parse(input, BUDGET, &mut output).err(),
            Some(AssetKeyError::OutputCapacity)
        );
        assert!(output.iter().all(|&byte| byte == 0xa5));
    }
    for (budget, expected) in [
        (
            AssetKeyBudget {
                max_bytes: input.len() - 1,
                ..BUDGET
            },
            AssetKeyError::ByteBudget,
        ),
        (
            AssetKeyBudget {
                max_components: 1,
                ..BUDGET
            },
            AssetKeyError::ComponentBudget,
        ),
        (
            AssetKeyBudget {
                max_components: 0,
                ..BUDGET
            },
            AssetKeyError::ComponentBudget,
        ),
    ] {
        let mut output = [0xa5; 11];
        assert_eq!(
            AssetKey::parse(input, budget, &mut output).err(),
            Some(expected)
        );
        assert_eq!(output, [0xa5; 11]);
    }
    let input = vec![b'A'; 65_536];
    let mut output = vec![0xa5; input.len()];
    let key = AssetKey::parse(
        &input,
        AssetKeyBudget {
            max_bytes: input.len(),
            max_components: 1,
        },
        &mut output,
    )
    .expect("Host-funded key beyond native fixed arrays");
    assert_eq!(key.bytes(), vec![b'a'; input.len()]);
}

#[test]
fn cooked_key_outlives_input_and_unicode_bytes_are_never_lossily_converted() {
    let mut output = [0; 32];
    let key = {
        let source = "目录/Ä%File".as_bytes().to_vec();
        AssetKey::parse(&source, BUDGET, &mut output).expect("UTF8 retained as bytes")
    };
    assert_eq!(key.bytes(), "目录/Ä%file".as_bytes());
    // These remain virtual keys only; OS device names and Unicode case folding
    // require separate policy if a future host resolves loose physical files.
    let mut other = [0; 8];
    assert_eq!(
        AssetKey::parse(b"CON", BUDGET, &mut other)
            .expect("Virtual bytes only")
            .bytes(),
        b"con"
    );
}

#[test]
fn deterministic_byte_mutations_preserve_atomic_output_and_canonical_invariants() {
    let original = b"Folder/File";
    for offset in 0..original.len() {
        for byte in 0..=255 {
            let mut input = *original;
            input[offset] = byte;
            let mut output = [0xa5; 16];
            match AssetKey::parse(&input, BUDGET, &mut output) {
                Ok(key) => {
                    assert_eq!(key.bytes().len(), input.len());
                    assert!(!key.bytes().contains(&b'\\'));
                    assert!(!key.bytes().iter().any(u8::is_ascii_uppercase));
                    assert!(key.component_count() > 0);
                }
                Err(_) => assert_eq!(output, [0xa5; 16]),
            }
            assert_eq!(&output[input.len()..], &[0xa5; 5]);
        }
    }
}
