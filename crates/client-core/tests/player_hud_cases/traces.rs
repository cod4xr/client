// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{FIELDS, MODERN, SnapshotHud, W, array, encoded, integer, parse, raw_float};

#[test]
fn modeled_array_traces_match_prefix_updates_and_stop_at_first_free_tail_rules() {
    let mut result = SnapshotHud::zero(MODERN);
    let mut expected = [[[0_u32; 3]; 31]; 2]; // kind, width, from-X raw bits.
    let mut random = 0x7182_abd9_u32;
    for step in 0..128 {
        random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let changed = !random.is_multiple_of(7);
        let mut counts = [0; 2];
        let mut updates = [[[None; 3]; 31]; 2];
        if changed {
            for group in 0..2 {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                counts[group] = usize::try_from((random >> 11) % 32).expect("Count");
                for change in updates[group].iter_mut().take(counts[group]) {
                    for (field, target) in change.iter_mut().enumerate() {
                        random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                        if !random.is_multiple_of(4) {
                            let value = match field {
                                0 => (random >> 9) & 3,
                                1 => random & 1023,
                                _ => random,
                            };
                            *target = Some(value);
                        }
                    }
                }
            }
        }
        let bytes = encoded(|writer| {
            writer
                .write_bits(u32::from(changed), 1)
                .expect("Section flag");
            if !changed {
                return;
            }
            for group in 0..2 {
                array(
                    writer,
                    u32::try_from(counts[group]).expect("Count"),
                    39,
                    |slot, index, writer| {
                        let mapped = match FIELDS[index].0 {
                            W::Kind => Some(0),
                            W::Width => Some(1),
                            W::FromX => Some(2),
                            _ => None,
                        };
                        if let Some(field) = mapped
                            && let Some(value) = updates[group][slot][field]
                        {
                            if field == 2 {
                                raw_float(writer, expected[group][slot][field], value);
                            } else {
                                integer(
                                    writer,
                                    if field == 0 { 4 } else { 10 },
                                    expected[group][slot][field],
                                    value,
                                );
                            }
                            return;
                        }
                        writer.write_bits(0, 1).expect("Unchanged field");
                    },
                );
            }
        });
        if changed {
            for group in 0..2 {
                for slot in 0..counts[group] {
                    for field in 0..3 {
                        if let Some(value) = updates[group][slot][field] {
                            expected[group][slot][field] = value;
                        }
                    }
                }
                for retained in expected[group].iter_mut().skip(counts[group]) {
                    if retained[0] == 0 {
                        break;
                    }
                    *retained = [0; 3];
                }
            }
        }
        result = parse(MODERN, 0, &bytes, Some(&result));
        for (group, actual) in [result.archival(), result.current()]
            .into_iter()
            .enumerate()
        {
            for slot in 0..31 {
                for (field, word) in [W::Kind, W::Width, W::FromX].into_iter().enumerate() {
                    assert_eq!(
                        actual[slot].word(word),
                        expected[group][slot][field],
                        "step {step} group {group} slot {slot}"
                    );
                }
            }
        }
    }
}
