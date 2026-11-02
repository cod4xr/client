// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete HUD sections, native value forms, trailing slots and atomic references.
#![allow(clippy::expect_used, reason = "Finite reviewed native HUD fixtures.")]
#[path = "player_hud_cases/catalog.rs"]
mod catalog;
#[path = "player_hud_cases/context.rs"]
mod context;
#[path = "player_hud_cases/failures.rs"]
mod failures;
#[path = "player_hud_cases/fields.rs"]
mod fields;
#[path = "player_hud_cases/retention.rs"]
mod retention;
#[path = "player_hud_cases/support.rs"]
mod support;
#[path = "player_hud_cases/traces.rs"]
mod traces;
#[path = "player_hud_cases/vectors.rs"]
mod vectors;

use catalog::FIELDS;
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entities::{EntityDecodeContext, LegacyMapCenter},
    message_body::BodyReader,
    player_hud::{HUD_SLOTS, HUD_WORDS, HudError, HudWord as W, SnapshotHud},
};
use support::{array, encoded, integer, one_field, parse, position, raw_float, seed};

const MODERN: EntityDecodeContext = EntityDecodeContext::Cod4x21;
fn legacy(center: [f32; 3]) -> EntityDecodeContext {
    EntityDecodeContext::Legacy6(LegacyMapCenter::new(center).expect("Explicit checked map center"))
}
