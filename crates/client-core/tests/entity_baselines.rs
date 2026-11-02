// SPDX-License-Identifier: AGPL-3.0-or-later
//! Entity baseline layout selection, independent native vectors and atomic admission.
#![allow(clippy::expect_used, reason = "Finite checked baseline fixtures.")]

#[path = "entity_baseline_cases/failures.rs"]
mod failures;
#[path = "entity_baseline_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{BaselineProtocol, gamestate::entity_baselines::EntityBaselineDecoder};

fn decoder() -> EntityBaselineDecoder {
    EntityBaselineDecoder::new(BaselineProtocol::Cod4x21).expect("Reviewed protocol")
}
fn header(writer: &mut MessageWriter<'_>, number: u32, count: Option<u32>, kind: u8) {
    // Independent absolute entity form: low two bits 10, then the full ten-bit ID.
    writer
        .write_bits((number << 2) | 2, 12)
        .expect("Absolute index");
    writer.write_bits(0, 1).expect("Not removed");
    writer
        .write_bits(u32::from(count.is_some()), 1)
        .expect("Delta present");
    if let Some(count) = count {
        writer.write_bits(count, 6).expect("Changed-field count");
        if count > 0 {
            writer
                .write_bits(u32::from(kind != 0), 1)
                .expect("Type changed");
            if kind != 0 {
                writer.write_bits(1, 1).expect("Type nonzero");
                writer.write_u8(kind).expect("Type byte");
            }
        }
    }
}
fn fixture(
    number: u32,
    count: Option<u32>,
    kind: u8,
    fields: impl FnOnce(&mut MessageWriter<'_>),
) -> Vec<u8> {
    let mut output = [0; 512];
    let mut writer = MessageWriter::try_new(&mut output).expect("Fixture storage");
    header(&mut writer, number, count, kind);
    fields(&mut writer);
    writer.as_bytes().to_vec()
}
