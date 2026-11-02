// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete constituent validation before publishing caller-funded gamestate state.
#![allow(
    clippy::expect_used,
    reason = "Finite checked full-gamestate fixtures."
)]

#[path = "gamestate_cases/budgets.rs"]
mod budgets;
#[path = "gamestate_cases/default_scopes.rs"]
mod default_scopes;
#[path = "gamestate_cases/failures.rs"]
mod failures;
#[path = "gamestate_cases/ordinary/mod.rs"]
mod ordinary;
#[path = "gamestate_cases/reliable.rs"]
mod reliable;
#[path = "gamestate_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{
        ClientDataSlot, EntityBaselineSlot, GameStateBudget, GameStatePlan, GameStateStorage,
        configstrings::ConfigStringSlot,
    },
};

const BUDGET: GameStateBudget = GameStateBudget {
    max_services: 16,
    max_configstrings: 16,
    max_text_bytes: 4096,
    max_entities: 8,
    max_client_records: 8,
};

#[derive(Clone, Debug, Eq, PartialEq)]
struct Output {
    text: Vec<u8>,
    strings: Vec<ConfigStringSlot>,
    entities: Vec<EntityBaselineSlot>,
    clients: Vec<ClientDataSlot>,
}
impl Output {
    fn new(text: usize, strings: usize, entities: usize, clients: usize) -> Self {
        Self {
            text: vec![0xa5; text],
            strings: vec![ConfigStringSlot::default(); strings],
            entities: vec![EntityBaselineSlot::default(); entities],
            clients: vec![ClientDataSlot::default(); clients],
        }
    }
    fn storage(&mut self) -> GameStateStorage<'_> {
        GameStateStorage {
            configstring_bytes: &mut self.text,
            configstring_slots: &mut self.strings,
            entity_slots: &mut self.entities,
            client_slots: &mut self.clients,
        }
    }
    fn dirty(text: usize, strings: usize, entities: usize, clients: usize) -> Self {
        let input = encoded(|writer| mixed(writer, 63));
        let mut reader = cod4xr_client_core::message_body::BodyReader::try_new(&input, input.len())
            .expect("Seed body");
        let mut seed = Self::new(7, 3, 2, 2);
        GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut [0; 5],
            seed.storage(),
        )
        .expect("Dirty reused slots");
        Self {
            text: vec![0xa5; text],
            strings: vec![seed.strings[0]; strings],
            entities: vec![seed.entities[0].clone(); entities],
            clients: vec![seed.clients[0].clone(); clients],
        }
    }
}

fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut output = vec![0; 64 * 1024];
    let mut writer = MessageWriter::try_new(&mut output).expect("Fixture");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
fn string(writer: &mut MessageWriter<'_>, value: &[u8]) {
    for &byte in value {
        writer.write_u8(byte).expect("String byte");
    }
    writer.write_u8(0).expect("Terminator");
}
fn configstrings(writer: &mut MessageWriter<'_>, entries: &[(i32, &[u8])]) {
    writer.write_u8(2).expect("Configstring opcode");
    writer
        .write_i32(i32::try_from(entries.len()).expect("Count"))
        .expect("Count");
    for &(index, value) in entries {
        writer.write_i32(index).expect("Index");
        string(writer, value);
    }
}
fn entity(writer: &mut MessageWriter<'_>, number: u32) {
    writer.write_u8(3).expect("Baseline opcode");
    writer
        .write_bits((number << 2) | 2, 12)
        .expect("Independent absolute entity index");
    writer
        .write_bits(0, 2)
        .expect("Not removed, no changes from zero");
}
fn client(writer: &mut MessageWriter<'_>, index: u8, name: &[u8], tag: &[u8]) {
    writer.write_u8(11).expect("Client identity opcode");
    writer.write_u8(index).expect("Client index");
    string(writer, name);
    string(writer, tag);
}
fn finish(writer: &mut MessageWriter<'_>, own_client: i32) {
    writer.write_u8(7).expect("Gamestate EOF");
    writer.write_i32(-2).expect("Config-data sequence bits");
    writer.write_i32(own_client).expect("Own client");
    writer
        .write_i32(0xaabb_ccdd_u32.cast_signed())
        .expect("Checksum feed bits");
}
fn mixed(writer: &mut MessageWriter<'_>, own_client: i32) {
    writer
        .write_i32(0x8000_0001_u32.cast_signed())
        .expect("Server command sequence bits");
    configstrings(writer, &[(0, b"a%"), (4883, b"")]);
    entity(writer, 17);
    client(writer, 63, b"Private%", b"Tag%");
    configstrings(writer, &[(0, b"later")]);
    entity(writer, 18);
    client(writer, 63, b"second", b"");
    finish(writer, own_client);
}
