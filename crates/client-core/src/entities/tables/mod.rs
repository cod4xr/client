// SPDX-License-Identifier: AGPL-3.0-or-later
use super::fields::{FieldSpec, f};

mod group_0;
mod group_1;
mod group_2;
mod group_3;
mod group_4;
mod group_5;

pub(super) fn select(kind: u32) -> &'static [FieldSpec] {
    match kind {
        0 => &group_0::TABLE_0,
        1 => &group_0::TABLE_1,
        2 => &group_0::TABLE_2,
        3 => &group_1::TABLE_3,
        4 => &group_1::TABLE_4,
        5 => &group_1::TABLE_5,
        6 => &group_2::TABLE_6,
        7 => &group_2::TABLE_7,
        8 => &group_2::TABLE_8,
        9 => &group_3::TABLE_9,
        10 => &group_3::TABLE_10,
        11 => &group_3::TABLE_11,
        12 => &group_4::TABLE_12,
        13 => &group_4::TABLE_13,
        14 => &group_4::TABLE_14,
        15 => &group_5::TABLE_15,
        16 => &group_5::TABLE_16,
        _ => &group_5::TABLE_17,
    }
}
