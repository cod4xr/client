// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Output, configstrings, encoded, finish};
use cod4xr_client_core::{BaselineProtocol, gamestate::GameStatePlan, message_body::BodyReader};
use std::collections::BTreeMap;

#[test]
fn original_block_starts_allow_default_restoration_without_losing_other_explicit_values() {
    let input = encoded(|writer| {
        writer.write_i32(0).expect("Header");
        configstrings(writer, &[(12, b"first"), (42, b"persist")]);
        configstrings(writer, &[(100, b"second")]);
        finish(writer, 0);
    });
    let mut output = Output::new(18, 3, 0, 0);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let plan = GameStatePlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        BUDGET,
        &mut [0; 7],
        output.storage(),
    )
    .expect("Native block scopes retained");
    assert_eq!(plan.configstring_block_count(), 2);
    assert_eq!(
        plan.configstrings()
            .map(|entry| entry.starts_block())
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    // Independent native behavior: each block reapplies known defaults, while
    // omitted IDs without a native default keep their earlier explicit value.
    let trusted_defaults = [(12, &b"base"[..])];
    let mut effective = BTreeMap::new();
    for patch in plan.configstrings() {
        if patch.starts_block() {
            for (index, value) in trusted_defaults {
                effective.insert(index, value.to_vec());
            }
        }
        effective.insert(patch.index(), patch.bytes().to_vec());
    }
    assert_eq!(effective.get(&12), Some(&b"base".to_vec()));
    assert_eq!(effective.get(&42), Some(&b"persist".to_vec()));
    assert_eq!(effective.get(&100), Some(&b"second".to_vec()));
}
