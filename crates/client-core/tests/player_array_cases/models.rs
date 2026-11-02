// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReader, PROFILES, WeaponModels, encoded, parse_models};

#[test]
fn changed_models_replace_all_raw_bytes_including_zero_and_high_values() {
    for profile in PROFILES {
        let values = std::array::from_fn::<_, 128, _>(|i| u8::try_from(i * 2).expect("Byte"));
        let mut bytes = vec![1];
        bytes.extend(values);
        let result = parse_models(profile, &bytes, None);
        assert_eq!(result.values(), &values);
        assert_eq!(result.profile(), profile);
        assert_eq!(result.get(127), Some(254));
        assert_eq!(result.get(128), None);
        let mut replacement = vec![1];
        replacement.extend([0xff; 128]);
        assert_eq!(
            parse_models(profile, &replacement, Some(&result)).values(),
            &[0xff; 128]
        );
    }
}

#[test]
fn unchanged_tables_keep_prior_or_explicit_zero_without_reading_payload_bytes() {
    let mut bytes = vec![1];
    bytes.extend([0xff; 128]);
    for profile in PROFILES {
        let prior = parse_models(profile, &bytes, None);
        assert_eq!(parse_models(profile, &[0], Some(&prior)), prior);
        assert_eq!(
            parse_models(profile, &[0], None),
            WeaponModels::zero(profile)
        );
    }
}

#[test]
fn model_payloads_leave_cached_framing_and_following_bytes_available() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        writer.write_bits(1, 1).expect("Models changed");
        for _ in 0..128 {
            writer.write_u8(0xfe).expect("Raw model byte");
        }
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    assert_eq!(
        WeaponModels::parse(&mut reader, PROFILES[1], None)
            .expect("Models")
            .values(),
        &[0xfe; 128]
    );
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
