// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ArchiveBudget, ArchiveError, ArchiveRegistry, BUDGET, Fixture, fs, image};

#[test]
fn exact_handle_budgets_and_file_limits_reject_before_retaining_resources() {
    let bytes = image(0x3610_a686);
    let fixture = Fixture::new(&bytes);
    let exact = ArchiveBudget {
        max_open_archives: 1,
        max_archive_bytes: u64::try_from(bytes.len()).expect("Small file"),
    };
    let mut zero = ArchiveRegistry::new(ArchiveBudget {
        max_open_archives: 0,
        ..exact
    });
    assert_eq!(
        zero.open_local_file(&fixture.0),
        Err(ArchiveError::OpenBudget)
    );
    let mut short = ArchiveRegistry::new(ArchiveBudget {
        max_archive_bytes: exact.max_archive_bytes - 1,
        ..exact
    });
    assert_eq!(
        short.open_local_file(&fixture.0),
        Err(ArchiveError::FileBudget)
    );
    assert_eq!(short.open_count(), 0);
    let mut registry = ArchiveRegistry::new(exact);
    let info = registry
        .open_local_file(&fixture.0)
        .expect("Exact source budget");
    assert_eq!(info.bytes(), exact.max_archive_bytes);
    assert_eq!(registry.info(info.id()), Some(info));
    assert_eq!(
        registry.open_local_file(&fixture.0),
        Err(ArchiveError::OpenBudget)
    );
    assert_eq!(registry.usage().open_attempts, 2);
    assert_eq!(registry.usage().opened, 1);
    assert_eq!(registry.open_count(), 1);
    assert_eq!(registry.budget(), exact);
}

#[test]
fn closed_reopened_and_foreign_registry_epochs_never_alias_retained_handles() {
    let fixture = Fixture::new(&image(0x3610_a686));
    let mut first = ArchiveRegistry::new(BUDGET);
    let mut second = ArchiveRegistry::new(BUDGET);
    let original = first.open_local_file(&fixture.0).expect("First owner");
    let foreign = second.open_local_file(&fixture.0).expect("Second owner");
    assert_ne!(original.id(), foreign.id());
    assert_eq!(second.info(original.id()), None);
    assert_eq!(
        second.read_image(original.id(), &mut []).err(),
        Some(ArchiveError::Missing)
    );
    assert!(first.close(original.id()));
    assert!(!first.close(original.id()));
    let reopened = first.open_local_file(&fixture.0).expect("New epoch");
    assert_ne!(original.id(), reopened.id());
    assert_eq!(
        first.read_image(original.id(), &mut []).err(),
        Some(ArchiveError::Missing)
    );
    assert_eq!(first.open_count(), 1);
    assert_eq!(first.usage().closed, 1);
}

#[test]
fn nonregular_or_missing_paths_and_debug_never_expose_source_names() {
    let fixture = Fixture::new(b"raw");
    let mut registry = ArchiveRegistry::new(BUDGET);
    registry
        .open_local_file(&fixture.0)
        .expect("Original handle");
    assert!(!format!("{registry:?}").contains("Private"));
    let missing = fixture.0.with_extension("PrivateMissing");
    let error = registry
        .open_local_file(&missing)
        .expect_err("Missing source");
    assert!(!format!("{error:?}: {error}").contains("Private"));
    assert!(matches!(error, ArchiveError::Io { .. }));
    assert!(registry.open_local_file(&std::env::temp_dir()).is_err());
    assert_eq!(registry.open_count(), 1);
    assert!(fs::metadata(&fixture.0).expect("Fixture remains").is_file());
}
