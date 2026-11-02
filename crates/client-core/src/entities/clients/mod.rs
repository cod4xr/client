// SPDX-License-Identifier: AGPL-3.0-or-later
//! Ordered snapshot client-state records against caller-selected prior words.
//! Native wire widths do not cap engine storage. Complete snapshot admission,
//! identity reconciliation and model/vehicle/resource interpretation remain separate.

mod decoder;
mod error;
mod prepared;
mod record;
mod state;
pub mod stream;
mod table;
mod word;

pub use decoder::ClientStateDecoder;
pub use error::ClientStateError;
pub use prepared::PreparedClientState;
pub use state::{ClientState, ClientStateDelta};
pub use word::ClientStateWord;

/// Number of inert semantic words in the native snapshot client-state schema.
/// This excludes the separately validated wire client index.
pub const CLIENT_STATE_WORDS: usize = 24;
