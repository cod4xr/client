// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use crate::{ReliableError, send::SendError};

/// Fixed scheduling failures without retained bytes or private routing data.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    /// Host timing intervals are zero or internally inconsistent.
    Configuration,
    /// Explicit host time precedes the last admitted observation.
    ClockRegressed,
    /// A required future timer does not fit the explicit duration domain.
    ClockOverflow,
    /// Queue metadata could not admit a selected original fragment.
    Queue(SendError),
    /// A supplied window snapshot could not form a representable body header.
    Body(ReliableError),
}

impl fmt::Display for ScheduleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration => formatter.write_str("invalid reliable schedule configuration"),
            Self::ClockRegressed => formatter.write_str("reliable schedule clock regressed"),
            Self::ClockOverflow => formatter.write_str("reliable schedule timer overflow"),
            Self::Queue(error) => error.fmt(formatter),
            Self::Body(error) => error.fmt(formatter),
        }
    }
}

impl Error for ScheduleError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Queue(error) => Some(error),
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
