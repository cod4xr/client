// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic read-only imports from caller-provided game files.
//! No filesystem, executable loading, logging subscriber or engine state is owned here.

pub mod asset_key;
pub mod defaults;
pub mod fastfile5;
pub mod iwi6;
pub mod mount_index;
pub mod pe32;
pub mod zip32;
pub mod zip_payload;
pub mod zone5;
