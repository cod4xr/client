// SPDX-License-Identifier: AGPL-3.0-or-later
//! Bounded native movement/action command encoding, before packet or I/O admission.
//! Wire widths do not define future engine asset/input storage or extension limits.

mod error;
mod input;
mod plan;
mod write;

#[cfg(test)]
mod cached;

pub use error::UserCommandEncodeError;
pub use input::{MovementAxis, UserCommandInput, UserCommandMelee};
pub use plan::AbsoluteUserCommandPlan;

/// Maximum full-form command bytes when starting a fresh native bit cursor.
/// This excludes movement opcode/count, packet headers, prediction trailer and EOF.
pub const MAX_ABSOLUTE_COMMAND_BYTES: usize = 22;
