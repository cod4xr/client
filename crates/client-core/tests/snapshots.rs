// SPDX-License-Identifier: AGPL-3.0-or-later
//! Whole native snapshot wire transactions, coherent references and prediction.
#![allow(clippy::expect_used, reason = "Bounded independent snapshot fixtures.")]
#[path = "snapshot_cases/ack_commands.rs"]
mod ack_commands;
#[path = "snapshot_cases/ack_failures.rs"]
mod ack_failures;
#[path = "snapshot_cases/ack_vectors.rs"]
mod ack_vectors;
#[path = "snapshot_cases/bindings.rs"]
mod bindings;
#[path = "snapshot_cases/ownership.rs"]
mod ownership;
#[path = "snapshot_cases/payload_failures.rs"]
mod payload_failures;
#[path = "snapshot_cases/payload_vectors.rs"]
mod payload_vectors;
#[path = "snapshot_cases/prediction.rs"]
mod prediction;
#[path = "snapshot_cases/rollback.rs"]
mod rollback;
#[path = "snapshot_cases/service_failures.rs"]
mod service_failures;
#[path = "snapshot_cases/service_vectors.rs"]
mod service_vectors;
#[path = "snapshot_cases/source_limits.rs"]
mod source_limits;
#[path = "snapshot_cases/support.rs"]
mod support;
#[path = "snapshot_cases/vectors.rs"]
mod vectors;

use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    client_states::ClientStateWord,
    entities::{EntityDecodeContext, LegacyMapCenter},
    entity_deltas::EntityWord,
    message_body::BodyReader,
    player_scalars::word,
    player_state::{PlayerStateError, PredictionSample, PredictionWords},
    snapshot_clients::{ClientStreamBudget, ClientStreamError, SnapshotClientSlot},
    snapshot_entities::{
        EntityStateSource, EntityStreamBudget, EntityStreamError, SnapshotEntitySlot,
    },
    snapshots::{
        SnapshotBaselines, SnapshotBudget, SnapshotError, SnapshotInput, SnapshotMessage,
        SnapshotOutput, SnapshotPlan, SnapshotSource,
    },
};
use support::{
    Change, body, client, contexts, dirty, empty_tail, encoded, entity, input, output, position,
    prefix, sample, values,
};
