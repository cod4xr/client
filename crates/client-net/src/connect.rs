// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, time::Instant};

use cod4xr_client_core::connect::{
    ConnectAction, ConnectBudget, ConnectProgress, ConnectUsage, MAX_CONNECT_PACKET_BYTES,
    NoAuthConnectAttempt, NoAuthConnectPlan,
};

use crate::{
    AcknowledgedSocket, ConnectOutcome, ConnectSummary, NetworkError, datagram, peer::PeerIo,
};

/// Terminal connect result with work charged even if the operation failed.
#[derive(Debug)]
pub struct ConnectRun {
    /// Acknowledgement retains the socket; refusal/failure closes it.
    pub outcome: Result<ConnectOutcome, NetworkError>,
    /// Aggregate work without private request/reply data.
    pub usage: ConnectUsage,
}

/// Owns a pending request, deadline and the socket used to obtain its challenge.
pub struct PendingConnect {
    io: PeerIo,
    attempt: NoAuthConnectAttempt,
    deadline: Instant,
}

enum TerminalReply {
    Acknowledged(ConnectSummary),
    Refused(ConnectSummary),
}

impl PendingConnect {
    pub(crate) fn new(
        io: PeerIo,
        plan: NoAuthConnectPlan,
        budget: ConnectBudget,
    ) -> Result<Self, NetworkError> {
        let now = io.started.elapsed();
        let attempt =
            NoAuthConnectAttempt::new(plan, now, budget).map_err(NetworkError::Connect)?;
        let deadline = io.at(now
            .checked_add(budget.timeout)
            .ok_or(NetworkError::ClockOverflow)?)?;
        Ok(Self {
            io,
            attempt,
            deadline,
        })
    }

    /// Drives this phase to acknowledgement, refusal or bounded failure.
    /// Notices consume work but are not retained or executed. No gameplay channel
    /// is initialized by this blocking operation; run it on an I/O worker.
    #[must_use]
    pub fn run(mut self) -> ConnectRun {
        let result = self.drive();
        let usage = self.attempt.usage();
        let outcome = result.map(|reply| match reply {
            TerminalReply::Acknowledged(summary) => {
                ConnectOutcome::Acknowledged(AcknowledgedSocket::new(self.io, summary))
            }
            TerminalReply::Refused(summary) => ConnectOutcome::Refused(summary),
        });
        ConnectRun { outcome, usage }
    }

    fn drive(&mut self) -> Result<TerminalReply, NetworkError> {
        let mut request = [0; MAX_CONNECT_PACKET_BYTES];
        loop {
            let wake_at = match self
                .attempt
                .poll(self.io.started.elapsed(), &mut request)
                .map_err(NetworkError::Connect)?
            {
                ConnectAction::Send { length, wake_at } => {
                    datagram::send(&self.io.socket, &request[..length], self.deadline)?;
                    wake_at
                }
                ConnectAction::WaitUntil(wake_at) => wake_at,
            };
            let wake_at = self.io.at(wake_at)?;
            if let Some(length) =
                datagram::receive_until(&self.io.socket, &mut self.io.incoming, wake_at)?
            {
                match self
                    .attempt
                    .receive(
                        self.io.peer,
                        &self.io.incoming[..length],
                        self.io.started.elapsed(),
                    )
                    .map_err(NetworkError::Connect)?
                {
                    ConnectProgress::Accepted(observation) => {
                        return Ok(TerminalReply::Acknowledged(ConnectSummary::capture(
                            observation,
                            &mut self.io.terminal,
                        )?));
                    }
                    ConnectProgress::Refused(observation) => {
                        return Ok(TerminalReply::Refused(ConnectSummary::capture(
                            observation,
                            &mut self.io.terminal,
                        )?));
                    }
                    ConnectProgress::Ignored(_) | ConnectProgress::Notice(_) => {}
                }
            }
        }
    }
}

impl fmt::Debug for PendingConnect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PendingConnect")
            .field("usage", &self.attempt.usage())
            .finish_non_exhaustive()
    }
}
