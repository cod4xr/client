// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_objective_table.py.
use super::super::fields::{ValueSpec, v};
use super::ObjectiveWord as W;

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

pub(super) const FIELDS: [FieldSpec; 6] = [
    f(W::OriginX, 0), // origin[0]
    f(W::OriginY, 0), // origin[1]
    f(W::OriginZ, 0), // origin[2]
    f(W::Icon, 12),   // icon
    f(W::Entity, 10), // entNum
    f(W::Team, 4),    // teamNum
];
