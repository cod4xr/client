// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    PredictionHistory, PredictionProvenance, PredictionSlot, contexts, retain, sample, word,
};
use crate::snapshots::{
    SnapshotClientSlot, SnapshotEntitySlot,
    history::payload_support::{Attempt, outgoing, request, seed},
    mixed_support::{Finals, carrier},
    values,
};
use cod4xr_client_core::{
    commands::LEGACY_COMMAND_SLOT_BYTES,
    server_payload::MixedSnapshotHistoryRequest,
    snapshots::{
        history::{SnapshotHistory, SnapshotHistorySlot},
        services::SnapshotPrediction,
    },
};

#[test]
fn actual_header_and_retained_prediction_compose_with_original_ack_cancel_retry_and_owned_lifetimes()
 {
    for context in contexts() {
        for distance in [0, 2] {
            let mut raw = [0; 2 * LEGACY_COMMAND_SLOT_BYTES];
            let mut outgoing = outgoing(&mut raw);
            let mut entities = [SnapshotEntitySlot::default()];
            let mut clients = [SnapshotClientSlot::default()];
            let mut slots = [SnapshotHistorySlot::new(&mut entities, &mut clients)];
            let mut snapshots =
                SnapshotHistory::new(7, context.profile(), &mut slots).expect("Snapshots");
            seed(&mut snapshots, context, 5);
            let mut prediction_slots = [PredictionSlot::default()];
            let mut prediction =
                PredictionHistory::new(7, context.profile(), &mut prediction_slots)
                    .expect("Prediction");
            retain(&mut prediction, 0, 999);
            let mut finals = Finals::new();
            let pending = {
                let mut attempt =
                    Attempt::new(&carrier(context.profile(), distance, false), context);
                let selected = MixedSnapshotHistoryRequest {
                    prediction: SnapshotPrediction::History(prediction.source()),
                    ..request(context, &snapshots)
                };
                attempt
                    .prepare(&mut outgoing, selected, &mut finals)
                    .expect("Complete original-owner preparation")
            };
            assert_eq!(
                pending.plan().snapshot().player().prediction(),
                PredictionProvenance::CommandTimeMatched
            );
            assert_eq!(
                pending
                    .plan()
                    .snapshot()
                    .player()
                    .scalars()
                    .word(word::motion::ORIGIN_X),
                999
            );
            // Cooked results retain neither the cache borrow nor decode workspace.
            prediction
                .prepare_replace(7, sample(context.profile()))
                .expect("Replay after decode")
                .commit();
            assert_eq!(
                pending
                    .plan()
                    .snapshot()
                    .player()
                    .scalars()
                    .word(word::motion::ORIGIN_X),
                999
            );
            drop(pending);
            assert_eq!(outgoing.acknowledged_sequence(), 0);
            assert_eq!(snapshots.newest_sequence(), Some(5));
            let mut attempt = Attempt::new(&carrier(context.profile(), distance, false), context);
            let selected = MixedSnapshotHistoryRequest {
                prediction: SnapshotPrediction::History(prediction.source()),
                ..request(context, &snapshots)
            };
            let pending = attempt
                .prepare(&mut outgoing, selected, &mut finals)
                .expect("Retry original key with corrected exact sample");
            let _ = snapshots
                .prepare(pending.plan().snapshot())
                .expect("Sibling snapshot resource preflight")
                .commit();
            let plan = pending.commit();
            prediction
                .reset(8, context.profile())
                .expect("No retained cache borrow");
            assert_eq!(outgoing.acknowledged_sequence(), 1);
            assert_eq!(snapshots.newest_sequence(), Some(7));
            assert_eq!(
                plan.snapshot()
                    .player()
                    .scalars()
                    .word(word::motion::ORIGIN_X),
                1
            );
            assert_eq!(values(plan.snapshot()), (vec![(1, 13)], vec![(63, 511)]));
        }
    }
}
