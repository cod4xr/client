// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::control_messages::CHALLENGE;
use cod4x_protocol::{MessageReader, compression, obfuscation};
use cod4xr_client_core::{
    BaselineProtocol,
    commands::{LEGACY_COMMAND_SLOT_BYTES, LegacyClientCommands},
    control::{ClientControlHeader, ControlMessagePlan, MovementBatch, ServerCommandKey},
    player_state::{PredictionSample, PredictionWords},
    snapshots::SnapshotSource,
    user_commands::{MovementAxis, UserCommandInput, UserCommandMelee},
};

pub(super) const FEED: u32 = 0x8765_4321;
pub(super) const ORIGIN: [u32; 3] = [0x7fc0_1234, 0x8000_0000, 0x3f80_0000];
pub(super) const PREDICTION_TIME: u32 = 0xfeed_cafe;
pub(super) fn header(
    profile: BaselineProtocol,
    acknowledge: u32,
    bytes: &[u8],
) -> ClientControlHeader<'_> {
    let server_command = ServerCommandKey::from_retained(9, bytes).expect("Retained native key");
    match profile {
        BaselineProtocol::Legacy6 => ClientControlHeader::Legacy6 {
            server_id: 0x12,
            message_acknowledge: acknowledge,
            server_command,
        },
        BaselineProtocol::Cod4x21 => ClientControlHeader::Cod4x21 {
            server_id: 0x1122_3344,
            message_acknowledge: acknowledge,
            server_command,
            config_acknowledge: 0xaabb_ccdd,
        },
    }
}
pub(super) fn prediction(profile: BaselineProtocol) -> PredictionSample {
    PredictionSample::new(
        profile,
        PREDICTION_TIME,
        PredictionWords::new(ORIGIN, [100; 3], [200; 3], 3, 4),
    )
}
pub(super) fn commands(count: u8) -> Vec<UserCommandInput> {
    (0..count)
        .map(|index| UserCommandInput {
            server_time: 100 + i32::from(index) * 16,
            buttons: 0x10004 | u32::from(index & 1),
            angles: [u16::from(index) * 997, u16::MAX - u16::from(index), 0x8000],
            weapon: index,
            offhand: 127 - index,
            movement: [MovementAxis::Negative, MovementAxis::Positive],
            selected_location: Some([index, 0x92]),
            melee: Some(UserCommandMelee {
                yaw: 0xc000,
                distance: 255 - index,
            }),
        })
        .collect()
}
pub(super) fn queued<T>(
    commands: &[&[u8]],
    inspect: impl FnOnce(&mut LegacyClientCommands<'_>) -> T,
) -> T {
    let mut bytes = vec![0; (commands.len().max(1) + 1) * LEGACY_COMMAND_SLOT_BYTES];
    let mut history = LegacyClientCommands::new(&mut bytes).expect("Reliable command owner");
    for bytes in commands {
        history.enqueue(bytes).expect("Reliable queued");
    }
    inspect(&mut history)
}
pub(super) fn build(
    profile: BaselineProtocol,
    acknowledge: u32,
    key: &[u8],
    input: &[UserCommandInput],
    reliable: &[&[u8]],
    snapshot: Option<SnapshotSource<'_>>,
) -> ControlMessagePlan {
    queued(reliable, |history| {
        ControlMessagePlan::with_movement(
            header(profile, acknowledge, key),
            CHALLENGE,
            history,
            MovementBatch {
                commands: input,
                checksum_feed: FEED,
                prediction: prediction(profile),
                snapshot,
            },
        )
        .expect("Complete owned movement frame")
    })
}
pub(super) fn body(
    plan: &ControlMessagePlan,
    profile: BaselineProtocol,
    acknowledge: u32,
    key: &[u8],
) -> Vec<u8> {
    let offset = match profile {
        BaselineProtocol::Legacy6 => 9,
        BaselineProtocol::Cod4x21 => 16,
    };
    let mut reader = MessageReader::try_new(&plan.as_bytes()[..offset]).expect("Plaintext header");
    let server_id = match profile {
        BaselineProtocol::Legacy6 => {
            assert_eq!(reader.read_u8().expect("Byte server ID"), 0x12);
            0x12
        }
        BaselineProtocol::Cod4x21 => {
            assert_eq!(
                reader.read_i32().expect("Full server ID").cast_unsigned(),
                0x1122_3344
            );
            0x1122_3344
        }
    };
    assert_eq!(
        reader.read_i32().expect("Message ACK").cast_unsigned(),
        acknowledge
    );
    assert_eq!(reader.read_i32().expect("Command ACK"), 9);
    if profile == BaselineProtocol::Cod4x21 {
        assert_eq!(
            reader.read_i32().expect("Config ACK").cast_unsigned(),
            0xaabb_ccdd
        );
    }
    let mut compressed = plan.as_bytes()[offset..].to_vec();
    obfuscation::apply_client_payload(
        &mut compressed,
        CHALLENGE.cast_unsigned(),
        server_id,
        acknowledge,
        key,
    )
    .expect("Original transform context");
    let mut decoded = [0; 4096];
    let length = compression::decompress(&compressed, &mut decoded)
        .expect("Actual compressed movement body");
    decoded[..length].to_vec()
}
pub(super) fn snapshot<T>(
    profile: BaselineProtocol,
    sequence: u32,
    inspect: impl FnOnce(SnapshotSource<'_>) -> T,
) -> T {
    crate::snapshot_support::with_snapshot(profile, 7, sequence, inspect)
}
