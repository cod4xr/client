// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic client primitives, independent of sockets, rendering and server state.
//!
//! Discovery is connectionless and does not establish a gameplay connection,
//! authenticated identity or negotiated extension support.

mod challenge;
pub mod client_data;
pub mod client_identities;
pub mod commands;
pub mod config_state;
pub mod connect;
pub mod control;
pub mod entities;
mod error;
pub mod gamestate;
pub mod handshake;
mod info_response;
pub mod message_body;
pub mod player_arrays;
pub mod pre_game;
mod profile;
mod request;
pub mod server_commands;
pub mod server_payload;
pub mod service_controls;
pub mod snapshots;
pub mod system_info;

pub use challenge::QueryChallenge;
/// Owned snapshot client-state records, distinct from protocol-21 identity updates.
pub use entities::clients as client_states;
/// Complete native client-state substreams with transactional merged output.
pub use entities::clients::stream as snapshot_clients;
/// Explicit-profile entity deltas sharing the reviewed baseline wire catalog.
pub use entities::deltas as entity_deltas;
/// Complete native entity substreams with transactional merged output.
pub use entities::deltas::stream as snapshot_entities;
/// Complete native player-HUD sections before typed presentation/resource validation.
pub use entities::hud as player_hud;
/// Complete native player-objective sections with owned, atomic decoded results.
pub use entities::objectives as player_objectives;
/// Owned live player-scalar prefixes before prediction and whole-state admission.
pub use entities::player_scalars;
/// Transactional complete live player-state wire projections and prediction resolution.
pub use entities::player_state;
pub use error::DiscoveryError;
pub use info_response::{InfoResponse, MAX_INFO_DATAGRAM_BYTES};
pub use profile::BaselineProtocol;
pub use request::INFO_REQUEST_BYTES;
