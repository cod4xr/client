// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::user_commands::zero;
use cod4xr_client_core::user_commands::UserCommandInput;
use cod4xr_client_core::{
    BaselineProtocol,
    user_commands::history::{
        InputCommandHistory, InputCommandId, InputCommandSlot, InputHistoryError,
    },
};
use std::collections::VecDeque;
type Record = (InputCommandId, UserCommandInput, bool);

#[test]
fn caller_funded_history_matches_independent_queue_through_cancel_backup_retirement_and_reset() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        for capacity in [1, 3, 17, 257] {
            let mut cells = vec![InputCommandSlot::default(); capacity];
            let mut owner = InputCommandHistory::new(7, profile, &mut cells).expect("Owner");
            let mut model = VecDeque::new();
            let mut latest = None;
            let mut transmitted = None;
            let mut retired = None;
            let mut epoch = 7;
            let mut random = 0x12ab_cdef_u32;
            for step in 0..512 {
                random = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                if step != 0 && step % 73 == 0 {
                    epoch += 1;
                    owner.reset(epoch, profile).expect("New namespace");
                    model.clear();
                    latest = None;
                    transmitted = None;
                    retired = None;
                }
                match random % 3 {
                    0 => {
                        let mut input = zero();
                        input.server_time = random.cast_signed();
                        if model.len() == capacity {
                            assert_eq!(owner.enqueue(input), Err(InputHistoryError::Capacity));
                        } else {
                            let id = owner.enqueue(input).expect("Retained input");
                            model.push_back((id, input, false));
                            latest = Some(id);
                        }
                    }
                    1 if !model.is_empty() => {
                        if let Some(through) = transmit(&mut owner, &mut model, random, transmitted)
                        {
                            transmitted = Some(through);
                        }
                    }
                    2 => {
                        let sent = model.iter().take_while(|(_, _, sent)| *sent).count();
                        if sent != 0 {
                            let count = 1 + usize::try_from(random).expect("Host word") % sent;
                            let id = model[count - 1].0;
                            assert_eq!(
                                owner.retire_through(id).expect("Explicit host retention"),
                                count
                            );
                            for _ in 0..count {
                                model.pop_front();
                            }
                            retired = Some(id);
                        }
                    }
                    _ => {}
                }
                assert_eq!(
                    (owner.len(), owner.pending_count()),
                    (
                        model.len(),
                        model.iter().filter(|(_, _, sent)| !sent).count()
                    )
                );
                assert_eq!(
                    (
                        owner.latest_id(),
                        owner.transmitted_id(),
                        owner.retired_id()
                    ),
                    (latest, transmitted, retired)
                );
                for &(id, input, _) in &model {
                    assert_eq!(
                        owner.command(id).expect("Exact retained circular tag"),
                        input
                    );
                }
                if let Some(id) = retired {
                    assert_eq!(owner.command(id), Err(InputHistoryError::Missing));
                }
            }
        }
    }
}

fn transmit(
    owner: &mut InputCommandHistory<'_>,
    model: &mut VecDeque<Record>,
    random: u32,
    transmitted: Option<InputCommandId>,
) -> Option<InputCommandId> {
    let sent = model.iter().take_while(|(_, _, sent)| *sent).count();
    let backup = sent.min(usize::try_from(random % 4).expect("Bounded backup"));
    let first = sent - backup;
    let count = (model.len() - first).min(32);
    if count == 0 {
        return None;
    }
    let last = model[first + count - 1].0;
    let pending = owner
        .prepare_range(model[first].0, last)
        .expect("Backup and contiguous pending prefix");
    assert_eq!(
        pending.commands(),
        model
            .iter()
            .skip(first)
            .take(count)
            .map(|(_, input, _)| *input)
            .collect::<Vec<_>>()
    );
    if random.is_multiple_of(4) {
        drop(pending);
        return None;
    }
    let expected_through = transmitted
        .filter(|id| id.sequence() > last.sequence())
        .unwrap_or(last);
    let newly = model
        .iter()
        .skip(first)
        .take(count)
        .filter(|(_, _, sent)| !sent)
        .count();
    let summary = pending.commit_transmission();
    assert_eq!(
        (summary.first, summary.last, summary.command_count),
        (model[first].0, last, count)
    );
    assert_eq!(
        (summary.through, summary.newly_transmitted),
        (expected_through, newly)
    );
    for record in model {
        if record.0.sequence() <= expected_through.sequence() {
            record.2 = true;
        }
    }
    Some(expected_through)
}
