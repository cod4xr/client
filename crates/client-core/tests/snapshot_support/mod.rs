// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent complete empty native snapshot fixture."
)]
use cod4xr_client_core::{
    BaselineProtocol,
    entities::{EntityDecodeContext, LegacyMapCenter},
    message_body::BodyReader,
    snapshot_clients::ClientStreamBudget,
    snapshot_entities::{EntityStateSource, EntityStreamBudget},
    snapshots::{
        SnapshotBaselines, SnapshotBudget, SnapshotInput, SnapshotMessage, SnapshotOutput,
        SnapshotPlan, SnapshotSource,
    },
};

pub(crate) fn with_snapshot<T>(
    profile: BaselineProtocol,
    epoch: u64,
    sequence: u32,
    inspect: impl FnOnce(SnapshotSource<'_>) -> T,
) -> T {
    let bytes = [0, 0, 0, 0, 0, 0, 1, 0, 0x80, 0xff, 3];
    let mut reader =
        BodyReader::try_new(&bytes, bytes.len()).expect("Independent empty native snapshot");
    let context = match profile {
        BaselineProtocol::Cod4x21 => EntityDecodeContext::Cod4x21,
        BaselineProtocol::Legacy6 => {
            EntityDecodeContext::Legacy6(LegacyMapCenter::new([0.0; 3]).expect("Center"))
        }
    };
    let plan = SnapshotPlan::prepare(
        &mut reader,
        SnapshotInput {
            context,
            message: SnapshotMessage::new(epoch, sequence, 9).expect("Admitted message"),
            previous: None,
            baselines: SnapshotBaselines::new(epoch, profile, EntityStateSource::from_states(&[])),
            budget: SnapshotBudget {
                entities: EntityStreamBudget {
                    max_wire_records: 1,
                    max_previous_entities: 0,
                    max_baselines: 0,
                    max_entities: 0,
                },
                clients: ClientStreamBudget {
                    max_wire_records: 1,
                    max_previous_clients: 0,
                    max_clients: 0,
                },
            },
        },
        SnapshotOutput {
            entities: &mut [],
            clients: &mut [],
        },
    )
    .expect("Whole snapshot")
    .commit_transmitted()
    .expect("Complete state");
    inspect(plan.as_source())
}
