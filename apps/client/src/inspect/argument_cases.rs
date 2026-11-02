// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Finite native argument fixtures.")]
use super::{Kind, parse};
use cod4x_observability::ConsoleFormat;
use std::{ffi::OsString, path::PathBuf};

#[test]
fn explicit_paths_with_spaces_and_named_formats_preserve_native_path_ownership() {
    let path = OsString::from("Game files/iw3mp.exe");
    let options = parse(vec![path.clone(), "--log-format".into(), "json".into()])
        .expect("Arguments")
        .expect("Image path");
    assert_eq!(options.path, PathBuf::from(path));
    assert_eq!(options.kind, Kind::ClientImage);
    assert!(matches!(options.logs, ConsoleFormat::Json));
    assert!(parse(vec![]).expect("No arguments").is_none());
    assert!(parse(vec!["--help".into()]).expect("Help").is_none());
}

#[test]
fn duplicate_excessive_unknown_or_missing_arguments_reject_without_echoing_input() {
    for values in [
        vec![""],
        vec!["--help", "PrivatePath"],
        vec!["image", "--log-format"],
        vec!["image", "--unknown", "PrivateValue"],
        vec!["image", "--log-format", "PrivateValue"],
        vec!["image", "--log-format", "json", "--log-format", "human"],
        vec!["--archive"],
        vec!["--archive", ""],
        vec!["--archive", "image", "--log-format"],
        vec!["--archive", "image", "--log-format", "json", "extra"],
        vec!["--verify-archive"],
        vec!["--verify-archive", ""],
        vec!["--verify-archive", "image", "--log-format", "json", "extra"],
        vec!["--inspect-images"],
        vec!["--inspect-images", ""],
        vec!["--inspect-images", "image", "--log-format", "json", "extra"],
        vec!["--inspect-fastfile"],
        vec!["--inspect-zone-directory"],
        vec!["--inspect-zone-directory", ""],
        vec![
            "--inspect-zone-directory",
            "image",
            "--log-format",
            "json",
            "extra",
        ],
        vec!["--inspect-fastfile", ""],
        vec![
            "--inspect-fastfile",
            "image",
            "--log-format",
            "json",
            "extra",
        ],
    ] {
        let result = parse(values.into_iter().map(OsString::from).collect());
        assert!(result.is_err());
        if let Err(error) = result {
            assert!(!format!("{error:?} {error}").contains("Private"));
        }
    }
}

#[test]
#[cfg(any(unix, windows))]
fn non_unicode_os_paths_are_preserved_instead_of_lossily_converted() {
    #[cfg(windows)]
    let path = {
        use std::os::windows::ffi::OsStringExt;
        OsString::from_wide(&[0xd800, 0x2e, 0x65, 0x78, 0x65])
    };
    #[cfg(unix)]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        OsString::from_vec(vec![0xff, b'.', b'e', b'x', b'e'])
    };
    assert!(path.to_str().is_none());
    let options = parse(vec![path.clone()])
        .expect("Native path")
        .expect("Inspection");
    assert_eq!(options.path.into_os_string(), path);
    for (flag, kind) in [
        ("--archive", Kind::Archive),
        ("--verify-archive", Kind::VerifiedArchive),
        ("--inspect-images", Kind::Images),
        ("--inspect-fastfile", Kind::Fastfile),
        ("--inspect-zone-directory", Kind::ZoneDirectory),
    ] {
        let options = parse(vec![flag.into(), path.clone()])
            .expect("Native archive path")
            .expect("Inspection");
        assert_eq!(options.kind, kind);
        assert_eq!(options.path.into_os_string(), path);
    }
}

#[test]
fn explicit_archive_mode_preserves_native_path_and_log_format() {
    let path = OsString::from("Game files/custom.iwd");
    let options = parse(vec![
        "--archive".into(),
        path.clone(),
        "--log-format".into(),
        "json".into(),
    ])
    .expect("Archive mode")
    .expect("Options");
    assert_eq!(options.kind, Kind::Archive);
    assert_eq!(options.path, PathBuf::from(path));
    assert!(matches!(options.logs, ConsoleFormat::Json));
}
