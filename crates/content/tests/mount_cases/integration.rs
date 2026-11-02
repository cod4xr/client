// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    ArchiveId, AssetKey, BUDGET, KEY_BUDGET, MountCandidate, MountIndex, MountIndexSlot,
    MountScratchSlot, Zip32Budget, Zip32Directory, Zip32EntrySlot, fixture,
};
use cod4xr_content::zip_payload::{ZipPayloadBudget, ZipPayloadDecoder};
use std::collections::BTreeMap;

#[test]
fn catalog_to_copied_lookup_selects_and_decodes_native_last_duplicate_after_source_release() {
    let mut final_keys = [0; 64];
    let mut final_slots = [MountIndexSlot::default()];
    let (index, retained_payloads) = {
        let source = fixture::build(
            &[
                fixture::Spec {
                    crc: 0,
                    ..fixture::Spec::stored(b"IMAGES\\Private.IWI", b"")
                },
                fixture::Spec {
                    crc: 0x3610_a686,
                    ..fixture::Spec::stored(b"images/private.iwi", b"hello")
                },
            ],
            &[],
            &[],
        );
        let mut zip_slots = [Zip32EntrySlot::default(); 2];
        let catalog = Zip32Directory::parse(
            &source.bytes,
            Zip32Budget {
                max_archive_bytes: 1024,
                max_directory_bytes: 1024,
                max_entries: 2,
                max_name_bytes: 64,
                max_total_name_bytes: 128,
                max_entry_uncompressed_bytes: 64,
                max_total_uncompressed_bytes: 128,
            },
            &mut [Zip32EntrySlot::default(); 2],
            &mut zip_slots,
        )
        .expect("Real duplicate catalog");
        let retained: BTreeMap<_, _> = catalog
            .entries()
            .map(|entry| {
                (
                    entry.metadata().data_offset(),
                    entry.compressed_bytes().to_vec(),
                )
            })
            .collect();
        let mut normalized = vec![vec![0; 64]; 2];
        let inputs: Vec<_> = catalog
            .entries()
            .zip(&mut normalized)
            .map(|(entry, bytes)| {
                let key = AssetKey::parse(entry.name(), KEY_BUDGET, bytes)
                    .expect("Native equivalent key");
                MountCandidate::new(key, ArchiveId::new(u64::MAX), entry.metadata())
            })
            .collect();
        let index = MountIndex::build(
            &inputs,
            BUDGET,
            &mut [MountScratchSlot::default(); 2],
            &mut final_keys,
            &mut final_slots,
        )
        .expect("Copied index");
        (index, retained)
    };
    assert_eq!(index.stats().files, 1);
    assert_eq!(index.stats().shadowed, 1);
    let mut query = [0; 64];
    let key = AssetKey::parse(b"Images/Private.IWI", KEY_BUDGET, &mut query).expect("Lookup");
    let asset = index.lookup(key).expect("Last native duplicate");
    assert_eq!(asset.archive().value(), u64::MAX);
    let metadata = asset.metadata();
    let input = retained_payloads
        .get(&metadata.data_offset())
        .expect("Independent bounded source");
    let mut output = [0; 5];
    let payload = ZipPayloadDecoder::new()
        .decode_bytes(
            metadata,
            input,
            ZipPayloadBudget {
                max_compressed_bytes: 5,
                max_uncompressed_bytes: 5,
            },
            &mut [],
            &mut output,
        )
        .expect("Selected payload length and CRC");
    assert_eq!(payload.bytes(), b"hello");
}
