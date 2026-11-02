// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    FORCED, PROFILES, PlayerScalars, UNCONDITIONAL, encoded, header, integer, parse, raw_float,
    word,
};

#[test]
fn modeled_prefix_traces_keep_omitted_words_and_apply_forced_and_unconditional_references() {
    for profile in PROFILES {
        let mut result = PlayerScalars::zero(profile);
        let mut expected = [0_u32; 141];
        let mut random = 0x7812_3a91_u32;
        for step in 0..128 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let count = usize::try_from((random >> 12) % 142).expect("Count");
            let origins = random & 1 != 0;
            let old = expected;
            let bytes = encoded(|writer| {
                header(writer, origins, count);
                for (index, &key) in word::ALL[..count].iter().enumerate() {
                    random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    if let Some(&(_, width)) =
                        UNCONDITIONAL.iter().find(|&&(field, _)| field == key)
                    {
                        let value = random & (u32::MAX >> (32 - width));
                        integer(writer, width, old[index], value, false);
                        expected[index] = value;
                        continue;
                    }
                    let forced = origins && FORCED.iter().any(|&(field, _)| field == key);
                    let reference = if forced { 0 } else { old[index] };
                    if key == word::motion::ORIGIN_X || key == word::effects::DOF_NEAR_BLUR {
                        raw_float(
                            writer,
                            reference,
                            random,
                            key == word::effects::DOF_NEAR_BLUR,
                        );
                        expected[index] = random;
                    } else if key == word::weapons::ADS_DELAY_TIME {
                        integer(writer, 32, reference, random, true);
                        expected[index] = random;
                    } else if key == word::events::EVENT_0 {
                        let value = random & 255;
                        integer(writer, 8, reference, value, true);
                        expected[index] = value;
                    } else {
                        writer.write_bits(0, 1).expect("Unchanged");
                        expected[index] = reference;
                    }
                }
            });
            let next = parse(profile, 0, &bytes, Some(&result)).into_scalars();
            for (index, &key) in word::ALL.iter().enumerate() {
                assert_eq!(result.word(key), old[index], "prior step {step}");
                assert_eq!(next.word(key), expected[index], "next step {step}");
            }
            result = next;
        }
    }
}
