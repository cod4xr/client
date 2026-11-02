// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native client-state records, physical byte vectors and transactional ownership.
#![allow(clippy::expect_used, reason = "Finite reviewed client-state fixtures.")]
#[path = "client_state_cases/failures.rs"]
mod failures;
#[path = "client_state_cases/traces.rs"]
mod traces;
#[path = "client_state_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    client_states::{
        ClientState, ClientStateDecoder, ClientStateDelta, ClientStateError, ClientStateWord as W,
    },
    message_body::BodyReader,
};

const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
const WORDS: [(W, u8); 24] = [
    (W::Model, 9),
    (W::Name0, 32),
    (W::Rank, 8),
    (W::Prestige, 8),
    (W::Team, 2),
    (W::VehicleEntity, 10),
    (W::Name1, 32),
    (W::AttachmentModel0, 9),
    (W::Name2, 32),
    (W::Perks, 32),
    (W::Name3, 32),
    (W::AttachmentModel1, 9),
    (W::SprintTimeMultiplier, 0),
    (W::VehicleSeat, 2),
    (W::AttachmentTag5, 5),
    (W::AttachmentTag0, 5),
    (W::AttachmentTag1, 5),
    (W::AttachmentTag2, 5),
    (W::AttachmentTag3, 5),
    (W::AttachmentTag4, 5),
    (W::AttachmentModel2, 9),
    (W::AttachmentModel3, 9),
    (W::AttachmentModel4, 9),
    (W::AttachmentModel5, 9),
];
fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 512];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
fn header(writer: &mut MessageWriter<'_>, number: u32, count: Option<u32>) {
    writer
        .write_bits(1 | (number << 2), 8)
        .expect("More and absolute client index");
    writer.write_bits(0, 1).expect("Not removed");
    writer
        .write_bits(u32::from(count.is_some()), 1)
        .expect("Delta flag");
    if let Some(count) = count {
        writer.write_bits(count, 5).expect("Prefix count");
    }
}
fn integer(writer: &mut MessageWriter<'_>, width: u8, prior: u32, value: u32) {
    writer.write_bits(1, 1).expect("Changed");
    writer
        .write_bits(u32::from(value != 0), 1)
        .expect("Nonzero");
    if value != 0 {
        let delta = prior ^ value;
        let residual = width % 8;
        writer
            .write_bits(delta & ((1 << residual) - 1), residual)
            .expect("Residual");
        for shift in (residual..width).step_by(8) {
            writer
                .write_u8(u8::try_from((delta >> shift) & 255).expect("Low byte"))
                .expect("XOR byte");
        }
    }
}
fn raw_float(writer: &mut MessageWriter<'_>, prior: u32, value: u32) {
    writer.write_bits(7, 3).expect("Changed nonzero raw float");
    writer
        .write_i32((prior ^ value).cast_signed())
        .expect("XOR float word");
}
fn present(delta: ClientStateDelta) -> ClientState {
    let ClientStateDelta::State(state) = delta else {
        unreachable!("State fixture")
    };
    state
}
fn decode(profile: BaselineProtocol, bytes: &[u8], prior: Option<&ClientState>) -> ClientState {
    let mut owner = ClientStateDecoder::new(profile);
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    present(
        owner
            .prepare(&mut reader)
            .expect("Header")
            .expect("Record")
            .decode(prior)
            .expect("State"),
    )
}
fn reference(number: u32, value: u32) -> ClientState {
    let bytes = encoded(|writer| {
        header(writer, number, Some(24));
        for &(_, width) in &WORDS {
            if width == 0 {
                raw_float(writer, 0, value);
            } else {
                integer(writer, width, 0, value & (u32::MAX >> (32 - width)));
            }
        }
    });
    decode(BaselineProtocol::Cod4x21, &bytes, None)
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
