// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Output, encoded, failures::rejected, mixed};
use cod4xr_client_core::{BaselineProtocol, gamestate::GameStatePlan, message_body::BodyReader};

#[test]
fn every_short_final_capacity_and_scratch_budget_preserves_reused_outputs() {
    let input = encoded(|writer| mixed(writer, 63));
    for (text, strings, entities, clients, scratch) in (0..7)
        .map(|length| (length, 3, 2, 2, 5))
        .chain((0..3).map(|length| (7, length, 2, 2, 5)))
        .chain((0..2).map(|length| (7, 3, length, 2, 5)))
        .chain((0..2).map(|length| (7, 3, 2, length, 5)))
        .chain((0..5).map(|length| (7, 3, 2, 2, length)))
    {
        rejected(
            &input,
            BUDGET,
            scratch,
            Output::dirty(text, strings, entities, clients),
        );
    }
}

#[test]
fn each_host_work_budget_is_independent_of_storage_and_accepts_its_exact_boundary() {
    let input = encoded(|writer| mixed(writer, 63));
    let mut budgets = vec![];
    for count in 0..7 {
        budgets.push(cod4xr_client_core::gamestate::GameStateBudget {
            max_services: count,
            ..BUDGET
        });
    }
    for count in 0..3 {
        budgets.push(cod4xr_client_core::gamestate::GameStateBudget {
            max_configstrings: count,
            ..BUDGET
        });
    }
    for count in 0..7 {
        budgets.push(cod4xr_client_core::gamestate::GameStateBudget {
            max_text_bytes: count,
            ..BUDGET
        });
    }
    for count in 0..2 {
        budgets.push(cod4xr_client_core::gamestate::GameStateBudget {
            max_entities: count,
            ..BUDGET
        });
    }
    for count in 0..2 {
        budgets.push(cod4xr_client_core::gamestate::GameStateBudget {
            max_client_records: count,
            ..BUDGET
        });
    }
    for budget in budgets {
        rejected(&input, budget, 5, Output::dirty(64, 8, 8, 8));
    }
    let mut output = Output::dirty(7, 3, 2, 2);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let exact = cod4xr_client_core::gamestate::GameStateBudget {
        max_services: 7,
        max_configstrings: 3,
        max_text_bytes: 7,
        max_entities: 2,
        max_client_records: 2,
    };
    let plan = GameStatePlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        exact,
        &mut [0; 5],
        output.storage(),
    )
    .expect("Exact budgets and capacities");
    assert_eq!(plan.service_count(), 7);
    assert_eq!(reader.remaining_bytes(), 0);
}
