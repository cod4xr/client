// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, time::Duration};

use super::ReliableScheduler;
use crate::{
    ReliableHeader, ReliablePacket,
    receive::ReceiveAcknowledgement,
    send::{PreparedTransmission, SentFragment},
};

/// One selected datagram family; data always uses its original queue sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendAction {
    /// One first transmission or retransmission of retained original data.
    Data {
        /// Original nonnegative fragment sequence.
        sequence: u32,
    },
    /// One empty native ACK-only body.
    Acknowledgement,
}

/// Copyable scheduling metadata without any payload or mutable admission authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduleDecision {
    /// Explicit host tick used for rate calculations.
    pub at: Duration,
    /// Selected family and optional original sequence.
    pub action: SendAction,
    /// Received cumulative/selective snapshot used by this datagram.
    pub receive: ReceiveAcknowledgement,
}

/// Result after one callback reported complete successful I/O.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ScheduledSend {
    /// Exact selected metadata, including the host tick and carried ACK page.
    pub decision: ScheduleDecision,
    /// Actual data-send admission, absent for ACK-only datagrams.
    pub fragment: Option<SentFragment>,
    /// Complete byte count reported by the successful I/O callback.
    pub bytes: usize,
}

/// A wait observation or an exclusive preflighted datagram lease.
#[derive(Debug)]
#[must_use = "wait for the next tick or explicitly send/drop the prepared datagram"]
pub enum SchedulePoll<'scheduler, 'window, 'buffers> {
    /// No datagram is eligible before this absolute host duration.
    WaitUntil(Duration),
    /// At most one datagram may be admitted at this tick.
    Send(ScheduledDatagram<'scheduler, 'window, 'buffers>),
}

pub(super) enum PreparedAction<'window, 'buffers> {
    Data(PreparedTransmission<'window, 'buffers>),
    Acknowledgement,
}

#[derive(Clone, Copy)]
pub(super) struct PreparedTimers {
    pub any: Duration,
    pub data: Duration,
    pub acknowledge: Duration,
    pub cursor: u32,
}

/// Exclusively borrowed timer and optional outgoing fragment admission.
/// Drop cancels both. A successful callback commits both infallibly.
#[must_use = "send through the callback only after complete I/O; dropping cancels all admission"]
pub struct ScheduledDatagram<'scheduler, 'window, 'buffers> {
    pub(super) owner: &'scheduler mut ReliableScheduler,
    pub(super) action: PreparedAction<'window, 'buffers>,
    pub(super) header: ReliableHeader,
    pub(super) decision: ScheduleDecision,
    pub(super) timers: PreparedTimers,
}

impl ScheduledDatagram<'_, '_, '_> {
    /// Returns metadata only while both pacing and original data remain exclusively held.
    #[must_use]
    pub fn decision(&self) -> ScheduleDecision {
        self.decision
    }

    /// Calls at most once with the checked body and retains its original bytes through I/O.
    /// Return success only after a complete datagram send (or explicit model admission).
    /// A callback error drops both leases without marking transmission or advancing timers.
    /// Successful admission uses this plan's host tick, without catch-up credits.
    ///
    /// # Errors
    /// Propagates the callback's error without changing either held state owner.
    pub fn send_with<E>(
        self,
        send: impl FnOnce(ReliablePacket<'_>) -> Result<usize, E>,
    ) -> Result<ScheduledSend, E> {
        let bytes = {
            let payload = match &self.action {
                PreparedAction::Data(transmission) => transmission.payload(),
                PreparedAction::Acknowledgement => b"",
            };
            // Header preflight and the exclusively held bounded fragment make this
            // construction infallible; the outer envelope/I/O belongs to the callback.
            send(ReliablePacket {
                header: self.header,
                payload,
            })?
        };
        let fragment = match self.action {
            PreparedAction::Data(transmission) => Some(transmission.record_sent()),
            PreparedAction::Acknowledgement => None,
        };
        self.owner.last_now = self.decision.at;
        self.owner.next_any = self.timers.any;
        self.owner.next_data = self.timers.data;
        self.owner.next_ack = self.timers.acknowledge;
        self.owner.cursor = self.timers.cursor;
        self.owner.last_ack_base = self.decision.receive.cumulative;
        self.owner.ack_cursor = self.decision.receive.next_offset;
        Ok(ScheduledSend {
            decision: self.decision,
            fragment,
            bytes,
        })
    }
}

impl fmt::Debug for ScheduledDatagram<'_, '_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ScheduledDatagram")
            .field("decision", &self.decision)
            .finish_non_exhaustive()
    }
}
