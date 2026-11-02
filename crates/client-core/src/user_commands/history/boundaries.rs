// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Reachable boundary states avoid trillions of enqueues."
)]
use super::{InputCommandHistory, InputCommandSlot, InputHistoryError};
use crate::{
    BaselineProtocol,
    user_commands::{MovementAxis, UserCommandInput},
};

#[test]
fn local_sequences_cross_native_words_then_exhaust_without_wrapping_or_overwriting() {
    let input = UserCommandInput {
        server_time: i32::MIN,
        buttons: 0,
        angles: [0; 3],
        weapon: 0,
        offhand: 0,
        movement: [MovementAxis::Stationary; 2],
        selected_location: None,
        melee: None,
    };
    for previous in [u64::from(u32::MAX) - 1, u64::MAX - 2] {
        let mut slots = vec![InputCommandSlot::default(); 3];
        let mut owner =
            InputCommandHistory::new(7, BaselineProtocol::Cod4x21, &mut slots).expect("Owner");
        // Empty after explicitly retiring every earlier input: a reachable state.
        owner.latest = previous;
        owner.transmitted = previous;
        owner.retired = previous;
        let first = owner
            .enqueue(input)
            .expect("Penultimate native-independent ID");
        let last = owner.enqueue(input).expect("Full-width ID");
        assert_eq!(
            (first.sequence(), last.sequence()),
            (previous + 1, previous + 2)
        );
        let summary = owner
            .prepare_range(first, last)
            .expect("Boundary batch")
            .commit_transmission();
        assert_eq!(summary.newly_transmitted, 2);
        assert_eq!(
            owner
                .retire_through(last)
                .expect("Explicit retention policy"),
            2
        );
        if last.sequence() == u64::MAX {
            assert_eq!(
                owner.enqueue(input),
                Err(InputHistoryError::SequenceExhausted)
            );
            assert!(owner.is_empty());
            assert_eq!(owner.latest_id(), Some(last));
        }
    }
}
