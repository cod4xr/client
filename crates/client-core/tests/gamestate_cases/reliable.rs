// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, Output, encoded, mixed};
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{GameStateBudget, ReliableGameStateError, ReliableGameStatePlan},
};

fn body(reserved: i32) -> Vec<u8> {
    encoded(|writer| {
        writer.write_i32(1).expect("Application command");
        writer.write_u8(1).expect("Embedded opcode");
        mixed(writer, 63);
        writer.write_i32(reserved).expect("Reserved checksum word");
        writer
            .write_bits(15, 4)
            .expect("Ignored native partial-byte padding");
    })
}
fn rejected(
    input: &[u8],
    maximum: usize,
    scratch: usize,
    mut output: Output,
) -> ReliableGameStateError {
    let before = output.clone();
    let error = ReliableGameStatePlan::parse(
        input,
        BaselineProtocol::Cod4x21,
        maximum,
        BUDGET,
        &mut vec![0; scratch],
        output.storage(),
    )
    .expect_err("Whole-frame admission");
    assert_eq!(output, before);
    error
}

#[test]
fn independent_reliable_prefix_and_reserved_tail_are_owned_without_activating_fields() {
    let mut output = Output::new(0, 0, 0, 0);
    let plan = {
        let input = [
            1, 0, 0, 0, 1, // Command and embedded opcode; no transport length prefix.
            7, 0, 0, 0, 7, // Server command sequence and inner EOF.
            9, 0, 0, 0, 63, 0, 0, 0, 0x44, 0x33, 0x22, 0x11, // Trailer.
            0xef, 0xcd, 0xab, 0x89, // Opaque reserved word, not an outer EOF.
        ];
        ReliableGameStatePlan::parse(
            &input,
            BaselineProtocol::Cod4x21,
            input.len(),
            BUDGET,
            &mut [],
            output.storage(),
        )
        .expect("Complete independent body")
    };
    assert_eq!(plan.body_bytes(), 26);
    assert_eq!(plan.reserved_checksum_word(), 0x89ab_cdef);
    assert_eq!(plan.game_state().server_command_sequence(), 7);
    assert_eq!(plan.game_state().config_data_sequence(), 9);
    assert_eq!(plan.game_state().checksum_feed(), 0x1122_3344);
    assert_eq!(plan.game_state().client_index(), 63);
    assert_eq!(plan.game_state().service_count(), 1);
    assert!(!format!("{plan:?}").contains("2309737967"));
}

#[test]
fn native_zero_and_opaque_nonzero_tails_accept_only_bounded_inert_bit_padding() {
    for reserved in [0, i32::MIN, i32::MAX, -1] {
        let input = body(reserved);
        let mut output = Output::dirty(7, 3, 2, 2);
        let plan = ReliableGameStatePlan::parse(
            &input,
            BaselineProtocol::Cod4x21,
            input.len(),
            BUDGET,
            &mut [0; 5],
            output.storage(),
        )
        .expect("Native wrapper and ignored cached padding");
        assert_eq!(plan.reserved_checksum_word(), reserved.cast_unsigned());
        assert_eq!(plan.body_bytes(), input.len());
        assert_eq!(plan.game_state().entity_count(), 2);
        assert_eq!(plan.game_state().client_record_count(), 2);
        assert_eq!(plan.game_state().text_bytes(), 7);
        assert!(format!("{plan:?}").contains("unused_cached_bits: 4"));
        assert!(!format!("{plan:?}").contains("Private"));
    }
}

