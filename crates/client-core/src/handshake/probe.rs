// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr, time::Duration};

use cod4x_protocol::connection::{ChallengeError, ChallengeRequest, ChallengeResponse};

use super::{
    ChallengeBudget, ChallengeFailure, ChallengeReceipt, ChallengeRejection, ChallengeUsage,
};

/// Next bounded I/O action chosen from explicit caller-supplied monotonic time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChallengeAction {
    /// One complete request was encoded. The runtime sends exactly this prefix.
    Send {
        /// Encoded datagram length.
        length: usize,
        /// Earliest retry or absolute expiry after this attempt.
        wake_at: Duration,
    },
    /// No request is due; continue receiving until this retry/expiry point.
    WaitUntil(Duration),
}

/// Result of inspecting a single datagram without transferring socket ownership.
#[derive(Clone, Copy, Debug)]
pub enum ChallengeProgress {
    /// The probe remains pending and retains its original absolute deadline.
    Ignored(ChallengeRejection),
    /// This receipt is delivered once; subsequent probe operations reject.
    Accepted(ChallengeReceipt),
}

/// Deterministic nonce-correlated challenge state for the pinned xproto profile.
/// No socket, wall clock, allocation, authentication, connect or gameplay state is owned.
pub struct ChallengeProbe {
    peer: SocketAddr,
    nonce: i32,
    budget: ChallengeBudget,
    started: Duration,
    last: Duration,
    deadline: Duration,
    next_send: Duration,
    usage: ChallengeUsage,
    terminal: bool,
}

impl ChallengeProbe {
    /// Creates pending state without sending, allocating or resolving any endpoint.
    /// The runtime supplies a fresh random nonce and a monotonic time origin.
    ///
    /// # Errors
    /// Rejects zero budgets and deadline overflow before constructing any state.
    pub fn new(
        peer: SocketAddr,
        nonce: i32,
        now: Duration,
        budget: ChallengeBudget,
    ) -> Result<Self, ChallengeFailure> {
        if budget.timeout.is_zero()
            || budget.retry_interval.is_zero()
            || budget.max_attempts == 0
            || budget.max_datagrams == 0
        {
            return Err(ChallengeFailure::InvalidBudget);
        }
        let deadline = now
            .checked_add(budget.timeout)
            .ok_or(ChallengeFailure::TimeOverflow)?;
        Ok(Self {
            peer,
            nonce,
            budget,
            started: now,
            last: now,
            deadline,
            next_send: now,
            usage: ChallengeUsage::default(),
            terminal: false,
        })
    }

    /// Returns accumulated work and the latest fixed rejection category.
    #[must_use]
    pub fn usage(&self) -> ChallengeUsage {
        self.usage
    }

    /// Admits a due request or returns the next receive-wait deadline.
    /// Reaching the attempt limit still permits replies until absolute expiry.
    /// Request storage and state remain unchanged on output-capacity failure.
    ///
    /// # Errors
    /// Reports terminal/clock/expiry failures and insufficient request storage.
    pub fn poll(
        &mut self,
        now: Duration,
        output: &mut [u8],
    ) -> Result<ChallengeAction, ChallengeFailure> {
        self.check(now)?;
        if self.usage.attempts >= self.budget.max_attempts || now < self.next_send {
            self.last = now;
            return Ok(ChallengeAction::WaitUntil(self.wake_at()));
        }
        let length = ChallengeRequest {
            client_challenge: self.nonce,
        }
        .encode(output)
        .map_err(|_| ChallengeFailure::OutputCapacity)?;
        self.usage.attempts += 1;
        self.last = now;
        self.next_send = now
            .checked_add(self.budget.retry_interval)
            .map_or(self.deadline, |time| time.min(self.deadline));
        Ok(ChallengeAction::Send {
            length,
            wake_at: self.wake_at(),
        })
    }

    /// Checks one received packet against complete endpoint, pending request,
    /// profile, nonce and absolute time. Rejected traffic never extends expiry.
    /// Every inspected datagram is charged, including wrong endpoints.
    ///
    /// # Errors
    /// Expiry and exhausted receive work terminate the probe. Invalid host time
    /// rejects without mutation. A terminal probe cannot publish another receipt.
    pub fn receive(
        &mut self,
        peer: SocketAddr,
        packet: &[u8],
        now: Duration,
    ) -> Result<ChallengeProgress, ChallengeFailure> {
        self.check(now)?;
        let elapsed = now
            .checked_sub(self.started)
            .ok_or(ChallengeFailure::TimeWentBackwards)?;
        self.last = now;
        self.usage.received += 1;
        let result = if self.usage.attempts == 0 {
            Err(ChallengeRejection::Unsolicited)
        } else if peer != self.peer {
            Err(ChallengeRejection::WrongPeer)
        } else {
            self.decode(packet)
        };
        match result {
            Ok(reply) => {
                self.terminal = true;
                Ok(ChallengeProgress::Accepted(ChallengeReceipt {
                    peer,
                    reply,
                    elapsed,
                    usage: self.usage,
                }))
            }
            Err(reason) => {
                self.usage.rejected += 1;
                self.usage.last_rejection = Some(reason);
                if self.usage.received == self.budget.max_datagrams {
                    self.terminal = true;
                    Err(ChallengeFailure::ReceiveBudget)
                } else {
                    Ok(ChallengeProgress::Ignored(reason))
                }
            }
        }
    }

    fn decode(&self, packet: &[u8]) -> Result<ChallengeResponse, ChallengeRejection> {
        let reply = ChallengeResponse::parse_packet(packet).map_err(|error| {
            if error == ChallengeError::UnsupportedProfile {
                ChallengeRejection::UnsupportedProfile
            } else {
                ChallengeRejection::Malformed
            }
        })?;
        if reply.client_challenge != self.nonce {
            return Err(ChallengeRejection::WrongNonce);
        }
        Ok(reply)
    }

    fn check(&mut self, now: Duration) -> Result<(), ChallengeFailure> {
        if self.terminal {
            return Err(ChallengeFailure::Terminal);
        }
        if now < self.last {
            return Err(ChallengeFailure::TimeWentBackwards);
        }
        if now >= self.deadline {
            self.terminal = true;
            return Err(ChallengeFailure::Expired);
        }
        Ok(())
    }

    fn wake_at(&self) -> Duration {
        if self.usage.attempts >= self.budget.max_attempts {
            self.deadline
        } else {
            self.next_send.min(self.deadline)
        }
    }
}

impl fmt::Debug for ChallengeProbe {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChallengeProbe")
            .field("usage", &self.usage)
            .field("terminal", &self.terminal)
            .finish_non_exhaustive()
    }
}
