// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use super::{
    ReliableScheduler, ScheduleDecision, ScheduleError, SchedulePoll, ScheduledDatagram,
    SendAction,
    plan::{PreparedAction, PreparedTimers},
};
use crate::{ReliablePacket, receive::ReceiveWindow, send::SendWindow};

impl ReliableScheduler {
    /// Selects one eligible datagram or a future wake point over explicit monotonic time.
    /// The returned lease holds the selected original fragment and pacing owner through
    /// callback completion. No frame can accumulate a catch-up burst after a long pause.
    ///
    /// # Errors
    /// Rejects backwards time, invalid queue/body state and timer overflow before mutation.
    pub fn poll<'scheduler, 'window, 'buffers>(
        &'scheduler mut self,
        now: Duration,
        outgoing: &'window mut SendWindow<'buffers>,
        incoming: &ReceiveWindow<'_>,
    ) -> Result<SchedulePoll<'scheduler, 'window, 'buffers>, ScheduleError> {
        if now < self.last_now {
            return Err(ScheduleError::ClockRegressed);
        }
        let candidate = outgoing
            .next_transmission_sequence(self.cursor)
            .map_err(ScheduleError::Queue)?;
        let due = candidate
            .map_or(self.next_ack, |_| self.next_data.min(self.next_ack))
            .max(self.next_any);
        if due > now {
            self.last_now = now;
            return Ok(SchedulePoll::WaitUntil(due));
        }
        let data_sequence = candidate.filter(|_| self.next_data <= now);
        let start = if incoming.acknowledge() == self.last_ack_base {
            self.ack_cursor
        } else {
            0
        };
        let receive = incoming.acknowledgement(start);
        let mut header = outgoing
            .acknowledgement_packet(receive)
            .map_err(ScheduleError::Body)?
            .header();
        header.sequence = data_sequence;
        // Validate all copied header fields before borrowing a fragment or offering I/O.
        ReliablePacket::try_new(header, b"").map_err(ScheduleError::Body)?;
        let action = data_sequence.map_or(SendAction::Acknowledgement, |sequence| {
            SendAction::Data { sequence }
        });
        let timers = PreparedTimers {
            any: now
                .checked_add(self.config.minimum_interval)
                .ok_or(ScheduleError::ClockOverflow)?,
            data: if data_sequence.is_some() {
                now.checked_add(self.config.data_interval)
                    .ok_or(ScheduleError::ClockOverflow)?
            } else {
                self.next_data
            },
            acknowledge: now
                .checked_add(self.config.acknowledgement_interval)
                .ok_or(ScheduleError::ClockOverflow)?,
            cursor: data_sequence.map_or(self.cursor, |sequence| sequence + 1),
        };
        let prepared = match data_sequence {
            Some(sequence) => PreparedAction::Data(
                outgoing
                    .prepare_transmission(sequence)
                    .map_err(ScheduleError::Queue)?,
            ),
            None => PreparedAction::Acknowledgement,
        };
        Ok(SchedulePoll::Send(ScheduledDatagram {
            owner: self,
            action: prepared,
            header,
            decision: ScheduleDecision {
                at: now,
                action,
                receive,
            },
            timers,
        }))
    }
}
