// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed connect-plan failure categories without userinfo, cookies or private names.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectPlanError {
    /// Protocol must be positive and the explicit version string nonempty.
    InvalidSettings,
    /// Caller fields conflict with typed protocol/challenge/qport/version settings.
    ReservedField,
    /// Caller userinfo repeats a case-insensitive key.
    DuplicateField,
    /// Userinfo/version cannot be encoded in the legacy command's complete budget.
    UserinfoEncoding,
    /// A bounded numeric formatter or shared envelope encoder failed.
    Encoding,
    /// Caller output is too small; no output bytes are changed.
    OutputCapacity,
}

impl fmt::Display for ConnectPlanError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "connect plan rejected: {self:?}")
    }
}

impl Error for ConnectPlanError {}
