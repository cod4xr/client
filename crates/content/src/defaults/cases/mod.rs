// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Finite synthetic image/profile fixtures."
)]
mod bounds;
mod layout;

use super::{DefaultSlot, DefaultsBudget, ImportedDefaults, profile::Profile};
use sha2::{Digest, Sha256};

const BUDGET: DefaultsBudget = DefaultsBudget {
    max_image_bytes: 1024 * 1024,
    max_sections: 4,
    max_records: 1024,
    max_text_bytes: 64 * 1024,
    max_value_bytes: 16 * 1024,
};
fn word(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
fn rehash(bytes: &[u8], profile: &mut Profile) {
    profile.digest.copy_from_slice(&Sha256::digest(bytes));
}
fn fixture(entries: &[(u32, &[u8])]) -> (Vec<u8>, Profile) {
    let text_bytes: usize = entries.iter().map(|(_, value)| value.len()).sum();
    let raw_size = (0x200 + text_bytes + entries.len()).next_multiple_of(0x200);
    let mut bytes = vec![0; 0x200 + raw_size];
    bytes[..2].copy_from_slice(b"MZ");
    word(&mut bytes, 0x3c, 0x80);
    bytes[0x80..0x84].copy_from_slice(b"PE\0\0");
    for (offset, value) in [(0x84, 0x14c_u16), (0x86, 1), (0x94, 224), (0x98, 0x10b)] {
        bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
    }
    word(&mut bytes, 0x98 + 28, 0x0040_0000);
    word(
        &mut bytes,
        0x98 + 56,
        u32::try_from((0x1000 + raw_size).next_multiple_of(0x1000)).expect("Image size"),
    );
    word(&mut bytes, 0x98 + 60, 0x200);
    word(&mut bytes, 0x98 + 92, 16);
    word(
        &mut bytes,
        0x178 + 8,
        u32::try_from(raw_size).expect("Virtual span"),
    );
    word(&mut bytes, 0x178 + 12, 0x1000);
    word(
        &mut bytes,
        0x178 + 16,
        u32::try_from(raw_size).expect("Raw span"),
    );
    word(&mut bytes, 0x178 + 20, 0x200);
    let mut offset = 0x400;
    for (position, &(index, value)) in entries.iter().enumerate() {
        word(&mut bytes, 0x200 + position * 16, index);
        word(
            &mut bytes,
            0x204 + position * 16,
            0x0040_1000 + u32::try_from(offset - 0x200).expect("String VA"),
        );
        bytes[offset..offset + value.len()].copy_from_slice(value);
        offset += value.len() + 1;
    }
    let mut profile = Profile {
        digest: [0; 32],
        image_bytes: bytes.len(),
        image_base: 0x0040_0000,
        table_address: 0x0040_1000,
        records: entries.len(),
        text_bytes,
        longest_value: entries
            .iter()
            .map(|(_, value)| value.len())
            .max()
            .unwrap_or(0),
        last_index: entries.last().map_or(0, |&(index, _)| index),
        index_limit: 2442,
    };
    rehash(&bytes, &mut profile);
    (bytes, profile)
}
fn standard() -> (Vec<u8>, Profile) {
    fixture(&[(12, b"one"), (100, b""), (2273, b"Private%\x92")])
}
fn dirty(count: usize) -> Vec<DefaultSlot> {
    vec![
        DefaultSlot {
            index: 2001,
            offset: 3,
            length: 1
        };
        count
    ]
}

#[test]
fn complete_import_preserves_raw_values_unused_tails_and_source_independent_ownership() {
    let mut output = [0xa5; 14];
    let mut slots = dirty(4);
    let unused = slots[3];
    let plan = {
        let (bytes, profile) = standard();
        ImportedDefaults::with_profile(&bytes, BUDGET, &mut output, &mut slots, profile)
            .expect("Private reviewed synthetic profile")
    };
    assert_eq!(plan.record_count(), 3);
    assert_eq!(plan.text_bytes(), 12);
    assert_eq!(plan.maximum_index(), Some(2273));
    let entries: Vec<_> = plan.records().collect();
    assert_eq!((entries[0].index(), entries[0].bytes()), (12, &b"one"[..]));
    assert!(entries[1].bytes().is_empty());
    assert_eq!(entries[2].bytes(), b"Private%\x92");
    let borrowed_values: Vec<_> = plan.records().map(|entry| entry.bytes()).collect();
    assert_eq!(borrowed_values[2], b"Private%\x92");
    assert!(!format!("{plan:?} {:?}", entries[2]).contains("Private"));
    assert_eq!(&output[12..], &[0xa5; 2]);
    assert_eq!(slots[3], unused);
    let (bytes, profile) = fixture(&[(12, b"")]);
    let mut slots = dirty(1);
    let empty = ImportedDefaults::with_profile(
        &bytes,
        DefaultsBudget {
            max_text_bytes: 0,
            max_value_bytes: 0,
            ..BUDGET
        },
        &mut [],
        &mut slots,
        profile,
    )
    .expect("Empty default needs only its source NUL");
    assert_eq!(empty.text_bytes(), 0);
    assert!(
        empty
            .records()
            .next()
            .expect("Empty record")
            .bytes()
            .is_empty()
    );
}
