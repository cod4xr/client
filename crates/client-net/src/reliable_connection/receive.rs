// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::Instant;

use super::{ConnectionReceive, ReliableConnection, ReliableRejection};
use crate::{AuxiliaryKind, ChannelEvent, NetworkError};
use cod4xr_reliable::{ReliablePacket, endpoint::EndpointError};

impl ReliableConnection<'_> {
    /// Receives at most one datagram, validating reliable bodies on the original endpoint.
    /// The ordinary/other callback remains scoped to the receive buffer. Original peer/qport
    /// admission is performed by the shared channel before this operation sees a body.
    /// Receipt time is observed after I/O, never supplied by a peer or external caller.
    ///
    /// # Errors
    /// Rejects terminal framing failure before I/O and propagates socket/local-clock errors.
    /// Malformed reliable bodies and receive/ACK failures return bounded rejection events.
    pub fn receive_until<T>(
        &mut self,
        deadline: Instant,
        inspect_other: impl FnOnce(ChannelEvent<'_>) -> T,
    ) -> Result<Option<ConnectionReceive<T>>, NetworkError> {
        self.ensure_active()?;
        let started = self.started;
        let endpoint = &mut self.endpoint;
        let usage = &mut self.usage;
        let result = self
            .channel
            .split()
            .receiver
            .receive_until(deadline, |event| {
                if let ChannelEvent::Auxiliary(message) = event
                    && message.kind() == AuxiliaryKind::Reliable
                {
                    let packet = match ReliablePacket::parse(message.body()) {
                        Ok(packet) => packet,
                        Err(error) => {
                            usage.rejected_datagrams = usage.rejected_datagrams.saturating_add(1);
                            return Ok(ConnectionReceive::Rejected(ReliableRejection::Body(error)));
                        }
                    };
                    let received_at = started.elapsed();
                    return match endpoint.ingest(packet, received_at) {
                        Ok(report) => {
                            usage.accepted_datagrams = usage.accepted_datagrams.saturating_add(1);
                            usage.last_accepted_at = Some(received_at);
                            Ok(ConnectionReceive::Accepted(report))
                        }
                        Err(error @ (EndpointError::Receive(_) | EndpointError::Send(_))) => {
                            usage.rejected_datagrams = usage.rejected_datagrams.saturating_add(1);
                            Ok(ConnectionReceive::Rejected(ReliableRejection::State(error)))
                        }
                        Err(error) => Err(NetworkError::Endpoint(error)),
                    };
                }
                Ok(ConnectionReceive::Other(inspect_other(event)))
            })?;
        result.transpose()
    }
}
