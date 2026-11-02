// SPDX-License-Identifier: AGPL-3.0-or-later
use super::UserCommandEncodeError;
use std::fmt;

/// Canonical native wire movement axis, without engine velocity or keyboard ownership.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MovementAxis {
    /// Negative native input (-127 after decoding).
    Negative,
    /// No native movement input.
    Stationary,
    /// Positive native input (127 after decoding).
    Positive,
}
impl MovementAxis {
    pub(super) fn mask(self) -> u32 {
        match self {
            Self::Negative => 2,
            Self::Stationary => 0,
            Self::Positive => 1,
        }
    }
}
/// Explicit native melee action, present only when button bit two is set.
/// Yaw is a raw unsigned sixteen-bit turn; decoding interprets its signed short.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UserCommandMelee {
    /// Raw native turn bits, without implicit float rounding or angle conversion.
    pub yaw: u16,
    /// Full native distance byte, without gameplay authorization.
    pub distance: u8,
}
impl fmt::Debug for UserCommandMelee {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserCommandMelee").finish_non_exhaustive()
    }
}
/// Explicit low-level native command fields, before validation and encoding.
/// Native conditional fields must agree with their button bits. These wire IDs
/// and quantized axes are separate from future engine input and resource handles.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct UserCommandInput {
    /// All wrapping signed native timestamp bits.
    pub server_time: i32,
    /// Native twenty-one-bit button word; unknown in-range bits remain inert here.
    pub buttons: u32,
    /// Pitch, yaw and roll as raw unsigned sixteen-bit turns.
    pub angles: [u16; 3],
    /// Native seven-bit weapon identifier.
    pub weapon: u8,
    /// Native seven-bit offhand identifier.
    pub offhand: u8,
    /// Canonical forward and right input axes, in that order.
    pub movement: [MovementAxis; 2],
    /// Two native selected-location bytes, present exactly when button bit sixteen is set.
    pub selected_location: Option<[u8; 2]>,
    /// Native melee fields, present exactly when button bit two is set.
    pub melee: Option<UserCommandMelee>,
}
impl UserCommandInput {
    pub(crate) fn validate(self) -> Result<(), UserCommandEncodeError> {
        if self.buttons >= 0x0020_0000 {
            return Err(UserCommandEncodeError::Buttons);
        }
        if self.weapon >= 128 {
            return Err(UserCommandEncodeError::Weapon);
        }
        if self.offhand >= 128 {
            return Err(UserCommandEncodeError::Offhand);
        }
        if self.selected_location.is_some() != (self.buttons & 0x10000 != 0) {
            return Err(UserCommandEncodeError::SelectedLocation);
        }
        if self.melee.is_some() != (self.buttons & 4 != 0) {
            return Err(UserCommandEncodeError::Melee);
        }
        Ok(())
    }
}
impl fmt::Debug for UserCommandInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UserCommandInput").finish_non_exhaustive()
    }
}
