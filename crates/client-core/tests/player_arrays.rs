// SPDX-License-Identifier: AGPL-3.0-or-later
//! Native mixed-width stats, absolute ammunition and raw model-table sections.
#![allow(
    clippy::expect_used,
    reason = "Finite checked native player-array fixtures."
)]
#[path = "player_array_cases/ammunition.rs"]
mod ammunition;
#[path = "player_array_cases/models.rs"]
mod models;
#[path = "player_array_cases/stats.rs"]
mod stats;
#[path = "player_array_cases/traces.rs"]
mod traces;
#[path = "player_array_cases/transactions.rs"]
mod transactions;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    message_body::BodyReader,
    player_arrays::{PlayerAmmunition, PlayerArrayError, PlayerStats, WeaponModels},
};

const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
const STAT_VECTOR: [u8; 9] = [0xff, 0, 0x80, 0xff, 0xff, 0xff, 0x7f, 0x0f, 0xff];
const AMMO_VECTOR: [u8; 16] = [
    0x13, 1, 0x80, 0, 0x80, 0xff, 0xff, 0, 0x80, 0xff, 0x7f, 0x10, 0, 0x80, 0x85, 0xff,
];
fn encoded(write: impl FnOnce(&mut MessageWriter<'_>)) -> Vec<u8> {
    let mut bytes = [0; 4096];
    let mut writer = MessageWriter::try_new(&mut bytes).expect("Storage");
    write(&mut writer);
    writer.as_bytes().to_vec()
}
fn write_stats(writer: &mut MessageWriter<'_>, mask: u32, values: &[i32; 5]) {
    writer
        .write_bits(u32::from(mask != 0), 1)
        .expect("Stats changed");
    if mask == 0 {
        return;
    }
    writer.write_bits(mask, 5).expect("Stats mask");
    for (slot, &value) in values[..3].iter().enumerate() {
        if mask & (1 << slot) != 0 {
            writer
                .write_i16(i16::try_from(value).expect("Short stat"))
                .expect("Stat short");
        }
    }
    if mask & 8 != 0 {
        writer
            .write_bits(values[3].cast_unsigned(), 6)
            .expect("Six-bit stat");
    }
    if mask & 16 != 0 {
        writer
            .write_u8(u8::try_from(values[4]).expect("Byte stat"))
            .expect("Stat byte");
    }
}
fn write_groups(writer: &mut MessageWriter<'_>, masks: &[u16], values: &[i32]) {
    for (group, &mask) in values.chunks_exact(16).zip(masks) {
        writer
            .write_bits(u32::from(mask != 0), 1)
            .expect("Group flag");
        if mask == 0 {
            continue;
        }
        writer.write_i16(mask.cast_signed()).expect("Mask");
        for (slot, &value) in group.iter().enumerate() {
            if mask & (1 << slot) != 0 {
                writer
                    .write_i16(i16::try_from(value).expect("Short ammo"))
                    .expect("Absolute ammo");
            }
        }
    }
}
fn write_ammo(
    writer: &mut MessageWriter<'_>,
    stored_masks: [u16; 4],
    clip_masks: &[u16; 8],
    stored: &[i32; 64],
    clips: &[i32; 128],
) {
    let changed = stored_masks.iter().any(|&mask| mask != 0);
    writer
        .write_bits(u32::from(changed), 1)
        .expect("Stored section flag");
    if changed {
        write_groups(writer, &stored_masks, stored);
    }
    write_groups(writer, clip_masks, clips);
}
fn parse_stats(
    profile: BaselineProtocol,
    bytes: &[u8],
    prior: Option<&PlayerStats>,
) -> PlayerStats {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    PlayerStats::parse(&mut reader, profile, prior).expect("Stats")
}
fn parse_ammo(
    profile: BaselineProtocol,
    bytes: &[u8],
    prior: Option<&PlayerAmmunition>,
) -> PlayerAmmunition {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    PlayerAmmunition::parse(&mut reader, profile, prior).expect("Ammunition")
}
fn parse_models(
    profile: BaselineProtocol,
    bytes: &[u8],
    prior: Option<&WeaponModels>,
) -> WeaponModels {
    let mut reader = BodyReader::try_new(bytes, bytes.len()).expect("Body");
    WeaponModels::parse(&mut reader, profile, prior).expect("Models")
}
fn position(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}
