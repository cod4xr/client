// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr, time::Duration};

use super::{ConnectRejection, ConnectReply, ConnectReplyKind, ConnectUsage};

/// Next host I/O action under an unchanged absolute connect deadline.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectAction {
    /// Exactly this request prefix was admitted for transmission.
    Send {
        /// Complete datagram size.
        length: usize,
        /// Earliest retry or phase expiry.
        wake_at: Duration,
    },
    /// Continue receiving without sending until the given retry/expiry time.
    WaitUntil(Duration),
}

/// A selected-peer observation validated against pending connect state and time.
/// Server data remains borrowed, opaque and without filesystem/command authority.
#[derive(Clone, Copy)]
pub struct ConnectObservation<'packet> {
    pub(super) peer: SocketAddr,
    pub(super) protocol: i32,
    pub(super) qport: u16,
    pub(super) reply: ConnectReply<'packet>,
    pub(super) elapsed: Duration,
    pub(super) usage: ConnectUsage,
}

impl<'packet> ConnectObservation<'packet> {
    /// Returns the exact checked endpoint. No redirection was accepted.
    #[must_use]
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    /// Returns the original explicitly requested protocol, not a new negotiation.
    #[must_use]
    pub fn requested_protocol(&self) -> i32 {
        self.protocol
    }

    /// Returns the original routing value for later channel initialization.
    #[must_use]
    pub fn qport(&self) -> u16 {
        self.qport
    }

    /// Returns whether the peer acknowledged, refused or supplied informational data.
    #[must_use]
    pub fn kind(&self) -> ConnectReplyKind {
        self.reply.kind()
    }

    /// Borrows the untrusted label/display message; never execute, open or log it.
    #[must_use]
    pub fn payload(&self) -> &'packet [u8] {
        self.reply.payload()
    }

    /// Returns cumulative elapsed time since phase creation, including all retries.
    #[must_use]
    pub fn elapsed(&self) -> Duration {
        self.elapsed
    }

    /// Returns charged work when this observation was produced.
    #[must_use]
    pub fn usage(&self) -> ConnectUsage {
        self.usage
    }
}

impl fmt::Debug for ConnectObservation<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectObservation")
            .field("kind", &self.reply.kind())
            .field("elapsed", &self.elapsed)
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}

/// One inspected pending-connect packet; acknowledgements are not playable sessions.
#[derive(Clone, Copy, Debug)]
pub enum ConnectProgress<'packet> {
    /// Pending state remains unchanged apart from charged inspection work.
    Ignored(ConnectRejection),
    /// Informational server bytes; the phase remains pending under the same deadline.
    Notice(ConnectObservation<'packet>),
    /// A connectResponse was accepted once; channel/gamestate setup is still required.
    Accepted(ConnectObservation<'packet>),
    /// An error response terminated this attempt without executing native markup.
    Refused(ConnectObservation<'packet>),
}
