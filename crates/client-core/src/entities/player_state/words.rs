// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::player_scalars::{PlayerScalars, PlayerWord, word};
use std::fmt;

/// Eleven raw words replaced by the native live prediction/fallback branch.
/// Values remain inert, including nonfinite floats and signed integer bit patterns.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredictionWords {
    origin: [u32; 3],
    velocity: [u32; 3],
    view_angles: [u32; 3],
    bob_cycle: u32,
    movement_direction: u32,
}
impl PredictionWords {
    /// Supplies raw XYZ vectors and full integer words without narrowing or policy.
    #[must_use]
    pub fn new(
        origin: [u32; 3],
        velocity: [u32; 3],
        view_angles: [u32; 3],
        bob_cycle: u32,
        movement_direction: u32,
    ) -> Self {
        Self {
            origin,
            velocity,
            view_angles,
            bob_cycle,
            movement_direction,
        }
    }
    /// Returns raw origin words in XYZ order.
    #[must_use]
    pub fn origin(self) -> [u32; 3] {
        self.origin
    }
    /// Returns raw velocity words in XYZ order.
    #[must_use]
    pub fn velocity(self) -> [u32; 3] {
        self.velocity
    }
    /// Returns raw view-angle words in XYZ order.
    #[must_use]
    pub fn view_angles(self) -> [u32; 3] {
        self.view_angles
    }
    /// Returns the complete bob-cycle word.
    #[must_use]
    pub fn bob_cycle(self) -> u32 {
        self.bob_cycle
    }
    /// Returns the complete movement-direction word.
    #[must_use]
    pub fn movement_direction(self) -> u32 {
        self.movement_direction
    }

    pub(super) fn from_scalars(scalars: &PlayerScalars) -> Self {
        Self::new(
            ORIGIN.map(|key| scalars.word(key)),
            VELOCITY.map(|key| scalars.word(key)),
            ANGLES.map(|key| scalars.word(key)),
            scalars.word(word::motion::BOB_CYCLE),
            scalars.word(word::motion::DIRECTION),
        )
    }
    pub(super) fn apply(self, scalars: &mut PlayerScalars) {
        for (keys, values) in [
            (ORIGIN, self.origin),
            (VELOCITY, self.velocity),
            (ANGLES, self.view_angles),
        ] {
            for (key, value) in keys.into_iter().zip(values) {
                scalars.replace_word(key, value);
            }
        }
        scalars.replace_word(word::motion::BOB_CYCLE, self.bob_cycle);
        scalars.replace_word(word::motion::DIRECTION, self.movement_direction);
    }
}
const ORIGIN: [PlayerWord; 3] = [
    word::motion::ORIGIN_X,
    word::motion::ORIGIN_Y,
    word::motion::ORIGIN_Z,
];
const VELOCITY: [PlayerWord; 3] = [
    word::motion::VELOCITY_X,
    word::motion::VELOCITY_Y,
    word::motion::VELOCITY_Z,
];
const ANGLES: [PlayerWord; 3] = [
    word::view::ANGLE_X,
    word::view::ANGLE_Y,
    word::view::ANGLE_Z,
];
impl fmt::Debug for PredictionWords {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionWords")
            .field("words", &11)
            .finish_non_exhaustive()
    }
}
