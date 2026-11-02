// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native configstring block vectors, namespace bounds and transactional storage.
#![allow(clippy::expect_used, reason = "Finite checked native field fixtures.")]

#[path = "configstring_cases/limits.rs"]
mod limits;
#[path = "configstring_cases/transactions.rs"]
mod transactions;
#[path = "configstring_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::configstrings::{ConfigStringBlockPlan, ConfigStringBudget, ConfigStringSlot},
    message_body::BodyReader,
};

const BUDGET: ConfigStringBudget = ConfigStringBudget {
    max_records: 4884,
    max_text_bytes: usize::MAX,
};
const LEGACY_VECTOR: &[u8] = &[
    2, 0, 0x45, b'o', b'n', b'e', 0, 0xc0, b't', b'w', b'o', 0, 7,
];

fn state(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}

fn block(profile: BaselineProtocol, entries: &[(i32, &[u8])]) -> Vec<u8> {
    let mut bytes = vec![
        0;
        8 + entries
            .iter()
            .map(|(_, text)| text.len() + 8)
            .sum::<usize>()
    ];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Fixture storage");
    match profile {
        BaselineProtocol::Legacy6 => writer
            .write_i16(i16::try_from(entries.len()).expect("Short fixture count"))
            .expect("Count"),
        BaselineProtocol::Cod4x21 => writer
            .write_i32(i32::try_from(entries.len()).expect("Long fixture count"))
            .expect("Count"),
    }
    let mut previous = -1;
    for &(index, text) in entries {
        match profile {
            BaselineProtocol::Legacy6 if index == previous + 1 => {
                writer.write_bits(1, 1).expect("Increment flag");
            }
            BaselineProtocol::Legacy6 => {
                writer.write_bits(0, 1).expect("Absolute flag");
                writer
                    .write_bits(u32::try_from(index).expect("Nonnegative fixture index"), 12)
                    .expect("Absolute index");
            }
            BaselineProtocol::Cod4x21 => writer.write_i32(index).expect("Long index"),
        }
        for &byte in text {
            writer.write_u8(byte).expect("String byte");
        }
        writer.write_u8(0).expect("String terminator");
        previous = index;
    }
    writer.as_bytes().to_vec()
}

fn dirty_slots(count: usize) -> Vec<ConfigStringSlot> {
    let input = block(BaselineProtocol::Cod4x21, &[(4883, b"old-private-value")]);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Old block");
    let mut scratch = [0; 32];
    let mut output = [0; 32];
    let mut slot = [ConfigStringSlot::default()];
    let plan = ConfigStringBlockPlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        BUDGET,
        &mut scratch,
        &mut output,
        &mut slot,
    )
    .expect("Initialized old metadata");
    assert_eq!(plan.record_count(), 1);
    vec![slot[0]; count]
}
