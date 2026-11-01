// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::SocketAddr};

use cod4x_protocol::{InfoLimit, InfoString, InfoWriter, MAX_COMMAND_BYTES, Packet};

use super::{ConnectPlanError, NoAuthConnectSettings, number_text::NumberText};
use crate::handshake::ChallengeReceipt;

/// Complete normal legacy command plus its four-byte connectionless envelope.
/// This is a wire profile boundary, not a limit on internal content or game storage.
pub const MAX_CONNECT_PACKET_BYTES: usize = MAX_COMMAND_BYTES + 4;
const FRAMING_BYTES: usize = 10; // connect, space and two quote delimiters.

/// Immutable owned no-auth connect datagram, bound to an already checked peer.
/// This builder opens no socket, invokes no auth service and admits no player.
/// The caller still owns retransmission, expiry and acceptance-state policy.
pub struct NoAuthConnectPlan {
    peer: SocketAddr,
    protocol: i32,
    qport: u16,
    packet: [u8; MAX_CONNECT_PACKET_BYTES],
    length: usize,
}

impl NoAuthConnectPlan {
    /// Builds owned wire bytes from a checked challenge and explicit compatibility settings.
    /// Unknown nonempty fields retain values with canonical shared info-string encoding;
    /// empty fields follow the shared setter's deletion rule and are omitted.
    /// Critical fields and all case-insensitive duplicates reject before publication.
    /// No dummy GUID, authorization token or client identity is fabricated.
    ///
    /// # Errors
    /// Rejects invalid settings, conflicting/duplicate fields and legacy framing capacity.
    pub fn new(
        receipt: ChallengeReceipt,
        settings: NoAuthConnectSettings<'_>,
    ) -> Result<Self, ConnectPlanError> {
        if settings.protocol <= 0 || settings.version.is_empty() {
            return Err(ConnectPlanError::InvalidSettings);
        }
        validate_userinfo(settings.userinfo)?;
        let protocol = NumberText::new(settings.protocol)?;
        let cookie = NumberText::new(receipt.server_challenge())?;
        let signed_qport = i16::from_le_bytes(settings.qport.to_le_bytes());
        let qport = NumberText::new(i32::from(signed_qport))?;
        // Reserve C-string scratch independently of the bytes copied onto the wire.
        let mut storage = [0; MAX_COMMAND_BYTES - FRAMING_BYTES + 1];
        let mut info = InfoWriter::new(&mut storage, InfoLimit::Normal);
        for pair in settings.userinfo.pairs() {
            info.set(pair.key(), pair.value())
                .map_err(|_| ConnectPlanError::UserinfoEncoding)?;
        }
        for (key, value) in [
            (b"protocol".as_slice(), protocol.bytes()),
            (b"challenge", cookie.bytes()),
            (b"qport", qport.bytes()),
            (b"xver", settings.version),
        ] {
            info.set(key, value)
                .map_err(|_| ConnectPlanError::UserinfoEncoding)?;
        }
        if info.as_bytes().len() > MAX_COMMAND_BYTES - FRAMING_BYTES {
            return Err(ConnectPlanError::UserinfoEncoding);
        }
        let prefix = b"connect \"";
        let end = prefix.len() + info.as_bytes().len();
        let mut body = [0; MAX_COMMAND_BYTES];
        body[..prefix.len()].copy_from_slice(prefix);
        body[prefix.len()..end].copy_from_slice(info.as_bytes());
        body[end] = b'"';
        let mut packet = [0; MAX_CONNECT_PACKET_BYTES];
        let length = Packet::Connectionless(&body[..=end])
            .encode(&mut packet)
            .map_err(|_| ConnectPlanError::Encoding)?;
        Ok(Self {
            peer: receipt.peer(),
            protocol: settings.protocol,
            qport: settings.qport,
            packet,
            length,
        })
    }

    /// Returns the exact peer from the checked receipt; userinfo cannot redirect it.
    #[must_use]
    pub fn peer(&self) -> SocketAddr {
        self.peer
    }

    /// Returns the explicit requested protocol, not an independently negotiated value.
    #[must_use]
    pub fn requested_protocol(&self) -> i32 {
        self.protocol
    }

    /// Returns the configured 16-bit routing value for subsequent channel setup.
    #[must_use]
    pub fn qport(&self) -> u16 {
        self.qport
    }

    /// Returns complete wire size without exposing private bytes.
    #[must_use]
    pub fn encoded_len(&self) -> usize {
        self.length
    }

    /// Copies the complete datagram. Failure preserves every output byte.
    /// Contents can include private caller userinfo and must never enter logs.
    ///
    /// # Errors
    /// Returns output capacity failure before copying or publishing a partial frame.
    pub fn encode(&self, output: &mut [u8]) -> Result<usize, ConnectPlanError> {
        let destination = output
            .get_mut(..self.length)
            .ok_or(ConnectPlanError::OutputCapacity)?;
        destination.copy_from_slice(&self.packet[..self.length]);
        Ok(self.length)
    }
}

fn validate_userinfo(info: InfoString<'_>) -> Result<(), ConnectPlanError> {
    let reserved: [&[u8]; 4] = [b"protocol", b"challenge", b"qport", b"xver"];
    for (index, pair) in info.pairs().enumerate() {
        if reserved
            .iter()
            .any(|key| pair.key().eq_ignore_ascii_case(key))
        {
            return Err(ConnectPlanError::ReservedField);
        }
        if info
            .pairs()
            .take(index)
            .any(|previous| previous.key().eq_ignore_ascii_case(pair.key()))
        {
            return Err(ConnectPlanError::DuplicateField);
        }
    }
    Ok(())
}

impl fmt::Debug for NoAuthConnectPlan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NoAuthConnectPlan")
            .field("encoded_bytes", &self.length)
            .finish_non_exhaustive()
    }
}
