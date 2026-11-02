// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{BUDGET, SystemInfo, SystemInfoError};
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    config_state::{
        ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigStateBudget, ConfigStateSlot,
    },
    gamestate::{
        GameStateBudget, GameStatePlan, GameStateStorage, configstrings::ConfigStringSlot,
    },
    message_body::BodyReader,
};

#[test]
fn decoded_wire_system_info_survives_source_release_after_sparse_merge() {
    let mut output = [0; 64];
    let mut slots = [ConfigStateSlot::default()];
    let snapshot = {
        let mut source = [0; 128];
        let mut writer = MessageWriter::try_new(&mut source).expect("Wire fixture");
        writer.write_i32(17).expect("Command sequence");
        writer.write_u8(2).expect("Configstring service");
        writer.write_i32(1).expect("Count");
        writer.write_i32(1).expect("System-info ID");
        for &byte in b"\\sv_serverid\\-1\\fs_game\\inert/path\0" {
            writer.write_u8(byte).expect("System-info value");
        }
        writer.write_u8(7).expect("EOF");
        for value in [23, 0, 0] {
            writer.write_i32(value).expect("Trailer");
        }
        let mut reader = BodyReader::try_new(writer.as_bytes(), 128).expect("Body");
        let mut bytes = [0; 64];
        let mut config_slots = [ConfigStringSlot::default()];
        let game = GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            GameStateBudget {
                max_services: 2,
                max_configstrings: 1,
                max_text_bytes: 64,
                max_entities: 0,
                max_client_records: 0,
            },
            &mut [0; 64],
            GameStateStorage {
                configstring_bytes: &mut bytes,
                configstring_slots: &mut config_slots,
                entity_slots: &mut [],
                client_slots: &mut [],
            },
        )
        .expect("Complete native gamestate constituent");
        let patches: Vec<_> = game
            .configstrings()
            .map(|value| ConfigPatch::new(value.index(), value.bytes(), value.starts_block()))
            .collect();
        ConfigSnapshot::build(
            &[],
            &patches,
            ConfigStateBudget {
                max_input_records: 1,
                max_input_bytes: 64,
                max_records: 1,
                max_text_bytes: 64,
            },
            &mut [ConfigMergeSlot::default()],
            &mut output,
            &mut slots,
        )
        .expect("Source-independent config state")
    };
    let info = SystemInfo::from_snapshot(&snapshot, BUDGET).expect("Effective system info");
    assert_eq!(info.server_id(), u32::MAX);
    assert_eq!(info.field(b"fs_game"), Some(&b"inert/path"[..]));
}

#[test]
fn missing_native_record_is_distinct_from_present_but_invalid_system_info() {
    let mut bytes = [0; 4];
    let mut slots = [ConfigStateSlot::default()];
    let budget = ConfigStateBudget {
        max_input_records: 1,
        max_input_bytes: 4,
        max_records: 1,
        max_text_bytes: 4,
    };
    for (patches, expected) in [
        (&[][..], SystemInfoError::MissingSystemInfo),
        (
            &[ConfigPatch::new(1, b"", true)][..],
            SystemInfoError::MissingServerId,
        ),
    ] {
        let state = ConfigSnapshot::build(
            &[],
            patches,
            budget,
            &mut [ConfigMergeSlot::default()],
            &mut bytes,
            &mut slots,
        )
        .expect("Byte-valid state");
        assert_eq!(
            SystemInfo::from_snapshot(&state, BUDGET).err(),
            Some(expected)
        );
    }
}
