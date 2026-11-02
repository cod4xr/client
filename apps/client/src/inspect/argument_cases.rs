// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(clippy::expect_used, reason = "Finite native argument fixtures.")]
use super::parse;
use cod4x_observability::ConsoleFormat;
use std::{ffi::OsString, path::PathBuf};

#[test]
fn explicit_paths_with_spaces_and_named_formats_preserve_native_path_ownership() {
    let path = OsString::from("Game files/iw3mp.exe");
    let options = parse(vec![path.clone(), "--log-format".into(), "json".into()])
        .expect("Arguments")
        .expect("Image path");
    assert_eq!(options.path, PathBuf::from(path));
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
}
