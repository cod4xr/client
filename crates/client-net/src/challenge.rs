// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr, time::Duration};

use cod4xr_client_core::{
    connect::{ConnectBudget, NoAuthConnectPlan, NoAuthConnectSettings},
    handshake::{
        ChallengeAction, ChallengeBudget, ChallengeProbe, ChallengeProgress, ChallengeReceipt,
        ChallengeUsage,
    },
};

use crate::{NetworkError, PendingConnect, datagram, peer::PeerIo};

/// Challenge result and aggregate work, including work charged before failure.
#[derive(Debug)]
pub struct ChallengeRun {
    /// Owns the selected socket on success; failures close it.
    pub outcome: Result<ChallengedSocket, NetworkError>,
    /// Deterministic work counters without nonce/cookie values.
    pub usage: ChallengeUsage,
}

/// Owns a checked challenge receipt and the exact socket that obtained it.
/// Begin connect promptly; the peer controls how long its cookie remains valid.
pub struct ChallengedSocket {
    io: PeerIo,
    receipt: ChallengeReceipt,
}

impl ChallengedSocket {
    /// Returns checked probe metadata. This copy does not transfer the socket.
    #[must_use]
    pub fn receipt(&self) -> ChallengeReceipt {
        self.receipt
    }

    /// Consumes this phase into a pending no-auth request on the same bound socket.
    /// The connect deadline starts here and includes time until `run` is called.
    ///
    /// # Errors
    /// Rejects invalid settings/budgets and unrepresentable deadlines, closing the socket.
    pub fn into_pending(
        self,
        settings: NoAuthConnectSettings<'_>,
        budget: ConnectBudget,
    ) -> Result<PendingConnect, NetworkError> {
        let plan = NoAuthConnectPlan::new(self.receipt, settings).map_err(NetworkError::Plan)?;
        PendingConnect::new(self.io, plan, budget)
    }
}

impl fmt::Debug for ChallengedSocket {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChallengedSocket")
            .field("receipt", &self.receipt)
            .finish_non_exhaustive()
    }
}

/// Obtains a nonce-correlated challenge while retaining socket ownership.
/// This operation blocks under explicit quotas; run it on an I/O worker.
#[must_use]
pub fn challenge(peer: SocketAddr, budget: ChallengeBudget) -> ChallengeRun {
    let mut usage = ChallengeUsage::default();
    let outcome = prepare(peer, budget).and_then(|(mut io, mut probe)| {
        let result = drive(&mut io, &mut probe, budget.timeout);
        usage = probe.usage();
        result.map(|receipt| ChallengedSocket { io, receipt })
    });
    ChallengeRun { outcome, usage }
}

fn prepare(
    peer: SocketAddr,
    budget: ChallengeBudget,
) -> Result<(PeerIo, ChallengeProbe), NetworkError> {
    let mut entropy = [0; 4];
    getrandom::fill(&mut entropy).map_err(NetworkError::Entropy)?;
    let probe = ChallengeProbe::new(peer, i32::from_le_bytes(entropy), Duration::ZERO, budget)
        .map_err(NetworkError::Challenge)?;
    Ok((PeerIo::new(peer)?, probe))
}

fn drive(
    io: &mut PeerIo,
    probe: &mut ChallengeProbe,
    timeout: Duration,
) -> Result<ChallengeReceipt, NetworkError> {
    let deadline = io.at(timeout)?;
    let mut request = [0; 64];
    loop {
        let wake_at = match probe
            .poll(io.started.elapsed(), &mut request)
            .map_err(NetworkError::Challenge)?
        {
            ChallengeAction::Send { length, wake_at } => {
                datagram::send(&io.socket, &request[..length], deadline)?;
                wake_at
            }
            ChallengeAction::WaitUntil(wake_at) => wake_at,
        };
        let wake_at = io.at(wake_at)?;
        if let Some(length) = datagram::receive_until(&io.socket, &mut io.incoming, wake_at)? {
            match probe
                .receive(io.peer, &io.incoming[..length], io.started.elapsed())
                .map_err(NetworkError::Challenge)?
            {
                ChallengeProgress::Accepted(receipt) => return Ok(receipt),
                ChallengeProgress::Ignored(_) => {}
            }
        }
    }
}
