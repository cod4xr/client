// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    Zone5Budget, Zone5Counts, Zone5Error,
    read::{Reader, word},
};

pub(super) struct Root {
    pub counts: Zone5Counts,
    pub allocate_assets: bool,
}
impl Root {
    pub fn parse(reader: &mut Reader<'_>, budget: Zone5Budget) -> Result<Self, Zone5Error> {
        let root = reader.take(16)?;
        let count = |offset| {
            usize::try_from(i32::from_le_bytes(word(root, offset).to_le_bytes()))
                .map_err(|_| Zone5Error::Count)
        };
        let script_strings = count(0)?;
        let assets = count(8)?;
        if script_strings > budget.max_script_strings {
            return Err(Zone5Error::StringCountBudget);
        }
        if assets > budget.max_assets {
            return Err(Zone5Error::AssetCountBudget);
        }
        for (count, pointer) in [(script_strings, word(root, 4)), (assets, word(root, 12))] {
            if !(pointer == u32::MAX || (pointer == 0 && count == 0)) {
                return Err(Zone5Error::RootLink);
            }
        }
        let minimum = script_strings
            .checked_mul(4)
            .and_then(|n| assets.checked_mul(8).and_then(|bytes| n.checked_add(bytes)))
            .ok_or(Zone5Error::PrefixBudget)?;
        reader.ensure(minimum)?;
        Ok(Self {
            counts: Zone5Counts {
                script_strings,
                assets,
            },
            allocate_assets: word(root, 12) != 0,
        })
    }
}
