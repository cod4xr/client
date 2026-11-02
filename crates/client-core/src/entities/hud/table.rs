// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_hud_table.py.
use super::super::fields::{ValueSpec, v};
use super::HudWord as W;

pub(super) struct FieldSpec {
    pub word: W,
    pub value: ValueSpec,
}

const fn f(word: W, bits: i16) -> FieldSpec {
    FieldSpec {
        word,
        value: v(bits, false),
    }
}

pub(super) const FIELDS: [FieldSpec; 40] = [
    f(W::Color, -85),               // color.rgba
    f(W::FadeStartTime, -97),       // fadeStartTime
    f(W::FromColor, -85),           // fromColor.rgba
    f(W::Y, -91),                   // y
    f(W::Kind, 4),                  // type
    f(W::Material, 8),              // materialIndex
    f(W::Height, 10),               // height
    f(W::Width, 10),                // width
    f(W::X, -92),                   // x
    f(W::FadeTime, 16),             // fadeTime
    f(W::Z, -90),                   // z
    f(W::Value, 0),                 // value
    f(W::AlignScreen, 6),           // alignScreen
    f(W::Sort, 0),                  // sort
    f(W::AlignOrigin, 4),           // alignOrg
    f(W::OffscreenMaterial, 8),     // offscreenMaterialIdx
    f(W::FontScale, -86),           // fontScale
    f(W::Text, 9),                  // text
    f(W::Font, 4),                  // font
    f(W::ScaleStartTime, -97),      // scaleStartTime
    f(W::ScaleTime, 16),            // scaleTime
    f(W::FromWidth, 10),            // fromWidth
    f(W::FromHeight, 10),           // fromHeight
    f(W::TargetEntity, 10),         // targetEntNum
    f(W::GlowColor, -85),           // glowColor.rgba
    f(W::EffectBirthTime, -97),     // fxBirthTime
    f(W::Sound, 5),                 // soundID
    f(W::EffectLetterTime, 12),     // fxLetterTime
    f(W::EffectDecayStartTime, 16), // fxDecayStartTime
    f(W::EffectDecayDuration, 16),  // fxDecayDuration
    f(W::Flags, 3),                 // flags
    f(W::Label, 9),                 // label
    f(W::Time, -97),                // time
    f(W::MoveStartTime, -97),       // moveStartTime
    f(W::MoveTime, 16),             // moveTime
    f(W::FromX, -99),               // fromX
    f(W::FromY, -99),               // fromY
    f(W::FromAlignScreen, 6),       // fromAlignScreen
    f(W::FromAlignOrigin, 4),       // fromAlignOrg
    f(W::Duration, 32),             // duration
];
