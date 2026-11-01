// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::{CommandLine, Direction, InfoLimit, Packet};

use super::ConnectReplyError;

/// Largest selected reply: OOB marker, print command, text and optional C NUL.
pub const MAX_CONNECT_REPLY_BYTES: usize = 4 + 6 + MAX_NOTICE_BYTES + 1;
const MAX_NOTICE_BYTES: usize = 8191;

/// Shape of an untrusted connectionless reply; parsing alone does not advance state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectReplyKind {
    /// connectResponse with an optional untrusted mod-directory label.
    Accepted,
    /// error followed by opaque diagnostic bytes, without evaluating native markup.
    Refused,
    /// print followed by informational bytes; it is not admission or refusal.
    Notice,
}

/// Bounded borrowed reply data with no endpoint, pending-state or time authority.
/// The payload is a label or display message, never a filesystem path or command.
#[derive(Clone, Copy)]
pub struct ConnectReply<'packet> {
    kind: ConnectReplyKind,
    payload: &'packet [u8],
}

impl<'packet> ConnectReply<'packet> {
    /// Parses complete OOB connect/error/print shapes using shared primitives.
    /// Mod labels retain bytes without opening paths. Error/print text may contain
    /// line breaks or native markup, but nothing here interprets or executes it.
    /// The selected error/print limits follow the pinned client destination buffers.
    ///
    /// # Errors
    /// Rejects envelope/header/field/trailing-data errors and over-limit text.
    pub fn parse(packet: &'packet [u8]) -> Result<Self, ConnectReplyError> {
        if packet.len() > MAX_CONNECT_REPLY_BYTES {
            return Err(ConnectReplyError::Capacity);
        }
        let Packet::Connectionless(body) = Packet::parse(packet, Direction::ServerToClient)
            .map_err(|_| ConnectReplyError::Envelope)?
        else {
            return Err(ConnectReplyError::Envelope);
        };
        let line = CommandLine::parse(body).map_err(|_| ConnectReplyError::Syntax)?;
        let mut tokens = line.tokens();
        let command = tokens.next().ok_or(ConnectReplyError::Syntax)?;
        if command.eq_ignore_ascii_case(b"connectResponse") {
            let payload = tokens.next().unwrap_or(b"");
            if tokens.next().is_some() {
                return Err(ConnectReplyError::Syntax);
            }
            if line.consumed_bytes() != body.len() {
                return Err(ConnectReplyError::TrailingData);
            }
            return Ok(Self {
                kind: ConnectReplyKind::Accepted,
                payload,
            });
        }
        let (kind, maximum) = if command.eq_ignore_ascii_case(b"error") {
            (ConnectReplyKind::Refused, InfoLimit::Normal.maximum_bytes())
        } else if command.eq_ignore_ascii_case(b"print") {
            (ConnectReplyKind::Notice, MAX_NOTICE_BYTES)
        } else {
            return Err(ConnectReplyError::Unsupported);
        };
        if tokens.next().is_some() {
            return Err(ConnectReplyError::Syntax);
        }
        let text = &body[line.consumed_bytes()..];
        let text = text.strip_suffix(&[0]).unwrap_or(text);
        if text.len() > maximum {
            return Err(ConnectReplyError::Capacity);
        }
        if text.contains(&0) {
            return Err(ConnectReplyError::TrailingData);
        }
        Ok(Self {
            kind,
            payload: text,
        })
    }

    /// Returns the wire shape without confirming the selected peer or pending phase.
    #[must_use]
    pub fn kind(&self) -> ConnectReplyKind {
        self.kind
    }

    /// Borrows untrusted label/display bytes. Never execute or open this as a path.
    /// Bytes can contain private diagnostics and must not enter operational logs.
    #[must_use]
    pub fn payload(&self) -> &'packet [u8] {
        self.payload
    }
}

impl fmt::Debug for ConnectReply<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ConnectReply")
            .field("kind", &self.kind)
            .field("payload_bytes", &self.payload.len())
            .finish_non_exhaustive()
    }
}
