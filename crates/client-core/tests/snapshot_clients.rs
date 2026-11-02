// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete client-state framing, sparse merge, budgets and cooked ownership.
#![allow(clippy::expect_used, reason = "Finite checked client-stream fixtures.")]
#[path = "snapshot_client_cases/bounds.rs"]
mod bounds;
#[path = "snapshot_client_cases/ownership.rs"]
mod ownership;
#[path = "snapshot_client_cases/traces.rs"]
mod traces;
#[path = "snapshot_client_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    client_states::{
        ClientState, ClientStateDecoder, ClientStateDelta, ClientStateError, ClientStateWord as W,
    },
    message_body::BodyReader,
    snapshot_clients::{
        ClientStateSource, ClientStreamBudget, ClientStreamError, SnapshotClientPlan,
        SnapshotClientSlot,
    },
};

const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
const BUDGET: ClientStreamBudget = ClientStreamBudget {
    max_wire_records: 65,
    max_previous_clients: 64,
    max_clients: 64,
};
#[derive(Clone, Copy)]
enum Update {
    Retain(u32),
    Remove(u32),
    Model(u32, u16, u16),
}
fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
fn write_update(writer: &mut MessageWriter<'_>, update: Update) {
    let number = match update {
        Update::Retain(n) | Update::Remove(n) | Update::Model(n, _, _) => n,
    };
    writer
        .write_bits(1 | (number << 2), 8)
        .expect("More plus absolute client index");
    writer
        .write_bits(u32::from(matches!(update, Update::Remove(_))), 1)
        .expect("Removed");
    if matches!(update, Update::Remove(_)) {
        return;
    }
    writer
        .write_bits(u32::from(matches!(update, Update::Model(..))), 1)
        .expect("Delta");
    if let Update::Model(_, from, to) = update {
        writer.write_bits(1, 5).expect("Model prefix");
        writer.write_bits(1, 1).expect("Changed model");
        writer
            .write_bits(u32::from(to != 0), 1)
            .expect("Nonzero model");
        if to != 0 {
            let delta = from ^ to;
            writer
                .write_bits(u32::from(delta & 1), 1)
                .expect("Residual model bit");
            writer
                .write_u8(u8::try_from(delta >> 1).expect("Model byte"))
                .expect("XOR model byte");
        }
    }
}
fn fixture(updates: &[Update], prefix: bool, tail: bool) -> Vec<u8> {
    encoded(|writer| {
        if prefix {
            writer.write_bits(5, 3).expect("Cached prefix");
        }
        for &update in updates {
            write_update(writer, update);
        }
        writer.write_bits(0, 1).expect("End bit");
        if tail {
            writer.write_u8(0xa7).expect("Following byte");
            writer.write_bits(3, 2).expect("Following cached bits");
        }
    })
}
fn state(number: u32, value: u16) -> ClientState {
    let bytes = fixture(&[Update::Model(number, 0, value)], false, false);
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Record body");
    let mut decoder = ClientStateDecoder::new(PROFILES[1]);
    let ClientStateDelta::State(state) = decoder
        .prepare(&mut reader)
        .expect("Header")
        .expect("Record")
        .decode(None)
        .expect("State")
    else {
        unreachable!("State fixture")
    };
    state
}
fn source(states: &[ClientState]) -> ClientStateSource<'_> {
    ClientStateSource::from_states(states)
}
fn values(plan: &SnapshotClientPlan<'_>) -> Vec<(u32, u32)> {
    plan.clients()
        .map(|state| (state.number(), state.word(W::Model)))
        .collect()
}
fn dirty(length: usize) -> Vec<SnapshotClientSlot> {
    let bytes = fixture(&[Update::Model(7, 0, 77)], false, false);
    let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Seed");
    let mut slots = [SnapshotClientSlot::default()];
    let _ = SnapshotClientPlan::parse(&mut reader, PROFILES[1], source(&[]), BUDGET, &mut slots)
        .expect("Dirty seed");
    vec![slots[0].clone(); length]
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
fn failure(
    bytes: &[u8],
    previous: &[ClientState],
    budget: ClientStreamBudget,
    capacity: usize,
    expected: ClientStreamError,
) {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    let before = position(&reader);
    let mut output = dirty(capacity);
    let saved = output.clone();
    assert_eq!(
        SnapshotClientPlan::parse(
            &mut reader,
            PROFILES[1],
            source(previous),
            budget,
            &mut output
        )
        .err(),
        Some(expected)
    );
    assert_eq!(position(&reader), before);
    assert_eq!(output, saved);
}
