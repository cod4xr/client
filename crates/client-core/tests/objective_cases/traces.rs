// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{FIELDS, PROFILES, W, encoded, integer, parse, raw_float, seed};

#[test]
fn deterministic_section_traces_preserve_statuses_and_every_omitted_word() {
    for profile in PROFILES {
        let mut result = seed(profile, 0);
        let mut expected = [[0_u32; 6]; 16];
        let mut statuses =
            std::array::from_fn::<_, 16, _>(|slot| u32::try_from(slot % 8).expect("Status"));
        let mut random = 0x7129_854a_u32;
        for step in 0..128 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let old = expected;
            let old_status = statuses;
            let bytes = encoded(|writer| {
                let changed = !random.is_multiple_of(5);
                writer
                    .write_bits(u32::from(changed), 1)
                    .expect("Section flag");
                if !changed {
                    return;
                }
                for slot in 0..16 {
                    random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    statuses[slot] = (random >> 12) & 7;
                    writer.write_bits(statuses[slot], 3).expect("Status");
                    let changed = !random.is_multiple_of(3);
                    writer
                        .write_bits(u32::from(changed), 1)
                        .expect("Entry flag");
                    if !changed {
                        continue;
                    }
                    for (index, &(word, width)) in FIELDS.iter().enumerate() {
                        random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        if random.is_multiple_of(4) {
                            writer.write_bits(0, 1).expect("Unchanged word");
                        } else {
                            let value = if width == 0 {
                                random
                            } else {
                                random & (u32::MAX >> (32 - width))
                            };
                            let prior = result.get(slot).expect("Prior slot").word(word);
                            if width == 0 {
                                raw_float(writer, prior, value);
                            } else {
                                integer(writer, width, prior, value);
                            }
                            expected[slot][index] = value;
                        }
                    }
                }
            });
            let next = parse(profile, &bytes, Some(&result));
            for slot in 0..16 {
                assert_eq!(
                    u32::from(result.get(slot).expect("Old slot").status()),
                    old_status[slot]
                );
                assert_eq!(
                    u32::from(next.get(slot).expect("New slot").status()),
                    statuses[slot]
                );
                for (index, &(word, _)) in FIELDS.iter().enumerate() {
                    assert_eq!(
                        result.get(slot).expect("Old slot").word(word),
                        old[slot][index]
                    );
                    assert_eq!(
                        next.get(slot).expect("New slot").word(word),
                        expected[slot][index],
                        "step {step}"
                    );
                }
            }
            result = next;
        }
    }
}

#[test]
fn complete_owned_results_outlive_source_bytes_and_prior_sets_and_hide_values_in_debug() {
    let result = {
        let prior = seed(PROFILES[1], 0xdead_beef);
        let bytes = vec![0];
        parse(PROFILES[1], &bytes, Some(&prior))
    };
    assert_eq!(
        result.get(15).expect("Owned slot").word(W::OriginX),
        0xdead_beef
    );
    let debug = format!("{result:?} {:?}", result.get(15).expect("Debug slot"));
    for private in ["dead", "beef", "3735928559"] {
        assert!(!debug.contains(private));
    }
}
