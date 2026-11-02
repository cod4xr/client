// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Duration;

use super::{EndpointError, ReliableEndpoint};
use crate::{ReliablePacket, receive::ReceiveDisposition, send::AcknowledgementUpdate};

/// Fully committed incoming packet effects, without retained payload contents.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IngestReport {
    /// Incoming original-data admission or ACK-only/duplicate/old disposition.
    pub incoming: ReceiveDisposition,
    /// Whole outgoing cumulative/selective ACK update.
    pub acknowledgement: AcknowledgementUpdate,
    /// Whether a completely admitted packet requested earlier rate-limited ACK eligibility.
    pub force_ack: bool,
}

impl ReliableEndpoint<'_> {
    /// Preflights clock/force-ACK, receive data and outgoing ACK effects before any commit.
    /// Caller must already admit the selected peer/profile/qport and complete body syntax.
    ///
    /// # Errors
    /// Rejects a failed endpoint, clock regression or either window's whole-packet failure
    /// without changing any endpoint bytes, counters, timer or acknowledgement state.
    pub fn ingest(
        &mut self,
        packet: ReliablePacket<'_>,
        now: Duration,
    ) -> Result<IngestReport, EndpointError> {
        self.ensure_active()?;
        let force_ack = packet.header().force_ack();
        let observed = self
            .scheduler
            .prepare_packet_observation(now, force_ack)
            .map_err(EndpointError::Schedule)?;
        let incoming = self
            .incoming
            .prepare(packet)
            .map_err(EndpointError::Receive)?;
        let acknowledged = self
            .outgoing
            .prepare_acknowledgement(packet)
            .map_err(EndpointError::Send)?;
        let acknowledgement = acknowledged.commit();
        let incoming = incoming.commit();
        observed.commit();
        Ok(IngestReport {
            incoming,
            acknowledgement,
            force_ack,
        })
    }
}
