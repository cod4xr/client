// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

use crate::{framing::FrameError, receive::ReceiveError, schedule::ScheduleError, send::SendError};

/// Fixed endpoint failures without application body contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EndpointError {
    /// Receive storage or incoming data-window admission failed.
    Receive(ReceiveError),
    /// Queue storage or outgoing acknowledgement admission failed.
    Send(SendError),
    /// Clock/timer/selection admission failed.
    Schedule(ScheduleError),
    /// Frame representation or storage budget failed.
    Frame(FrameError),
    /// A previously invalid frame prefix permanently failed this endpoint.
    Failed,
}

impl fmt::Display for EndpointError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Receive(error) => error.fmt(formatter),
            Self::Send(error) => error.fmt(formatter),
            Self::Schedule(error) => error.fmt(formatter),
            Self::Frame(error) => error.fmt(formatter),
            Self::Failed => formatter.write_str("reliable endpoint failed; reconnect required"),
        }
    }
}

impl Error for EndpointError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Receive(error) => Some(error),
            Self::Send(error) => Some(error),
            Self::Schedule(error) => Some(error),
            Self::Frame(error) => Some(error),
            Self::Failed => None,
        }
    }
}
