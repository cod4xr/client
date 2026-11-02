// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, BaselineProtocol, Finals, SERVICES, carrier, parse};

#[test]
fn complete_mixed_carrier_preserves_inner_records_and_outer_control_boundary_after_source_release()
{
    let mut finals = Finals::new();
    let saved = finals.clone();
    let records = {
        let source = carrier(7);
        let plan = parse(
            &source,
            BaselineProtocol::Cod4x21,
            SERVICES,
            BUDGET,
            &mut finals,
        )
        .expect("Complete native carrier");
        assert_eq!(
            (
                plan.prefix().records,
                plan.prefix().commands,
                plan.prefix().client_updates
            ),
            (3, 1, 1)
        );
        let controls = plan.controls();
        assert_eq!(controls.message_sequence(), 7);
        assert_eq!(controls.client_command_acknowledge(), 1);
        assert_eq!(controls.record_count(), 8);
        assert_eq!(controls.logical_bytes(), source.len());
        assert_eq!(
            controls
                .commands()
                .map(|command| (command.sequence(), command.as_bytes()))
                .collect::<Vec<_>>(),
            [(4, b"before.".as_slice()), (9, b"after".as_slice())]
        );
        assert_eq!(controls.client_update_count(), 2);
        assert_eq!(plan.game_state().server_command_sequence(), 0x8000_0001);
        assert_eq!(plan.game_state().config_data_sequence(), 0xffff_fffe);
        assert_eq!(plan.game_state().client_index(), 63);
        assert_eq!(plan.game_state().checksum_feed(), 0xaabb_ccdd);
        assert_eq!(plan.game_state().service_count(), 7);
        let text = format!("{plan:?}");
        for private in ["Private", "before", "2864434397"] {
            assert!(!text.contains(private));
        }
        plan.game_state().records()
    };
    assert_eq!(records.profile(), BaselineProtocol::Cod4x21);
    assert_eq!(
        (
            records.configstring_count(),
            records.configstring_block_count(),
            records.text_bytes(),
            records.entity_count(),
            records.client_record_count()
        ),
        (3, 2, 7, 2, 2)
    );
    assert_eq!(
        records
            .configstrings()
            .map(|value| (value.index(), value.bytes().to_vec(), value.starts_block()))
            .collect::<Vec<_>>(),
        [
            (0, b"a.".to_vec(), true),
            (4883, vec![], false),
            (0, b"later".to_vec(), true)
        ]
    );
    assert_eq!(
        records
            .entities()
            .map(cod4xr_client_core::entities::EntityBaseline::number)
            .collect::<Vec<_>>(),
        [17, 18]
    );
    assert_eq!(records.client_records().count(), 2);
    assert_eq!(&finals.commands[12..], &saved.commands[12..]);
    assert_eq!(&finals.game_state.text[7..], &saved.game_state.text[7..]);
    assert_eq!(
        &finals.game_state.strings[3..],
        &saved.game_state.strings[3..]
    );
    assert_eq!(
        &finals.game_state.entities[2..],
        &saved.game_state.entities[2..]
    );
    assert_eq!(
        &finals.game_state.clients[2..],
        &saved.game_state.clients[2..]
    );
    assert_eq!(&finals.identities[2..], &saved.identities[2..]);
}

#[test]
fn independent_minimal_carrier_requires_both_eofs_and_ignores_padding_after_outer_eof() {
    let source = [
        1, 1, 0, 0, 0, 7, 2, 0, 0, 0, 63, 0, 0, 0, 0x44, 0x33, 0x22, 0x11, 7, 0xfe,
    ];
    let mut finals = Finals::new();
    let plan = parse(
        &source,
        BaselineProtocol::Cod4x21,
        SERVICES,
        BUDGET,
        &mut finals,
    )
    .expect("Independent physical framing");
    assert_eq!(plan.controls().logical_bytes(), 19);
    assert_eq!(plan.controls().record_count(), 2);
    assert_eq!(plan.prefix().records, 0);
    assert_eq!(plan.game_state().consumed_bytes(), 17);
    assert_eq!(plan.game_state().records().configstring_count(), 0);
    assert_eq!(plan.game_state().server_command_sequence(), 1);
    assert_eq!(plan.game_state().config_data_sequence(), 2);
    assert_eq!(plan.game_state().checksum_feed(), 0x1122_3344);
}
