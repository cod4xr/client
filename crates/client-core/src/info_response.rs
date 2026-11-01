// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::{Direction, InfoLimit, InfoString, Packet};

use crate::{DiscoveryError, QueryChallenge};

/// Maximum normal-info response including header, command and optional terminal NUL.
pub const MAX_INFO_DATAGRAM_BYTES: usize = 4 + 13 + 1023 + 1;

/// Checked borrowed metadata; remote byte strings remain bytes rather than forced UTF-8.
/// An advertised protocol is not proof of the server's gameplay/authentication support.
pub struct InfoResponse<'packet> {
    info: InfoString<'packet>,
    protocol: u32,
}

impl<'packet> InfoResponse<'packet> {
    /// Validates the complete response and exactly one matching challenge.
    /// Accepts a terminal C NUL for peers that send it; embedded NULs/trailing data reject.
    /// Unknown fields and byte encodings are retained by the shared info codec.
    ///
    /// # Errors
    /// Reports fixed envelope/command/info/correlation/protocol failures without contents.
    pub fn parse(packet: &'packet [u8], challenge: QueryChallenge) -> Result<Self, DiscoveryError> {
        if packet.len() > MAX_INFO_DATAGRAM_BYTES {
            return Err(DiscoveryError::InvalidEnvelope);
        }
        let Packet::Connectionless(body) = Packet::parse(packet, Direction::ServerToClient)
            .map_err(|_| DiscoveryError::InvalidEnvelope)?
        else {
            return Err(DiscoveryError::UnexpectedResponse);
        };
        let end = body
            .iter()
            .position(|&byte| byte == b'\n')
            .ok_or(DiscoveryError::UnexpectedResponse)?;
        let (command, rest) = body.split_at(end);
        let info = &rest[1..];
        if !command.eq_ignore_ascii_case(b"infoResponse") {
            return Err(DiscoveryError::UnexpectedResponse);
        }
        let info = info.strip_suffix(&[0]).unwrap_or(info);
        let info =
            InfoString::parse(info, InfoLimit::Normal).map_err(|_| DiscoveryError::InvalidInfo)?;
        let mut challenges = info
            .pairs()
            .filter(|pair| pair.key().eq_ignore_ascii_case(b"challenge"));
        let echoed = challenges.next().ok_or(DiscoveryError::ChallengeMismatch)?;
        if challenges.next().is_some() {
            return Err(DiscoveryError::AmbiguousChallenge);
        }
        if echoed.value() != challenge.bytes() {
            return Err(DiscoveryError::ChallengeMismatch);
        }
        let protocol = info
            .get(b"protocol")
            .and_then(decimal)
            .ok_or(DiscoveryError::InvalidProtocol)?;
        Ok(Self { info, protocol })
    }

    /// Returns the discovery advertisement, without inferring gameplay compatibility.
    #[must_use]
    pub fn advertised_protocol(&self) -> u32 {
        self.protocol
    }

    /// Returns a case-insensitive field's first value without allocation or UTF-8 coercion.
    #[must_use]
    pub fn field(&self, key: &[u8]) -> Option<&'packet [u8]> {
        self.info.get(key)
    }

    /// Borrows the complete validated info string including unknown fields.
    #[must_use]
    pub fn info(&self) -> InfoString<'packet> {
        self.info
    }
}

fn decimal(bytes: &[u8]) -> Option<u32> {
    if bytes.is_empty() {
        return None;
    }
    bytes.iter().try_fold(0_u32, |value, &byte| {
        if !byte.is_ascii_digit() {
            return None;
        }
        value.checked_mul(10)?.checked_add(u32::from(byte - b'0'))
    })
}

impl fmt::Debug for InfoResponse<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("InfoResponse")
            .field("advertised_protocol", &self.protocol)
            .field("encoded_bytes", &self.info.as_bytes().len())
            .finish_non_exhaustive()
    }
}
