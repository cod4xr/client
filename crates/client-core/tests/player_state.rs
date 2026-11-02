// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole live player-state ownership, composition and native prediction semantics.
#![allow(
    clippy::expect_used,
    reason = "Reviewed bounded player-state fixtures."
)]
#[path = "player_state_cases/failures.rs"]
mod failures;
#[path = "player_state_cases/history/mod.rs"]
mod history;
#[path = "player_state_cases/ownership.rs"]
mod ownership;
#[path = "player_state_cases/prediction.rs"]
mod prediction;
#[path = "player_state_cases/support.rs"]
mod support;
#[path = "player_state_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    entities::{EntityDecodeContext, LegacyMapCenter},
    message_body::BodyReader,
    player_scalars::word,
    player_state::{
        LivePlayerState, PlayerStateError, PredictionProvenance, PredictionSample, PredictionWords,
    },
};
use support::{contexts, decode, empty, encoded, mixed, position, sample};
