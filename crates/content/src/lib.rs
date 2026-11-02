// SPDX-License-Identifier: AGPL-3.0-or-later
//! Deterministic read-only imports from caller-provided game files.
//! No filesystem, executable loading, logging subscriber or engine state is owned here.

pub mod defaults;
pub mod pe32;
