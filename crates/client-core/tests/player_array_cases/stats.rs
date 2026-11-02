// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BodyReader, PROFILES, PlayerStats, STAT_VECTOR, encoded, parse_stats, write_stats};

#[test]
fn independent_physical_stats_vector_covers_signed_shorts_six_bits_and_unsigned_byte() {
    for profile in PROFILES {
        let result = parse_stats(profile, &STAT_VECTOR, None);
        assert_eq!(result.values(), &[-32768, -1, 32767, 63, 255]);
        assert_eq!(result.profile(), profile);
        assert_eq!(result.get(0), Some(-32768));
        assert_eq!(result.get(5), None);
        let encoded = encoded(|writer| write_stats(writer, 31, &[-32768, -1, 32767, 63, 255]));
        assert_eq!(encoded, STAT_VECTOR);
    }
}

#[test]
fn each_stat_mask_retains_omitted_prior_words_and_updates_absolutely() {
    for profile in PROFILES {
        let prior = parse_stats(profile, &STAT_VECTOR, None);
        for slot in 0..5 {
            let values = [17, -1234, 0, 9, 7];
            let bytes = encoded(|writer| write_stats(writer, 1 << slot, &values));
            let result = parse_stats(profile, &bytes, Some(&prior));
            for (index, &value) in result.values().iter().enumerate() {
                assert_eq!(
                    value,
                    if index == slot {
                        values[index]
                    } else {
                        prior.values()[index]
                    }
                );
            }
        }
    }
}

#[test]
fn unchanged_and_explicit_zero_masks_retain_every_prior_stat() {
    for profile in PROFILES {
        let prior = parse_stats(profile, &STAT_VECTOR, None);
        for bytes in [&[0][..], &[1][..]] {
            assert_eq!(parse_stats(profile, bytes, Some(&prior)), prior);
            assert_eq!(
                parse_stats(profile, bytes, None),
                PlayerStats::zero(profile)
            );
        }
    }
}

#[test]
fn stats_cached_bits_and_following_bytes_remain_distinct() {
    let bytes = encoded(|writer| {
        writer.write_bits(5, 3).expect("Prefix");
        write_stats(writer, 31, &[-32768, -1, 32767, 63, 255]);
        writer.write_u8(0xa7).expect("Following byte");
        writer.write_bits(3, 2).expect("Following bits");
    });
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
    assert_eq!(reader.read_bits(3).expect("Prefix"), 5);
    assert_eq!(
        PlayerStats::parse(&mut reader, PROFILES[1], None)
            .expect("Stats")
            .values(),
        &[-32768, -1, 32767, 63, 255]
    );
    assert_eq!(reader.read_u8().expect("Following byte"), 0xa7);
    assert_eq!(reader.read_bits(2).expect("Following bits"), 3);
}
