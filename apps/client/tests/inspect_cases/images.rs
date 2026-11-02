// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Fixture, archive::run_mode, fs};
#[path = "../../../../crates/content/tests/zip_cases/fixture.rs"]
mod zip_fixture;

fn pixels(version: u8, format: u8) -> Vec<u8> {
    let mut bytes = vec![0; 28];
    bytes[..4].copy_from_slice(&[b'I', b'W', b'i', version]);
    bytes[4] = format;
    bytes[5] = 2;
    let dimension = if format == 11 { 4_u16 } else { 1 };
    bytes[6..8].copy_from_slice(&dimension.to_le_bytes());
    bytes[8..10].copy_from_slice(&dimension.to_le_bytes());
    bytes[10..12].copy_from_slice(&1_u16.to_le_bytes());
    if format == 11 {
        bytes.extend_from_slice(b"Private!");
    }
    let size = u32::try_from(bytes.len()).expect("Small image");
    bytes[12..16].copy_from_slice(&size.to_le_bytes());
    bytes
}

#[test]
fn selected_images_report_direct_layouts_and_wavelet_headers_without_claiming_wavelet_pixels() {
    let direct = pixels(6, 11);
    let wavelet = pixels(6, 6);
    // CRC values were independently computed with Python zlib over these synthetic bytes.
    let source = zip_fixture::build(
        &[
            zip_fixture::Spec {
                crc: 0xe7b4_70e6,
                ..zip_fixture::Spec::stored(b"PrivateImage.IWI", &direct)
            },
            zip_fixture::Spec {
                crc: 0xdc44_d60d,
                ..zip_fixture::Spec::stored(b"PrivateWavelet.iwi", &wavelet)
            },
            zip_fixture::Spec {
                crc: 0,
                ..zip_fixture::Spec::stored(b"PrivateIgnored.txt", b"ignored bad CRC")
            },
        ],
        &[],
        &[],
    );
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, &source.bytes).expect("Synthetic images");
    for format in ["human", "json"] {
        let output = run_mode(&fixture, format, "--inspect-images");
        assert!(output.status.success());
        let report: serde_json::Value =
            serde_json::from_slice(&output.stdout).expect("Metadata report");
        assert_eq!(report["phase"], "iwi6_image_inspection");
        assert_eq!(report["entries"], 3);
        assert_eq!(report["verified_entries"], 2);
        assert_eq!(report["payload_integrity"], "selected_image_crc32_verified");
        assert_eq!(report["image_headers"], 2);
        assert_eq!(report["direct_image_layouts"], 1);
        assert_eq!(report["wavelet_headers"], 1);
        assert_eq!(report["wavelet_pixel_decoding_implemented"], false);
        assert_eq!(report["image_texels"], 17);
        assert_eq!(report["largest_image_dimension"], 4);
        let text = format!(
            "{} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!text.contains("Private"));
    }
}

#[test]
fn crc_valid_but_wrong_iwi_profile_reports_fixed_failure_without_partial_metadata() {
    let invalid = pixels(8, 11);
    let source = zip_fixture::build(
        &[zip_fixture::Spec {
            crc: 0x3d7e_9315,
            ..zip_fixture::Spec::stored(b"PrivateWrongVersion.iwi", &invalid)
        }],
        &[],
        &[],
    );
    let fixture = Fixture::new(false);
    fs::write(&fixture.0, &source.bytes).expect("Wrong image profile");
    let output = run_mode(&fixture, "json", "--inspect-images");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("Logs");
    assert!(!stderr.contains("Private"));
    assert!(stderr.contains("version-6 profile mismatch"));
    assert!(!stderr.contains("content.inspect.completed"));
    for line in stderr.lines() {
        let _: serde_json::Value = serde_json::from_str(line).expect("JSON log");
    }
}
