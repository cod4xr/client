// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    ArchiveError, ArchiveRegistry, BUDGET, Fixture, PAYLOAD, ZipPayloadBudget, ZipPayloadError,
    declarations, image,
};

#[test]
fn owned_source_catalog_and_later_reads_survive_image_release_and_handle_closure() {
    let bytes = image(0x3610_a686);
    let fixture = Fixture::new(&bytes);
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry
        .open_local_file(&fixture.0)
        .expect("Retained original handle");
    let metadata = {
        let mut input = vec![0xa5; bytes.len() + 3];
        let copied = registry
            .read_image(info.id(), &mut input)
            .expect("Bounded source image");
        assert_eq!(copied, bytes);
        let metadata = declarations(copied)[0];
        assert_eq!(&input[bytes.len()..], &[0xa5; 3]);
        input.fill(0);
        metadata
    };
    let mut output = [0xa5; 8];
    let result = registry
        .read_entry(
            info.id(),
            metadata,
            PAYLOAD,
            &mut [0; 7],
            &mut [0; 6],
            &mut output,
        )
        .expect("Original handle seek and checked payload");
    assert!(registry.close(info.id()));
    assert_eq!(registry.open_count(), 0);
    assert_eq!(registry.usage().completed_images, 1);
    assert_eq!(registry.usage().completed_payloads, 1);
    assert_eq!(registry.usage().admitted_source_bytes, info.bytes() + 7);
    assert_eq!(registry.usage().admitted_payload_bytes, 5);
    drop(registry);
    assert_eq!(result.bytes(), b"hello");
    assert_eq!(&output[5..], &[0xa5; 3]);
}

#[test]
fn every_short_image_and_payload_capacity_preserves_final_outputs() {
    let bytes = image(0x3610_a686);
    let metadata = declarations(&bytes)[0];
    let fixture = Fixture::new(&bytes);
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry.open_local_file(&fixture.0).expect("Source");
    for length in 0..bytes.len() {
        let mut input = vec![0xa5; length];
        assert_eq!(
            registry.read_image(info.id(), &mut input).err(),
            Some(ArchiveError::SourceCapacity)
        );
        assert!(input.iter().all(|&byte| byte == 0xa5));
    }
    for (source, scratch, final_length, expected) in [
        (6, 6, 5, ArchiveError::SourceCapacity),
        (
            7,
            5,
            5,
            ArchiveError::Payload(ZipPayloadError::ScratchCapacity),
        ),
        (
            7,
            6,
            4,
            ArchiveError::Payload(ZipPayloadError::OutputCapacity),
        ),
    ] {
        let mut output = vec![0xa5; final_length];
        let before = output.clone();
        assert_eq!(
            registry
                .read_entry(
                    info.id(),
                    metadata,
                    PAYLOAD,
                    &mut vec![0; source],
                    &mut vec![0; scratch],
                    &mut output
                )
                .err(),
            Some(expected)
        );
        assert_eq!(output, before);
    }
    assert_eq!(registry.usage().completed_images, 0);
    assert_eq!(registry.usage().completed_payloads, 0);
    assert_eq!(registry.usage().admitted_source_bytes, 0);
}

#[test]
fn work_budgets_and_outside_source_spans_reject_before_payload_publication() {
    let bytes = image(0x3610_a686);
    let metadata = declarations(&bytes)[0];
    let fixture = Fixture::new(&bytes);
    let tiny = Fixture::new(b"x");
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry.open_local_file(&fixture.0).expect("Source");
    let short = registry
        .open_local_file(&tiny.0)
        .expect("Small regular source");
    for (budget, expected) in [
        (
            ZipPayloadBudget {
                max_compressed_bytes: 6,
                ..PAYLOAD
            },
            ArchiveError::Payload(ZipPayloadError::InputBudget),
        ),
        (
            ZipPayloadBudget {
                max_uncompressed_bytes: 4,
                ..PAYLOAD
            },
            ArchiveError::Payload(ZipPayloadError::OutputBudget),
        ),
    ] {
        let mut output = [0xa5; 5];
        assert_eq!(
            registry
                .read_entry(
                    info.id(),
                    metadata,
                    budget,
                    &mut [0; 7],
                    &mut [0; 6],
                    &mut output
                )
                .err(),
            Some(expected)
        );
        assert_eq!(output, [0xa5; 5]);
    }
    let mut output = [0xa5; 5];
    assert_eq!(
        registry
            .read_entry(
                short.id(),
                metadata,
                PAYLOAD,
                &mut [0; 7],
                &mut [0; 6],
                &mut output
            )
            .err(),
        Some(ArchiveError::Bounds)
    );
    assert_eq!(output, [0xa5; 5]);
    assert_eq!(registry.usage().admitted_source_bytes, 0);
}

#[test]
fn crc_rejection_leaves_output_and_admitted_byte_counters_unchanged() {
    let bytes = image(1);
    let metadata = declarations(&bytes)[0];
    let fixture = Fixture::new(&bytes);
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry.open_local_file(&fixture.0).expect("Source");
    let mut output = [0xa5; 5];
    assert_eq!(
        registry
            .read_entry(
                info.id(),
                metadata,
                PAYLOAD,
                &mut [0; 7],
                &mut [0; 6],
                &mut output
            )
            .err(),
        Some(ArchiveError::Payload(ZipPayloadError::Checksum))
    );
    assert_eq!(output, [0xa5; 5]);
    assert_eq!(registry.usage().payload_attempts, 1);
    assert_eq!(registry.usage().completed_payloads, 0);
    assert_eq!(registry.usage().admitted_source_bytes, 0);
}
