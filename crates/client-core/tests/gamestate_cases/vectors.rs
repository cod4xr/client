// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, Output, client, configstrings, encoded, entity, finish, mixed};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    gamestate::{GameStateBudget, GameStatePlan, entity_baselines::BaselineWord},
    message_body::BodyReader,
};

#[test]
fn independent_minimal_constituent_owns_its_output_and_stops_before_outer_service() {
    let mut output = Output::new(0, 0, 0, 0);
    let plan = {
        let input = [
            1, 0, 0, 0, 7, 2, 0, 0, 0, 63, 0, 0, 0, 0x44, 0x33, 0x22, 0x11, 7,
        ];
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let plan = GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut [],
            output.storage(),
        )
        .expect("Independent framing");
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.remaining_bytes(), 0);
        plan
    };
    assert_eq!(plan.profile(), BaselineProtocol::Cod4x21);
    assert_eq!(plan.server_command_sequence(), 1);
    assert_eq!(plan.config_data_sequence(), 2);
    assert_eq!(plan.client_index(), 63);
    assert_eq!(plan.checksum_feed(), 0x1122_3344);
    assert_eq!(plan.service_count(), 1);
    assert_eq!(plan.consumed_bytes(), 17);
    assert_eq!(plan.configstrings().len(), 0);
    assert_eq!(plan.configstring_block_count(), 0);
    assert_eq!(plan.entities().count(), 0);
    assert_eq!(plan.client_records().count(), 0);
    assert!(!format!("{plan:?}").contains("287454020"));
}

#[test]
fn mixed_blocks_keep_global_text_offsets_index_order_and_repeated_client_records() {
    let mut output = Output::dirty(9, 4, 3, 3);
    let before = output.clone();
    let plan = {
        let input = encoded(|writer| {
            mixed(writer, 63);
            writer.write_u8(7).expect("Outer EOF");
            writer.write_bits(21, 5).expect("Future cached bits");
        });
        let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
        let plan = GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            BUDGET,
            &mut [0; 5],
            output.storage(),
        )
        .expect("Complete mixed gamestate");
        assert_eq!(reader.read_u8(), Ok(7));
        assert_eq!(reader.read_bits(5), Ok(21));
        assert_eq!(reader.remaining_bytes(), 0);
        plan
    };
    assert_eq!(plan.server_command_sequence(), 0x8000_0001);
    assert_eq!(plan.config_data_sequence(), 0xffff_fffe);
    assert_eq!(plan.checksum_feed(), 0xaabb_ccdd);
    assert_eq!(plan.service_count(), 7);
    assert_eq!(plan.configstring_count(), 3);
    assert_eq!(plan.configstring_block_count(), 2);
    assert_eq!(
        plan.configstrings()
            .map(|entry| entry.starts_block())
            .collect::<Vec<_>>(),
        [true, false, true]
    );
    assert_eq!(plan.text_bytes(), 7);
    assert_eq!(plan.entity_count(), 2);
    assert_eq!(plan.client_record_count(), 2);
    let patches: Vec<_> = plan
        .configstrings()
        .map(|entry| (entry.index(), entry.bytes().to_vec()))
        .collect();
    assert_eq!(
        patches,
        [(0, b"a.".to_vec()), (4883, vec![]), (0, b"later".to_vec())]
    );
    let entities: Vec<_> = plan.entities().collect();
    assert_eq!((entities[0].number(), entities[1].number()), (17, 18));
    assert!(
        entities
            .iter()
            .all(|entity| entity.word(BaselineWord::Kind) == 0)
    );
    let clients: Vec<_> = plan.client_records().collect();
    assert_eq!(clients[0].name(), b"Private.");
    assert_eq!(clients[0].clan_tag(), b"Tag.");
    assert_eq!(clients[1].name(), b"second");
    assert_eq!((clients[0].index(), clients[1].index()), (63, 63));
    let plan_debug = format!("{plan:?}");
    let storage_debug = format!("{:?}", output.storage());
    for debug in [plan_debug, storage_debug] {
        assert!(!debug.contains("Private"));
        assert!(!debug.contains("Tag"));
        assert!(!debug.contains("later"));
        assert!(!debug.contains("2864434397"));
    }
    assert_eq!(output.text, b"a.later\xa5\xa5");
    assert_eq!(output.strings[3], before.strings[3]);
    assert_eq!(output.entities[2], before.entities[2]);
    assert_eq!(output.clients[2], before.clients[2]);
}

#[test]
fn full_native_namespaces_and_more_than_sixty_four_client_records_use_caller_storage() {
    let input = encoded(|writer| {
        writer.write_i32(0).expect("Header");
        let entries: Vec<_> = (0..4884).map(|index| (index, &b""[..])).collect();
        configstrings(writer, &entries);
        for number in 0..1024 {
            entity(writer, number);
        }
        for _ in 0..65 {
            client(writer, 63, b"", b"");
        }
        finish(writer, 0);
    });
    let budget = GameStateBudget {
        max_services: 1091,
        max_configstrings: 4884,
        max_text_bytes: 0,
        max_entities: 1024,
        max_client_records: 65,
    };
    let mut output = Output::new(0, 4884, 1024, 65);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let plan = GameStatePlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        budget,
        &mut [],
        output.storage(),
    )
    .expect("Complete caller-funded namespaces");
    assert_eq!(plan.service_count(), 1091);
    assert_eq!(plan.configstring_count(), 4884);
    assert_eq!(plan.entity_count(), 1024);
    assert_eq!(plan.client_record_count(), 65);
    assert_eq!(
        plan.entities().last().expect("Last baseline").number(),
        1023
    );
    assert_eq!(plan.client_records().count(), 65);
    assert_eq!(reader.remaining_bytes(), 0);
}

#[test]
fn complete_gamestate_above_shared_view_size_has_no_inherited_native_string_arena() {
    let mut input = 0_i32.to_le_bytes().to_vec();
    input.push(2);
    input.extend_from_slice(&20_i32.to_le_bytes());
    for index in 0_i32..20 {
        input.extend_from_slice(&index.to_le_bytes());
        input.extend_from_slice(&[b'x'; 8190]);
        input.push(0);
    }
    input.push(7);
    for word in [42_i32, 0, 0x1234_5678] {
        input.extend_from_slice(&word.to_le_bytes());
    }
    assert!(input.len() > MAX_MESSAGE_BYTES);
    let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
    let mut output = Output::new(20 * 8190, 20, 0, 0);
    let budget = GameStateBudget {
        max_services: 2,
        max_configstrings: 20,
        max_text_bytes: 20 * 8190,
        max_entities: 0,
        max_client_records: 0,
    };
    let plan = GameStatePlan::parse(
        &mut reader,
        BaselineProtocol::Cod4x21,
        budget,
        &mut [0; 8190],
        output.storage(),
    )
    .expect("Large complete gamestate");
    assert_eq!(plan.text_bytes(), 20 * 8190);
    assert_eq!(plan.configstrings().len(), 20);
    assert_eq!(plan.config_data_sequence(), 42);
    assert_eq!(reader.remaining_bytes(), 0);
}
