// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic client primitives, independent of sockets, rendering and server state.
//!
//! Discovery is connectionless and does not establish a gameplay connection,
//! authenticated identity or negotiated extension support.

mod challenge;
pub mod commands;
pub mod connect;
pub mod control;
mod error;
pub mod handshake;
mod info_response;
mod request;

pub use challenge::QueryChallenge;
pub use error::DiscoveryError;
pub use info_response::{InfoResponse, MAX_INFO_DATAGRAM_BYTES};
pub use request::INFO_REQUEST_BYTES;
