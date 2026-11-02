// SPDX-License-Identifier: AGPL-3.0-or-later
//! Nonzero legacy origins, explicit context ownership and atomic record admission.
#![allow(
    clippy::expect_used,
    reason = "Finite checked legacy prior-state fixtures."
)]
#[path = "legacy_delta_cases/context.rs"]
mod context;
#[path = "legacy_delta_cases/failures.rs"]
mod failures;
#[path = "legacy_delta_cases/layouts.rs"]
mod layouts;
mod legacy_entity_support;
#[path = "legacy_delta_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entity_deltas::{
        EntityDelta, EntityDeltaDecoder, EntityDeltaError, EntityState, EntityWord as W,
    },
    gamestate::entity_baselines::{EntityBaselineDecoder, LegacyMapCenter},
    message_body::BodyReader,
};
use legacy_entity_support::{Coordinate, decoder as baseline_decoder, fixture, header, position};

fn decoder(center: [f32; 3]) -> EntityDeltaDecoder {
    EntityDeltaDecoder::with_legacy_map_center(LegacyMapCenter::new(center).expect("Context"))
}
fn reference(number: u32) -> EntityState {
    let source = fixture(
        number,
        [
            Coordinate::Small(65),
            Coordinate::Small(63),
            Coordinate::Small(66),
        ],
        false,
        false,
    );
    let mut reader = BodyReader::try_new(&source, source.len()).expect("Baseline");
    EntityState::from_baseline(
        &baseline_decoder([0.0; 3])
            .parse(&mut reader)
            .expect("Initial legacy state"),
    )
}
fn raw_reference(number: u32, values: [u32; 3]) -> EntityState {
    let mut bytes = [0; 128];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    header(&mut writer, number, 5, 0);
    writer.write_bits(0, 1).expect("Unchanged flags");
    for value in values {
        writer
            .write_bits(1, 1)
            .expect("Changed absolute coordinate");
        writer
            .write_i32(value.cast_signed())
            .expect("Raw semantic word");
    }
    let mut reader =
        BodyReader::try_new(writer.as_bytes(), writer.as_bytes().len()).expect("Raw baseline");
    let baseline = EntityBaselineDecoder::new(BaselineProtocol::Cod4x21)
        .expect("Modern semantic seed")
        .parse(&mut reader)
        .expect("Owned raw words");
    EntityState::from_baseline(&baseline)
}
fn present(record: EntityDelta) -> EntityState {
    match record {
        EntityDelta::State(state) => state,
        EntityDelta::Removed { .. } => unreachable!("State fixture"),
    }
}
