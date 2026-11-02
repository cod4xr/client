// SPDX-License-Identifier: AGPL-3.0-or-later
//! Complete service control plans expose normalized records only after logical EOF.
#![allow(
    clippy::expect_used,
    reason = "Finite service vectors use explicit caller buffers."
)]
#[path = "service_cases/limits.rs"]
mod limits;
#[path = "service_cases/records.rs"]
mod records;

use crate::service_support;
use service_support::{Fixture, append_command};
