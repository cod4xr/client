// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BUDGET, dirty_slots};
use cod4x_protocol::MessageWriter;
use cod4xr_client_core::{
    BaselineProtocol,
    config_state::{ConfigMergeSlot, ConfigPatch, ConfigSnapshot, ConfigValue},
    gamestate::{
        GameStateBudget, GameStatePlan, GameStateStorage, configstrings::ConfigStringSlot,
    },
    message_body::BodyReader,
};

#[test]
fn validated_wire_views_merge_without_retaining_parent_plan_descriptors_or_frame() {
    let mut final_bytes = [0; 16];
    let mut final_slots = dirty_slots(2);
    let mut scratch = [ConfigMergeSlot::default(); 3];
    let result = {
        let mut encoded = [0; 128];
        let mut writer = MessageWriter::try_new(&mut encoded).expect("Wire fixture");
        writer.write_i32(0).expect("Header");
        for (index, value) in [(12, &b"first"[..]), (42, &b"persist"[..])] {
            writer.write_u8(2).expect("Config block");
            writer.write_i32(1).expect("Count");
            writer.write_i32(index).expect("Index");
            for &byte in value {
                writer.write_u8(byte).expect("Value");
            }
            writer.write_u8(0).expect("NUL");
        }
        writer.write_u8(7).expect("EOF");
        for _ in 0..3 {
            writer.write_i32(0).expect("Trailer");
        }
        let mut reader = BodyReader::try_new(writer.as_bytes(), 128).expect("Body");
        let mut decoded = [0; 12];
        let mut strings = [ConfigStringSlot::default(); 2];
        let mut entities = [];
        let mut clients = [];
        let game = GameStatePlan::parse(
            &mut reader,
            BaselineProtocol::Cod4x21,
            GameStateBudget {
                max_services: 3,
                max_configstrings: 2,
                max_text_bytes: 12,
                max_entities: 0,
                max_client_records: 0,
            },
            &mut [0; 7],
            GameStateStorage {
                configstring_bytes: &mut decoded,
                configstring_slots: &mut strings,
                entity_slots: &mut entities,
                client_slots: &mut clients,
            },
        )
        .expect("Whole gamestate");
        let patches: Vec<_> = game
            .configstrings()
            .map(|entry| ConfigPatch::new(entry.index(), entry.bytes(), entry.starts_block()))
            .collect();
        ConfigSnapshot::build(
            &[ConfigValue::new(12, b"base")],
            &patches,
            BUDGET,
            &mut scratch,
            &mut final_bytes,
            &mut final_slots,
        )
        .expect("Immutable source view adapter")
    };
    scratch.fill(ConfigMergeSlot::default());
    assert_eq!(result.value(12), Some(&b"base"[..]));
    assert_eq!(result.value(42), Some(&b"persist"[..]));
    assert_eq!(result.stats().blocks, 2);
}
