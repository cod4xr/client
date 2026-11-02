// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_client_core::{
    BaselineProtocol,
    commands::LegacyClientCommands,
    entities::EntityDecodeContext,
    server_payload::{
        MixedSnapshotAcknowledgementError, MixedSnapshotRequest,
        PreparedMixedSnapshotAcknowledgement, SnapshotDecodeStorage,
    },
    service_controls::{ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage},
    snapshot_clients::{ClientStreamBudget, SnapshotClientSlot},
    snapshot_entities::{EntityStateSource, EntityStreamBudget, SnapshotEntitySlot},
    snapshots::{
        SnapshotBaselines, SnapshotBudget, SnapshotInput, SnapshotMessage, SnapshotOutput,
        mixed_services::MixedSnapshotStorage, services::SnapshotPrediction,
    },
};
use cod4xr_client_net::{ChannelEvent, ClientChannel};
use std::time::{Duration, Instant};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Finals {
    text: Vec<u8>,
    identities: Vec<ClientDataUpdateSlot>,
    entities: Vec<SnapshotEntitySlot>,
    clients: Vec<SnapshotClientSlot>,
}
impl Finals {
    pub fn new() -> Self {
        Self {
            text: vec![0xa5; 128],
            identities: vec![ClientDataUpdateSlot::default(); 4],
            entities: vec![SnapshotEntitySlot::default(); 2],
            clients: vec![SnapshotClientSlot::default(); 2],
        }
    }
}
pub(super) fn receive<'owner, 'history, 'output>(
    channel: &mut ClientChannel<'_>,
    outgoing: &'owner mut LegacyClientCommands<'history>,
    command_sequence: u32,
    finals: &'output mut Finals,
) -> Result<
    PreparedMixedSnapshotAcknowledgement<'owner, 'history, 'output>,
    MixedSnapshotAcknowledgementError,
> {
    let payload_codec = channel.server_decoder().expect("Original channel decoder");
    channel
        .split()
        .receiver
        .receive_until(Instant::now() + Duration::from_secs(3), |event| {
            let ChannelEvent::Message(message) = event else {
                return None;
            };
            // All wire and private decode scratch expires before the host sees the result.
            let mut work = [0; 2048];
            let mut decoded = [0; 2048];
            let mut text = [0; 128];
            let mut updates = vec![ClientDataUpdateSlot::default(); 4];
            Some(
                payload_codec.prepare_mixed_snapshot_ack(
                    outgoing,
                    message.sequence(),
                    message.body(),
                    SnapshotDecodeStorage {
                        work: &mut work,
                        decoded: &mut decoded,
                    },
                    MixedSnapshotRequest {
                        input: SnapshotInput {
                            context: EntityDecodeContext::Cod4x21,
                            message: SnapshotMessage::new(1, message.sequence(), command_sequence)
                                .expect("Host association"),
                            previous: None,
                            baselines: SnapshotBaselines::new(
                                1,
                                BaselineProtocol::Cod4x21,
                                EntityStateSource::from_states(&[]),
                            ),
                            budget: SnapshotBudget {
                                entities: EntityStreamBudget {
                                    max_wire_records: 16,
                                    max_previous_entities: 16,
                                    max_baselines: 0,
                                    max_entities: 4,
                                },
                                clients: ClientStreamBudget {
                                    max_wire_records: 16,
                                    max_previous_clients: 4,
                                    max_clients: 4,
                                },
                            },
                        },
                        services: ClientServiceBudget {
                            max_records: 32,
                            max_client_updates: 4,
                        },
                        prediction: SnapshotPrediction::Transmitted,
                    },
                    MixedSnapshotStorage {
                        text_scratch: &mut text,
                        controls: ClientServiceStorage {
                            command_bytes: &mut finals.text,
                            client_scratch: &mut updates,
                            client_updates: &mut finals.identities,
                        },
                        snapshot: SnapshotOutput {
                            entities: &mut finals.entities,
                            clients: &mut finals.clients,
                        },
                    },
                ),
            )
        })
        .expect("Receive datagram")
        .expect("Before deadline")
        .expect("Ordinary frame")
}
