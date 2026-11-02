// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4x_protocol::MessageError;
use std::{error::Error, fmt};

/// Fixed encoding failures without private keys, input, timestamps or inventory IDs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UserCommandEncodeError {
    /// The native button word exceeds twenty-one bits.
    Buttons,
    /// The native weapon identifier exceeds seven bits.
    Weapon,
    /// The native offhand identifier exceeds seven bits.
    Offhand,
    /// Selected-location presence disagrees with its native button.
    SelectedLocation,
    /// Melee presence disagrees with its native button.
    Melee,
    /// Caller final storage cannot hold the whole encoded command.
    OutputCapacity,
    /// A shared primitive write failed in private bounded storage.
    Message(MessageError),
}
impl fmt::Display for UserCommandEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Buttons => "user command button word exceeds native width",
            Self::Weapon => "user command weapon exceeds native width",
            Self::Offhand => "user command offhand exceeds native width",
            Self::SelectedLocation => "user command selected-location presence mismatch",
            Self::Melee => "user command melee presence mismatch",
            Self::OutputCapacity => "user command output capacity exhausted",
            Self::Message(_) => "user command primitive encoding failed",
        })
    }
}
impl Error for UserCommandEncodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Message(error) => Some(error),
            _ => None,
        }
    }
}
