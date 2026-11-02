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
mod error;
pub mod gamestate;
pub mod handshake;
mod info_response;
pub mod message_body;
pub mod pre_game;
mod profile;
mod request;
pub mod server_commands;
pub mod server_payload;
pub mod service_controls;
pub mod system_info;

pub use challenge::QueryChallenge;
pub use error::DiscoveryError;
/// Protocol-21 individual entity deltas sharing the reviewed baseline wire catalog.
pub use gamestate::entity_baselines::deltas as entity_deltas;
pub use info_response::{InfoResponse, MAX_INFO_DATAGRAM_BYTES};
pub use profile::BaselineProtocol;
pub use request::INFO_REQUEST_BYTES;
