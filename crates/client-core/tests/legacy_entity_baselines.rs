// SPDX-License-Identifier: AGPL-3.0-or-later
//! Legacy zero-reference coordinate contexts, field layouts and atomic baseline admission.
#![allow(
    clippy::expect_used,
    reason = "Finite explicit legacy coordinate fixtures."
)]
#[path = "legacy_entity_cases/context.rs"]
mod context;
#[path = "legacy_entity_cases/failures.rs"]
mod failures;
mod legacy_entity_support;
#[path = "legacy_entity_cases/vectors.rs"]
mod vectors;
use legacy_entity_support::{Coordinate, coordinate, decoder, fixture, header, position};

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::entity_baselines::{
        BaselineWord as W, EntityBaselineDecoder, EntityBaselineError, LegacyMapCenter,
    },
    message_body::BodyReader,
};
