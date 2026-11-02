// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, BaselineProtocol, Error, Finals, SERVICES, carrier, parse};
use cod4xr_client_core::service_controls::ServiceControlError;

#[test]
fn independent_outer_and_inner_budgets_reject_before_any_publication() {
    for records in 0..8 {
        let mut services = SERVICES;
        services.max_records = records;
        let mut finals = Finals::new();
        let saved = finals.clone();
        let expected = if records == 0 {
            ServiceControlError::InvalidBudget
        } else {
            ServiceControlError::RecordBudget
        };
        assert_eq!(
            parse(
                &carrier(7),
                BaselineProtocol::Cod4x21,
                services,
                BUDGET,
                &mut finals
            )
            .expect_err("Outer work bounded"),
            Error::Controls(expected)
        );
        assert_eq!(finals, saved);
    }
    let mut variants = Vec::new();
    for field in 0..5 {
        let mut budget = BUDGET;
        match field {
            0 => budget.max_services = 6,
            1 => budget.max_configstrings = 2,
            2 => budget.max_text_bytes = 6,
            3 => budget.max_entities = 1,
            _ => budget.max_client_records = 1,
        }
        variants.push(budget);
    }
    for budget in variants {
        let mut finals = Finals::new();
        let saved = finals.clone();
        assert!(matches!(
            parse(
                &carrier(7),
                BaselineProtocol::Cod4x21,
                SERVICES,
                budget,
                &mut finals
            ),
            Err(Error::GameState(_))
        ));
        assert_eq!(finals, saved);
    }
    let mut services = SERVICES;
    services.max_client_updates = 1;
    let mut finals = Finals::new();
    let saved = finals.clone();
    assert_eq!(
        parse(
            &carrier(7),
            BaselineProtocol::Cod4x21,
            services,
            BUDGET,
            &mut finals
        )
        .expect_err("Identity after gamestate bounded"),
        Error::Controls(ServiceControlError::ClientDataBudget)
    );
    assert_eq!(finals, saved);
}

#[test]
fn each_final_output_capacity_is_independent_and_atomic() {
    for case in 0..6 {
        let mut finals = Finals::new();
        match case {
            0 => finals.game_state.text.truncate(6),
            1 => finals.game_state.strings.truncate(2),
            2 => finals.game_state.entities.truncate(1),
            3 => finals.game_state.clients.truncate(1),
            4 => finals.commands.truncate(11),
            _ => finals.identities.truncate(1),
        }
        let saved = finals.clone();
        parse(
            &carrier(7),
            BaselineProtocol::Cod4x21,
            SERVICES,
            BUDGET,
            &mut finals,
        )
        .expect_err("Caller capacity checked before all writes");
        assert_eq!(finals, saved);
    }
}

#[test]
fn private_workspace_limits_preserve_all_outputs_and_exact_capacities_succeed() {
    let mut short = Vec::new();
    short.extend((0..7).map(|length| (length, 5, 2)));
    short.extend((0..5).map(|length| (7, length, 2)));
    short.extend((0..2).map(|length| (7, 5, length)));
    for scratch in short {
        let mut finals = Finals::new();
        let saved = finals.clone();
        super::parse_with_scratch(
            &carrier(7),
            BaselineProtocol::Cod4x21,
            SERVICES,
            BUDGET,
            &mut finals,
            scratch,
        )
        .expect_err("Independent private workspace bound");
        assert_eq!(finals, saved);
    }
    let mut finals = Finals::new();
    super::parse_with_scratch(
        &carrier(7),
        BaselineProtocol::Cod4x21,
        SERVICES,
        BUDGET,
        &mut finals,
        (7, 5, 2),
    )
    .expect("Exact scratch extents");
}
