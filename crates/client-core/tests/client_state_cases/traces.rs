// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, ClientStateDecoder, ClientStateDelta, PROFILES, W, WORDS, decode, encoded, header,
    integer, present, raw_float, reference,
};

#[test]
fn all_sixty_four_clients_use_increment_shortcuts_and_a_separate_end_bit() {
    for profile in PROFILES {
        let bytes = encoded(|writer| {
            for _ in 0..64 {
                writer
                    .write_bits(3, 4)
                    .expect("More, increment, present, no delta");
            }
            writer.write_bits(0, 1).expect("End");
            writer.write_u8(0x93).expect("Next byte");
        });
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let mut owner = ClientStateDecoder::new(profile);
        for number in 0..64 {
            assert_eq!(
                present(
                    owner
                        .prepare(&mut reader)
                        .expect("Header")
                        .expect("Client")
                        .decode(None)
                        .expect("State")
                )
                .number(),
                number
            );
        }
        assert!(owner.prepare(&mut reader).expect("End").is_none());
        assert_eq!(reader.read_u8().expect("Next byte"), 0x93);
        owner.reset();
        let mut first = BodyReader::try_new(&[3], 1).expect("New stream");
        assert_eq!(
            present(
                owner
                    .prepare(&mut first)
                    .expect("Header")
                    .expect("Client")
                    .decode(None)
                    .expect("State")
            )
            .number(),
            0
        );
    }
}

#[test]
fn decoded_results_outlive_original_message_reference_and_decoder() {
    let state = {
        let prior = reference(7, 0x89ab_cdef);
        let bytes = encoded(|writer| {
            header(writer, 7, Some(1));
            integer(writer, 9, prior.word(W::Model), 17);
        });
        decode(PROFILES[1], &bytes, Some(&prior))
    };
    assert_eq!(state.number(), 7);
    assert_eq!(state.word(W::Model), 17);
    assert_eq!(state.word(W::Perks), 0x89ab_cdef);
}

#[test]
fn deterministic_prefix_traces_retain_omitted_fields_against_an_independent_model() {
    for profile in PROFILES {
        let mut state = reference(9, 0);
        let mut expected = [0; 24];
        let mut random = 0x7812_391d_u32;
        for step in 0..128 {
            random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let count = usize::try_from(random % 25).expect("Count");
            let old = expected;
            let bytes = encoded(|writer| {
                header(writer, 9, Some(u32::try_from(count).expect("Prefix")));
                for (index, &(word, width)) in WORDS[..count].iter().enumerate() {
                    random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                    if random.is_multiple_of(4) {
                        writer.write_bits(0, 1).expect("Unchanged");
                    } else {
                        let value = if width == 0 {
                            random
                        } else {
                            random & (u32::MAX >> (32 - width))
                        };
                        expected[index] = value;
                        if width == 0 {
                            raw_float(writer, state.word(word), value);
                        } else {
                            integer(writer, width, state.word(word), value);
                        }
                    }
                }
            });
            let next = decode(profile, &bytes, Some(&state));
            for (index, &(word, _)) in WORDS.iter().enumerate() {
                assert_eq!(state.word(word), old[index], "prior step {step}");
                assert_eq!(next.word(word), expected[index], "next step {step}");
            }
            state = next;
        }
    }
}

#[test]
fn removal_uses_only_index_and_flag_and_debug_hides_prior_content() {
    let prior = reference(5, 0xdead_beef);
    let mut owner = ClientStateDecoder::new(PROFILES[1]);
    let mut reader = BodyReader::try_new(&[0x15, 1], 2).expect("Removal body");
    let prepared = owner.prepare(&mut reader).expect("Header").expect("Record");
    assert!(!format!("{prepared:?}").contains("dead"));
    assert_eq!(
        prepared.decode(Some(&prior)),
        Ok(ClientStateDelta::Removed { number: 5 })
    );
    assert_eq!(reader.remaining_cached_bits(), 7);
    assert!(!format!("{owner:?}").contains("dead"));
}
