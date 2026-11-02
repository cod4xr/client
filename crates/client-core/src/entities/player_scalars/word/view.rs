// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_player_scalar_table.py.
//! Native view keys; values remain inert until typed interpretation.
use super::PlayerWord;

/// Key for the native `viewangles[1]` word.
pub const ANGLE_Y: PlayerWord = PlayerWord(1);

/// Key for the native `viewangles[0]` word.
pub const ANGLE_X: PlayerWord = PlayerWord(2);

/// Key for the native `viewangles[2]` word.
pub const ANGLE_Z: PlayerWord = PlayerWord(3);

/// Key for the native `viewHeightCurrent` word.
pub const HEIGHT_CURRENT: PlayerWord = PlayerWord(31);

/// Key for the native `viewHeightTarget` word.
pub const HEIGHT_TARGET: PlayerWord = PlayerWord(34);

/// Key for the native `viewlocked_entNum` word.
pub const LOCKED_ENTITY: PlayerWord = PlayerWord(57);

/// Key for the native `viewHeightLerpTarget` word.
pub const HEIGHT_LERP_TARGET: PlayerWord = PlayerWord(64);

/// Key for the native `viewHeightLerpDown` word.
pub const HEIGHT_LERP_DOWN: PlayerWord = PlayerWord(71);

/// Key for the native `cursorHint` word.
pub const HINT: PlayerWord = PlayerWord(72);

/// Key for the native `cursorHintString` word.
pub const HINT_STRING: PlayerWord = PlayerWord(73);

/// Key for the native `cursorHintEntIndex` word.
pub const HINT_ENTITY: PlayerWord = PlayerWord(74);

/// Key for the native `viewHeightLerpTime` word.
pub const HEIGHT_LERP_TIME: PlayerWord = PlayerWord(75);

/// Key for the native `radarEnabled` word.
pub const RADAR_ENABLED: PlayerWord = PlayerWord(77);

/// Key for the native `viewlocked` word.
pub const LOCK_TYPE: PlayerWord = PlayerWord(108);

/// Key for the native `viewAngleClampRange[1]` word.
pub const CLAMP_RANGE_Y: PlayerWord = PlayerWord(110);

/// Key for the native `viewAngleClampBase[1]` word.
pub const CLAMP_BASE_Y: PlayerWord = PlayerWord(111);

/// Key for the native `viewAngleClampRange[0]` word.
pub const CLAMP_RANGE_X: PlayerWord = PlayerWord(112);

/// Key for the native `locationSelectionInfo` word.
pub const LOCATION_SELECTION_INFO: PlayerWord = PlayerWord(114);

/// Key for the native `iCompassPlayerInfo` word.
pub const COMPASS_PLAYER_INFO: PlayerWord = PlayerWord(118);

/// Key for the native `viewAngleClampBase[0]` word.
pub const CLAMP_BASE_X: PlayerWord = PlayerWord(135);
