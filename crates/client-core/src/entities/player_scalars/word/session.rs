// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_player_scalar_table.py.
//! Native session keys; values remain inert until typed interpretation.
use super::PlayerWord;

/// Key for the native `commandTime` word.
pub const COMMAND_TIME: PlayerWord = PlayerWord(0);

/// Key for the native `eFlags` word.
pub const ENTITY_FLAGS: PlayerWord = PlayerWord(30);

/// Key for the native `otherFlags` word.
pub const OTHER_FLAGS: PlayerWord = PlayerWord(49);

/// Key for the native `perks` word.
pub const PERKS: PlayerWord = PlayerWord(51);

/// Key for the native `killCamEntity` word.
pub const KILL_CAMERA_ENTITY: PlayerWord = PlayerWord(52);

/// Key for the native `clientNum` word.
pub const CLIENT: PlayerWord = PlayerWord(62);

/// Key for the native `deltaTime` word.
pub const DELTA_TIME: PlayerWord = PlayerWord(109);
