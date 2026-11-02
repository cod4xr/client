// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveError, ArchiveRegistry, BUDGET, Fixture, PAYLOAD, declarations, fs, image};

#[test]
fn replaced_pathname_does_not_replace_the_original_retained_file_handle() {
    let bytes = image(0x3610_a686);
    let metadata = declarations(&bytes)[0];
    let fixture = Fixture::new(&bytes);
    let moved = Fixture::new(b"");
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry
        .open_local_file(&fixture.0)
        .expect("Original handle");
    fs::rename(&fixture.0, &moved.0).expect("Rename original over exclusively owned destination");
    fs::write(&fixture.0, b"PrivateReplacement").expect("Different file at original path");
    let mut output = [0; 5];
    let result = registry
        .read_entry(
            info.id(),
            metadata,
            PAYLOAD,
            &mut [0; 7],
            &mut [0; 6],
            &mut output,
        )
        .expect("Read original handle, not replaced pathname");
    assert_eq!(result.bytes(), b"hello");
}

#[test]
fn observable_in_place_source_change_rejects_without_final_payload_writes() {
    let bytes = image(0x3610_a686);
    let metadata = declarations(&bytes)[0];
    let fixture = Fixture::new(&bytes);
    let mut registry = ArchiveRegistry::new(BUDGET);
    let info = registry
        .open_local_file(&fixture.0)
        .expect("Original handle");
    fs::write(&fixture.0, b"changed").expect("Observable length change");
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
        Some(ArchiveError::Changed)
    );
    assert_eq!(output, [0xa5; 5]);
    assert_eq!(registry.usage().admitted_source_bytes, 0);
    // Resource preflight occurs before asking the changed file for bytes.
    assert_eq!(
        registry
            .read_entry(
                info.id(),
                metadata,
                PAYLOAD,
                &mut [0; 7],
                &mut [0; 6],
                &mut []
            )
            .err(),
        Some(ArchiveError::Payload(
            crate::ZipPayloadError::OutputCapacity
        ))
    );
}
