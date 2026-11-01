// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_transport::{ReceiveDrop, ReceivedMessage};

/// Fixed reason a selected-peer datagram did not deliver an ordinary message.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ChannelDrop {
    /// Envelope syntax or declared fragment length is invalid.
    Malformed,
    /// Separate reliable transport carried another client's routing value.
    WrongQport,
    /// Shared sequencing/reassembly state rejected the envelope.
    State(ReceiveDrop),
    /// A future shared envelope/outcome is outside this reviewed client boundary.
    Unsupported,
}

/// A non-ordinary envelope, classified before interpreting the fragment flag.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuxiliaryKind {
    /// Out-of-band command bytes, without command execution.
    Connectionless,
    /// Separate reliable transport with the original qport checked.
    Reliable,
    /// Update-proxy bytes, without update handling.
    UpdateProxy,
}

/// Borrowed untrusted auxiliary bytes; classification grants no execution authority.
#[derive(Clone, Copy)]
pub struct AuxiliaryMessage<'packet> {
    pub(crate) kind: AuxiliaryKind,
    pub(crate) body: &'packet [u8],
}

impl<'packet> AuxiliaryMessage<'packet> {
    /// Returns the envelope family only, not an implemented transport/session state.
    #[must_use]
    pub fn kind(&self) -> AuxiliaryKind {
        self.kind
    }
    /// Borrows opaque bytes after the marker and any checked qport.
    #[must_use]
    pub fn body(&self) -> &'packet [u8] {
        self.body
    }
}

impl fmt::Debug for AuxiliaryMessage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuxiliaryMessage")
            .field("kind", &self.kind)
            .field("bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}

/// Borrowed complete ordinary message; the profile-specific body is not decoded.
#[derive(Clone, Copy)]
pub struct ChannelMessage<'message> {
    pub(crate) inner: ReceivedMessage<'message>,
}

impl<'message> ChannelMessage<'message> {
    /// Returns the delivered sequence, excluding the fragment flag.
    #[must_use]
    pub fn sequence(&self) -> u32 {
        self.inner.sequence()
    }
    /// Returns skipped logical sequences, not lost UDP packet count.
    #[must_use]
    pub fn skipped_sequences(&self) -> u32 {
        self.inner.skipped_sequences()
    }
    /// Borrows the undecoded body. Reliable acknowledgements and gameplay need another codec.
    #[must_use]
    pub fn body(&self) -> &'message [u8] {
        self.inner.body()
    }
}

impl fmt::Debug for ChannelMessage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChannelMessage")
            .field("sequence", &self.sequence())
            .field("skipped_sequences", &self.skipped_sequences())
            .field("bytes", &self.body().len())
            .finish_non_exhaustive()
    }
}

/// One inspected datagram. Borrowed data is valid only during the receive callback.
#[derive(Clone, Copy, Debug)]
pub enum ChannelEvent<'message> {
    /// One complete ordinary body, before profile-specific decoding.
    Message(ChannelMessage<'message>),
    /// Full fragment retained; a shorter terminating fragment is still required.
    FragmentPending,
    /// Packet rejected without delivering a message.
    Dropped(ChannelDrop),
    /// Classified opaque bytes for a separately implemented transport/command layer.
    Auxiliary(AuxiliaryMessage<'message>),
}
