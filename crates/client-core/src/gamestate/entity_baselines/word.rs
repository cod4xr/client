// SPDX-License-Identifier: AGPL-3.0-or-later
/// Semantic words in the fixed baseline wire state, independent of C struct packing.
/// Union words are interpreted later according to entity kind; no pointer casts occur.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum BaselineWord {
    /// Entity type; selects one of the reviewed field layouts.
    Kind,
    /// Interpolation flags.
    Flags,
    /// Position trajectory type.
    PositionType,
    /// Position trajectory time bits.
    PositionTime,
    /// Position trajectory duration bits.
    PositionDuration,
    /// Position X binary32 bits.
    PositionX,
    /// Position Y binary32 bits.
    PositionY,
    /// Position Z binary32 bits.
    PositionZ,
    /// Position X trajectory delta binary32 bits.
    PositionDeltaX,
    /// Position Y trajectory delta binary32 bits.
    PositionDeltaY,
    /// Position Z trajectory delta binary32 bits.
    PositionDeltaZ,
    /// Angular trajectory type.
    AngleType,
    /// Angular trajectory time bits.
    AngleTime,
    /// Angular trajectory duration bits.
    AngleDuration,
    /// Angular X binary32 bits.
    AngleX,
    /// Angular Y binary32 bits.
    AngleY,
    /// Angular Z binary32 bits.
    AngleZ,
    /// Angular X trajectory delta binary32 bits.
    AngleDeltaX,
    /// Angular Y trajectory delta binary32 bits.
    AngleDeltaY,
    /// Angular Z trajectory delta binary32 bits.
    AngleDeltaZ,
    /// Type-specific union word zero.
    Union0,
    /// Type-specific union word one.
    Union1,
    /// Type-specific union word two.
    Union2,
    /// Type-specific union word three.
    Union3,
    /// Type-specific union word four.
    Union4,
    /// Type-specific union word five.
    Union5,
    /// Type-specific union word six.
    Union6,
    /// Secondary time bits.
    Time2,
    /// Other entity reference.
    OtherEntity,
    /// Attacker entity reference.
    AttackerEntity,
    /// Ground entity reference.
    GroundEntity,
    /// Loop sound identifier.
    LoopSound,
    /// Surface type.
    SurfaceType,
    /// Type-dependent index.
    Index,
    /// Client reference.
    Client,
    /// Head icon identifier.
    HeadIcon,
    /// Head icon team.
    HeadIconTeam,
    /// Packed solid bounds.
    Solid,
    /// Event parameter.
    EventParameter,
    /// Event sequence bits.
    EventSequence,
    /// Event zero.
    Event0,
    /// Event one.
    Event1,
    /// Event two.
    Event2,
    /// Event three.
    Event3,
    /// Event parameter zero.
    EventParameter0,
    /// Event parameter one.
    EventParameter1,
    /// Event parameter two.
    EventParameter2,
    /// Event parameter three.
    EventParameter3,
    /// Weapon identifier.
    Weapon,
    /// Weapon-model identifier.
    WeaponModel,
    /// Leg animation bits.
    LegsAnimation,
    /// Torso animation bits.
    TorsoAnimation,
    /// First type-dependent auxiliary union.
    Auxiliary1,
    /// Second type-dependent auxiliary union.
    Auxiliary2,
    /// Torso pitch binary32 bits.
    TorsoPitch,
    /// Waist pitch binary32 bits.
    WaistPitch,
    /// Part mask zero.
    Part0,
    /// Part mask one.
    Part1,
    /// Part mask two.
    Part2,
    /// Part mask three.
    Part3,
}
