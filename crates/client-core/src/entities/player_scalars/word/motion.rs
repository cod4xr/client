// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_player_scalar_table.py.
//! Native motion keys; values remain inert until typed interpretation.
use super::PlayerWord;

/// Key for the native `origin[0]` word.
pub const ORIGIN_X: PlayerWord = PlayerWord(4);

/// Key for the native `origin[1]` word.
pub const ORIGIN_Y: PlayerWord = PlayerWord(5);

/// Key for the native `bobCycle` word.
pub const BOB_CYCLE: PlayerWord = PlayerWord(6);

/// Key for the native `velocity[1]` word.
pub const VELOCITY_Y: PlayerWord = PlayerWord(7);

/// Key for the native `velocity[0]` word.
pub const VELOCITY_X: PlayerWord = PlayerWord(8);

/// Key for the native `movementDir` word.
pub const DIRECTION: PlayerWord = PlayerWord(9);

/// Key for the native `legsAnim` word.
pub const LEGS_ANIMATION: PlayerWord = PlayerWord(11);

/// Key for the native `origin[2]` word.
pub const ORIGIN_Z: PlayerWord = PlayerWord(12);

/// Key for the native `torsoTimer` word.
pub const TORSO_TIMER: PlayerWord = PlayerWord(15);

/// Key for the native `pm_flags` word.
pub const MOVE_FLAGS: PlayerWord = PlayerWord(16);

/// Key for the native `velocity[2]` word.
pub const VELOCITY_Z: PlayerWord = PlayerWord(19);

/// Key for the native `torsoAnim` word.
pub const TORSO_ANIMATION: PlayerWord = PlayerWord(28);

/// Key for the native `legsTimer` word.
pub const LEGS_TIMER: PlayerWord = PlayerWord(33);

/// Key for the native `sprintState.lastSprintStart` word.
pub const LAST_SPRINT_START: PlayerWord = PlayerWord(35);

/// Key for the native `sprintState.lastSprintEnd` word.
pub const LAST_SPRINT_END: PlayerWord = PlayerWord(36);

/// Key for the native `sprintState.sprintStartMaxLength` word.
pub const SPRINT_START_MAX_LENGTH: PlayerWord = PlayerWord(39);

/// Key for the native `groundEntityNum` word.
pub const GROUND_ENTITY_NUM: PlayerWord = PlayerWord(41);

/// Key for the native `delta_angles[1]` word.
pub const DELTA_ANGLE_Y: PlayerWord = PlayerWord(46);

/// Key for the native `pm_time` word.
pub const MOVE_TIME: PlayerWord = PlayerWord(48);

/// Key for the native `moveSpeedScaleMultiplier` word.
pub const MOVE_SPEED_SCALE_MULTIPLIER: PlayerWord = PlayerWord(50);

/// Key for the native `delta_angles[0]` word.
pub const DELTA_ANGLE_X: PlayerWord = PlayerWord(55);

/// Key for the native `speed` word.
pub const SPEED: PlayerWord = PlayerWord(56);

/// Key for the native `gravity` word.
pub const GRAVITY: PlayerWord = PlayerWord(58);

/// Key for the native `pm_type` word.
pub const MOVE_TYPE: PlayerWord = PlayerWord(78);

/// Key for the native `fTorsoPitch` word.
pub const TORSO_PITCH: PlayerWord = PlayerWord(79);

/// Key for the native `jumpTime` word.
pub const JUMP_TIME: PlayerWord = PlayerWord(82);

/// Key for the native `mantleState.flags` word.
pub const MANTLE_FLAGS: PlayerWord = PlayerWord(83);

/// Key for the native `fWaistPitch` word.
pub const WAIST_PITCH: PlayerWord = PlayerWord(84);

/// Key for the native `proneDirection` word.
pub const PRONE_DIRECTION: PlayerWord = PlayerWord(86);

/// Key for the native `mantleState.timer` word.
pub const MANTLE_TIMER: PlayerWord = PlayerWord(87);

/// Key for the native `sprintState.sprintButtonUpRequired` word.
pub const SPRINT_BUTTON_UP_REQUIRED: PlayerWord = PlayerWord(91);

/// Key for the native `proneTorsoPitch` word.
pub const PRONE_TORSO_PITCH: PlayerWord = PlayerWord(93);

/// Key for the native `sprintState.sprintDelay` word.
pub const SPRINT_DELAY: PlayerWord = PlayerWord(94);

/// Key for the native `proneDirectionPitch` word.
pub const PRONE_DIRECTION_PITCH: PlayerWord = PlayerWord(98);

/// Key for the native `jumpOriginZ` word.
pub const JUMP_ORIGIN_Z: PlayerWord = PlayerWord(99);

/// Key for the native `mantleState.yaw` word.
pub const MANTLE_YAW: PlayerWord = PlayerWord(100);

/// Key for the native `mantleState.transIndex` word.
pub const MANTLE_TRANSITION_INDEX: PlayerWord = PlayerWord(101);

/// Key for the native `vLadderVec[0]` word.
pub const LADDER_X: PlayerWord = PlayerWord(107);

/// Key for the native `vLadderVec[1]` word.
pub const LADDER_Y: PlayerWord = PlayerWord(113);

/// Key for the native `vLadderVec[2]` word.
pub const LADDER_Z: PlayerWord = PlayerWord(122);

/// Key for the native `delta_angles[2]` word.
pub const DELTA_ANGLE_Z: PlayerWord = PlayerWord(124);

/// Key for the native `leanf` word.
pub const LEAN: PlayerWord = PlayerWord(139);
