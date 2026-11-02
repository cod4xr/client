// SPDX-License-Identifier: AGPL-3.0-or-later
//! Live scalar prefixes, native field hints and prediction-mode provenance.
#![allow(
    clippy::expect_used,
    reason = "Finite reviewed live player-scalar fixtures."
)]
#[path = "player_scalar_cases/failures.rs"]
mod failures;
#[path = "player_scalar_cases/hints.rs"]
mod hints;
#[path = "player_scalar_cases/support.rs"]
mod support;
#[path = "player_scalar_cases/traces.rs"]
mod traces;
#[path = "player_scalar_cases/values.rs"]
mod values;
#[path = "player_scalar_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    message_body::BodyReader,
    player_scalars::{
        PLAYER_SCALAR_WORDS, PlayerScalarError, PlayerScalarPrefix, PlayerScalars, PlayerWord, word,
    },
};
use support::{encoded, header, index, integer, one_field, parse, position, raw_float, seed};
const PROFILES: [BaselineProtocol; 2] = [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21];
const UNCONDITIONAL: [(PlayerWord, u8); 6] = [
    (word::view::HEIGHT_LERP_DOWN, 1),
    (word::weapons::OFFHAND_SECONDARY, 1),
    (word::view::RADAR_ENABLED, 1),
    (word::effects::SHELLSHOCK_DURATION, 16),
    (word::motion::SPRINT_BUTTON_UP_REQUIRED, 1),
    (word::motion::SPRINT_DELAY, 1),
];
const FORCED: [(PlayerWord, i16); 8] = [
    (word::motion::ORIGIN_X, -88),
    (word::motion::ORIGIN_Y, -88),
    (word::motion::BOB_CYCLE, 8),
    (word::motion::VELOCITY_Y, -88),
    (word::motion::VELOCITY_X, -88),
    (word::motion::DIRECTION, -8),
    (word::motion::ORIGIN_Z, -88),
    (word::motion::VELOCITY_Z, -88),
];
