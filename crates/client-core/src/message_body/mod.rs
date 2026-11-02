// SPDX-License-Identifier: AGPL-3.0-or-later
//! Caller-bounded complete-body views over the pinned shared primitive codecs.
//! A shared reader's 128 KiB storage-view cap does not cap native reliable bodies.
//! This adapter changes no field layout, legacy namespace or negotiated profile.

mod error;
mod index;
mod reader;
mod typed;
mod window;

pub use error::BodyReadError;
pub use reader::BodyReader;
