// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{fmt, net::UdpSocket, time::Instant};

use cod4x_protocol::{Direction, Packet};
use cod4x_transport::{ReceiveChannel, ReceiveOutcome};

use crate::{
    AuxiliaryKind, AuxiliaryMessage, ChannelDrop, ChannelEvent, ChannelMessage, NetworkError,
    ReceiveUsage, datagram,
};

/// Borrowed receive half of one selected socket and its ordinary reassembly state.
pub struct ClientReceiver<'channel, 'storage> {
    pub(crate) socket: &'channel UdpSocket,
    pub(crate) incoming: &'channel mut [u8],
    pub(crate) state: &'channel mut ReceiveChannel<'storage>,
    pub(crate) qport: u16,
    pub(crate) usage: &'channel mut ReceiveUsage,
}

impl ClientReceiver<'_, '_> {
    /// Reads and inspects at most one datagram before an unchanged absolute deadline.
    /// Returns `None` on expiry without invoking the callback. Borrowed data cannot
    /// escape through its result; copy explicitly only if the next layer needs ownership.
    ///
    /// # Errors
    /// Reports OS I/O failures. Malformed packets become bounded drop events.
    ///
    /// Borrowed packet bytes cannot escape the scoped callback:
    /// ```compile_fail
    /// use cod4xr_client_net::{ChannelEvent, ClientReceiver, NetworkError};
    /// use std::time::Instant;
    /// fn escape<'a>(receiver: &'a mut ClientReceiver<'_, '_>)
    ///     -> Result<Option<&'a [u8]>, NetworkError> {
    ///     receiver.receive_until(Instant::now(), |event| match event {
    ///         ChannelEvent::Message(message) => message.body(),
    ///         _ => &[],
    ///     })
    /// }
    /// ```
    pub fn receive_until<T>(
        &mut self,
        wake_at: Instant,
        inspect: impl FnOnce(ChannelEvent<'_>) -> T,
    ) -> Result<Option<T>, NetworkError> {
        let Some(length) = datagram::receive_until(self.socket, self.incoming, wake_at)? else {
            return Ok(None);
        };
        self.usage.datagrams = self.usage.datagrams.saturating_add(1);
        self.usage.bytes = self
            .usage
            .bytes
            .saturating_add(u64::try_from(length).unwrap_or(u64::MAX));
        let packet = Packet::parse(&self.incoming[..length], Direction::ServerToClient);
        let event = match packet {
            Ok(Packet::Channel(ref packet)) => match self.state.accept(packet) {
                ReceiveOutcome::Message(inner) => ChannelEvent::Message(ChannelMessage { inner }),
                ReceiveOutcome::Pending => ChannelEvent::FragmentPending,
                ReceiveOutcome::Dropped(reason) => {
                    ChannelEvent::Dropped(ChannelDrop::State(reason))
                }
                _ => ChannelEvent::Dropped(ChannelDrop::Unsupported),
            },
            Ok(Packet::Connectionless(body)) => ChannelEvent::Auxiliary(AuxiliaryMessage {
                kind: AuxiliaryKind::Connectionless,
                body,
            }),
            Ok(Packet::UpdateProxy(body)) => ChannelEvent::Auxiliary(AuxiliaryMessage {
                kind: AuxiliaryKind::UpdateProxy,
                body,
            }),
            Ok(Packet::Reliable { qport, body }) if qport == self.qport => {
                ChannelEvent::Auxiliary(AuxiliaryMessage {
                    kind: AuxiliaryKind::Reliable,
                    body,
                })
            }
            Ok(Packet::Reliable { .. }) => ChannelEvent::Dropped(ChannelDrop::WrongQport),
            Ok(_) => ChannelEvent::Dropped(ChannelDrop::Unsupported),
            Err(_) => ChannelEvent::Dropped(ChannelDrop::Malformed),
        };
        match event {
            ChannelEvent::Message(_) => self.usage.messages = self.usage.messages.saturating_add(1),
            ChannelEvent::FragmentPending => {
                self.usage.pending_fragments = self.usage.pending_fragments.saturating_add(1);
            }
            ChannelEvent::Dropped(_) => self.usage.dropped = self.usage.dropped.saturating_add(1),
            ChannelEvent::Auxiliary(_) => {
                self.usage.auxiliary = self.usage.auxiliary.saturating_add(1);
            }
        }
        Ok(Some(inspect(event)))
    }

    /// Returns cumulative bounded-category counts, without remote bytes.
    #[must_use]
    pub fn usage(&self) -> ReceiveUsage {
        *self.usage
    }
    /// Returns the last delivered ordinary sequence, initially zero.
    #[must_use]
    pub fn incoming_sequence(&self) -> u32 {
        self.state.incoming_sequence()
    }
    /// Returns retained partial-fragment bytes.
    #[must_use]
    pub fn partial_bytes(&self) -> usize {
        self.state.partial_bytes()
    }
    /// Discards incomplete reassembly when the host's independent fragment timeout expires.
    /// This leaves the last delivered sequence unchanged.
    pub fn discard_partial(&mut self) {
        self.state.discard_partial();
    }
}

impl fmt::Debug for ClientReceiver<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientReceiver")
            .field("usage", &self.usage)
            .field("partial_bytes", &self.partial_bytes())
            .finish_non_exhaustive()
    }
}
