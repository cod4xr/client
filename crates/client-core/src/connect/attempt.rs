// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr, time::Duration};

use super::{
    ConnectAction, ConnectBudget, ConnectFailure, ConnectObservation, ConnectProgress,
    ConnectRejection, ConnectReply, ConnectReplyError, ConnectReplyKind, ConnectUsage,
    NoAuthConnectPlan,
};

/// Flat pending-connect state over an owned request and explicit host time.
/// It owns no socket, auth service, content loader or game channel. An accepted
/// acknowledgement still requires subsequent channel and gamestate initialization.
pub struct NoAuthConnectAttempt {
    plan: NoAuthConnectPlan,
    budget: ConnectBudget,
    started: Duration,
    last: Duration,
    deadline: Duration,
    next_send: Duration,
    usage: ConnectUsage,
    terminal: bool,
}

impl NoAuthConnectAttempt {
    /// Admits one bounded pending phase without sending or opening any resource.
    ///
    /// # Errors
    /// Rejects zero durations/quotas and deadline overflow without partial state.
    pub fn new(
        plan: NoAuthConnectPlan,
        now: Duration,
        budget: ConnectBudget,
    ) -> Result<Self, ConnectFailure> {
        if budget.timeout.is_zero()
            || budget.retry_interval.is_zero()
            || budget.max_attempts == 0
            || budget.max_datagrams == 0
        {
            return Err(ConnectFailure::InvalidBudget);
        }
        let deadline = now
            .checked_add(budget.timeout)
            .ok_or(ConnectFailure::TimeOverflow)?;
        Ok(Self {
            plan,
            budget,
            started: now,
            last: now,
            deadline,
            next_send: now,
            usage: ConnectUsage::default(),
            terminal: false,
        })
    }

    /// Returns cumulative work, without exposing request contents or routing IDs.
    #[must_use]
    pub fn usage(&self) -> ConnectUsage {
        self.usage
    }

    /// Returns a due complete request or a future retry/expiry wait point.
    /// Last-attempt replies remain eligible until the original deadline.
    ///
    /// # Errors
    /// Rejects terminal state, clock/expiry errors and insufficient request storage.
    /// Capacity and clock-regression failures leave bytes and state unchanged.
    pub fn poll(
        &mut self,
        now: Duration,
        output: &mut [u8],
    ) -> Result<ConnectAction, ConnectFailure> {
        self.check(now)?;
        if self.usage.attempts >= self.budget.max_attempts || now < self.next_send {
            self.last = now;
            return Ok(ConnectAction::WaitUntil(self.wake_at()));
        }
        let length = self
            .plan
            .encode(output)
            .map_err(|_| ConnectFailure::OutputCapacity)?;
        self.usage.attempts += 1;
        self.last = now;
        self.next_send = now
            .checked_add(self.budget.retry_interval)
            .map_or(self.deadline, |time| time.min(self.deadline));
        Ok(ConnectAction::Send {
            length,
            wake_at: self.wake_at(),
        })
    }

    /// Inspects a bounded reply under the exact peer, pending request and deadline.
    /// Notices do not accept/refuse or extend time. All inspected packets consume
    /// work; the final quota slot may acknowledge or refuse, but cannot remain pending.
    /// No label/message grants filesystem, command, auth or version-fallback authority.
    ///
    /// # Errors
    /// Reports terminal/clock/expiry failures or exhausted inspection work.
    pub fn receive<'packet>(
        &mut self,
        peer: SocketAddr,
        packet: &'packet [u8],
        now: Duration,
    ) -> Result<ConnectProgress<'packet>, ConnectFailure> {
        self.check(now)?;
        let elapsed = now
            .checked_sub(self.started)
            .ok_or(ConnectFailure::TimeWentBackwards)?;
        self.last = now;
        self.usage.received += 1;
        let reply = if self.usage.attempts == 0 {
            Err(ConnectRejection::Unsolicited)
        } else if peer != self.plan.peer() {
            Err(ConnectRejection::WrongPeer)
        } else {
            ConnectReply::parse(packet).map_err(|error| {
                if error == ConnectReplyError::Unsupported {
                    ConnectRejection::Unsupported
                } else {
                    ConnectRejection::Malformed
                }
            })
        };
        let reply = match reply {
            Ok(reply) => reply,
            Err(reason) => {
                self.usage.rejected += 1;
                self.usage.last_rejection = Some(reason);
                self.receive_quota()?;
                return Ok(ConnectProgress::Ignored(reason));
            }
        };
        match reply.kind() {
            ConnectReplyKind::Accepted | ConnectReplyKind::Refused => self.terminal = true,
            ConnectReplyKind::Notice => {
                self.usage.notices += 1;
                self.receive_quota()?;
            }
        }
        let observation = ConnectObservation {
            peer,
            protocol: self.plan.requested_protocol(),
            qport: self.plan.qport(),
            reply,
            elapsed,
            usage: self.usage,
        };
        Ok(match reply.kind() {
            ConnectReplyKind::Accepted => ConnectProgress::Accepted(observation),
            ConnectReplyKind::Refused => ConnectProgress::Refused(observation),
            ConnectReplyKind::Notice => ConnectProgress::Notice(observation),
        })
    }

    fn receive_quota(&mut self) -> Result<(), ConnectFailure> {
        if self.usage.received == self.budget.max_datagrams {
            self.terminal = true;
            Err(ConnectFailure::ReceiveBudget)
        } else {
            Ok(())
        }
    }

    fn check(&mut self, now: Duration) -> Result<(), ConnectFailure> {
        if self.terminal {
            return Err(ConnectFailure::Terminal);
        }
        if now < self.last {
            return Err(ConnectFailure::TimeWentBackwards);
        }
        if now >= self.deadline {
            self.terminal = true;
            return Err(ConnectFailure::Expired);
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

impl fmt::Debug for NoAuthConnectAttempt {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NoAuthConnectAttempt")
            .field("usage", &self.usage)
            .field("terminal", &self.terminal)
            .finish_non_exhaustive()
    }
}
