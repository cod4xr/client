// SPDX-License-Identifier: AGPL-3.0-or-later
/// Inert native HUD word keys; no pointer, string or resource lookup is performed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum HudWord {
    /// Element type; zero is the native free-slot marker.
    Kind,
    /// X coordinate binary32 bits.
    X,
    /// Y coordinate binary32 bits.
    Y,
    /// Z coordinate binary32 bits.
    Z,
    /// Target entity reference.
    TargetEntity,
    /// Quantized font-scale binary32 bits.
    FontScale,
    /// Font identifier.
    Font,
    /// Origin alignment bits.
    AlignOrigin,
    /// Screen alignment bits.
    AlignScreen,
    /// Packed little-endian RGBA.
    Color,
    /// Previous packed RGBA for a transition.
    FromColor,
    /// Fade start time bits.
    FadeStartTime,
    /// Fade duration.
    FadeTime,
    /// Label configstring identifier.
    Label,
    /// Width value.
    Width,
    /// Height value.
    Height,
    /// Material resource index.
    Material,
    /// Offscreen material resource index.
    OffscreenMaterial,
    /// Previous width for scaling.
    FromWidth,
    /// Previous height for scaling.
    FromHeight,
    /// Scale start time bits.
    ScaleStartTime,
    /// Scale duration.
    ScaleTime,
    /// Previous X binary32 bits.
    FromX,
    /// Previous Y binary32 bits.
    FromY,
    /// Previous origin alignment.
    FromAlignOrigin,
    /// Previous screen alignment.
    FromAlignScreen,
    /// Move start time bits.
    MoveStartTime,
    /// Move duration.
    MoveTime,
    /// Element time bits.
    Time,
    /// Element duration bits.
    Duration,
    /// Display value binary32 bits.
    Value,
    /// Text configstring identifier.
    Text,
    /// Sort value binary32 bits.
    Sort,
    /// Packed glow RGBA.
    GlowColor,
    /// Effect birth time bits.
    EffectBirthTime,
    /// Effect letter timing.
    EffectLetterTime,
    /// Effect decay start timing.
    EffectDecayStartTime,
    /// Effect decay duration.
    EffectDecayDuration,
    /// Sound identifier.
    Sound,
    /// Element flags.
    Flags,
}