#[test]
fn all_truncations_bad_prefixes_and_extra_bytes_preserve_all_final_storage() {
    let input = body(0);
    for end in 0..input.len() {
        rejected(&input[..end], input.len(), 5, Output::dirty(7, 3, 2, 2));
    }
    for command in [0_i32, 4, 8, 9, -1, i32::MAX] {
        let mut bad = input.clone();
        bad[..4].copy_from_slice(&command.to_le_bytes());
        assert_eq!(
            rejected(&bad, bad.len(), 5, Output::dirty(7, 3, 2, 2)),
            ReliableGameStateError::Command
        );
    }
    for opcode in [0, 2, 7, 255] {
        let mut bad = input.clone();
        bad[4] = opcode;
        assert_eq!(
            rejected(&bad, bad.len(), 5, Output::dirty(7, 3, 2, 2)),
            ReliableGameStateError::Opcode
        );
    }
    for byte in 0..=u8::MAX {
        let mut bad = input.clone();
        bad.push(byte);
        assert_eq!(
            rejected(&bad, bad.len(), 5, Output::dirty(7, 3, 2, 2)),
            ReliableGameStateError::TrailingBytes
        );
    }
}

#[test]
fn whole_body_work_profile_and_caller_capacities_reject_before_publication() {
    let input = body(0);
    for maximum in 0..input.len() {
        assert!(matches!(
            rejected(&input, maximum, 5, Output::dirty(7, 3, 2, 2)),
            ReliableGameStateError::Body(_)
        ));
    }
    for (text, strings, entities, clients, scratch) in [
        (6, 3, 2, 2, 5),
        (7, 2, 2, 2, 5),
        (7, 3, 1, 2, 5),
        (7, 3, 2, 1, 5),
        (7, 3, 2, 2, 4),
    ] {
        rejected(
            &input,
            input.len(),
            scratch,
            Output::dirty(text, strings, entities, clients),
        );
    }
    let mut output = Output::dirty(7, 3, 2, 2);
    let before = output.clone();
    assert!(
        ReliableGameStatePlan::parse(
            &[],
            BaselineProtocol::Legacy6,
            0,
            BUDGET,
            &mut [],
            output.storage()
        )
        .is_err()
    );
    assert_eq!(output, before);
    let budget = GameStateBudget {
        max_services: 6,
        ..BUDGET
    };
    assert!(
        ReliableGameStatePlan::parse(
            &input,
            BaselineProtocol::Cod4x21,
            input.len(),
            budget,
            &mut [0; 5],
            output.storage()
        )
        .is_err()
    );
    assert_eq!(output, before);
}

#[test]
fn complete_large_reliable_body_uses_shared_views_and_does_not_ignore_late_extra_bytes() {
    let mut input = 1_i32.to_le_bytes().to_vec();
    input.push(1);
    input.extend_from_slice(&0_i32.to_le_bytes());
    input.push(2);
    input.extend_from_slice(&20_i32.to_le_bytes());
    for index in 0_i32..20 {
        input.extend_from_slice(&index.to_le_bytes());
        input.extend_from_slice(&[b'x'; 8190]);
        input.push(0);
    }
    input.push(7);
    for word in [42_i32, 0, 0x1234_5678, 0] {
        input.extend_from_slice(&word.to_le_bytes());
    }
    let budget = GameStateBudget {
        max_services: 2,
        max_configstrings: 20,
        max_text_bytes: 20 * 8190,
        max_entities: 0,
        max_client_records: 0,
    };
    let mut output = Output::new(20 * 8190, 20, 0, 0);
    let before = output.clone();
    input.push(0);
    assert_eq!(
        ReliableGameStatePlan::parse(
            &input,
            BaselineProtocol::Cod4x21,
            input.len(),
            budget,
            &mut [0; 8190],
            output.storage()
        )
        .expect_err("Late extra physical byte"),
        ReliableGameStateError::TrailingBytes
    );
    assert_eq!(output, before);
    input.pop();
    let plan = ReliableGameStatePlan::parse(
        &input,
        BaselineProtocol::Cod4x21,
        input.len(),
        budget,
        &mut [0; 8190],
        output.storage(),
    )
    .expect("Large complete reliable application body");
    assert_eq!(plan.game_state().text_bytes(), 20 * 8190);
    assert_eq!(plan.game_state().config_data_sequence(), 42);
}
