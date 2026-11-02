// SPDX-License-Identifier: AGPL-3.0-or-later
// Generated from hash-locked msg.c by tools/import_client_state_table.py.
use super::super::fields::{ValueSpec, v};
use super::ClientStateWord as W;

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

pub(super) const FIELDS: [FieldSpec; 24] = [
    f(W::Model, 9),                // modelindex
    f(W::Name0, 32),               // name[0]
    f(W::Rank, 8),                 // rank
    f(W::Prestige, 8),             // prestige
    f(W::Team, 2),                 // team
    f(W::VehicleEntity, 10),       // attachedVehEntNum
    f(W::Name1, 32),               // name[4]
    f(W::AttachmentModel0, 9),     // attachModelIndex[0]
    f(W::Name2, 32),               // name[8]
    f(W::Perks, 32),               // perks
    f(W::Name3, 32),               // name[12]
    f(W::AttachmentModel1, 9),     // attachModelIndex[1]
    f(W::SprintTimeMultiplier, 0), // maxSprintTimeMultiplier
    f(W::VehicleSeat, 2),          // attachedVehSeat
    f(W::AttachmentTag5, 5),       // attachTagIndex[5]
    f(W::AttachmentTag0, 5),       // attachTagIndex[0]
    f(W::AttachmentTag1, 5),       // attachTagIndex[1]
    f(W::AttachmentTag2, 5),       // attachTagIndex[2]
    f(W::AttachmentTag3, 5),       // attachTagIndex[3]
    f(W::AttachmentTag4, 5),       // attachTagIndex[4]
    f(W::AttachmentModel2, 9),     // attachModelIndex[2]
    f(W::AttachmentModel3, 9),     // attachModelIndex[3]
    f(W::AttachmentModel4, 9),     // attachModelIndex[4]
    f(W::AttachmentModel5, 9),     // attachModelIndex[5]
];
