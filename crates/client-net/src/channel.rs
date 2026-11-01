// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::Route;
use cod4x_transport::{ReceiveChannel, SendChannel};

use crate::{
    ChannelContext, ClientReceiver, ClientSender, NetworkError, ReceiveUsage, SendUsage,
    peer::PeerIo,
};

/// An acknowledged socket with shared ordinary-channel sequencing/reassembly state.
/// Bodies remain undecoded. This is not a gameplay or auxiliary reliable session.
pub struct ClientChannel<'storage> {
    io: PeerIo,
    context: ChannelContext,
    send: SendChannel,
    receive: ReceiveChannel<'storage>,
    send_usage: SendUsage,
    receive_usage: ReceiveUsage,
}

/// Disjoint borrowed channel halves permit receive work between outbound fragments.
pub struct ChannelParts<'channel, 'storage> {
    /// Sender for complete already encoded ordinary-message bodies.
    pub sender: ClientSender<'channel>,
    /// Receiver for scoped inspection of undecoded messages and auxiliary envelopes.
    pub receiver: ClientReceiver<'channel, 'storage>,
}

impl<'storage> ClientChannel<'storage> {
    pub(crate) fn new(
        io: PeerIo,
        context: ChannelContext,
        storage: &'storage mut [u8],
    ) -> Result<Self, NetworkError> {
        let receive =
            ReceiveChannel::try_new(storage, Route::ToClient).map_err(NetworkError::Channel)?;
        let send = SendChannel::new(Route::ToServer {
            qport: context.summary().qport(),
        });
        Ok(Self {
            io,
            context,
            send,
            receive,
            send_usage: SendUsage::default(),
            receive_usage: ReceiveUsage::default(),
        })
    }

    /// Borrows original protocol, routing and private obfuscation context.
    #[must_use]
    pub fn context(&self) -> &ChannelContext {
        &self.context
    }
    /// Returns cumulative outgoing work without payloads/routing identifiers.
    #[must_use]
    pub fn send_usage(&self) -> SendUsage {
        self.send_usage
    }
    /// Returns cumulative incoming work without payloads/routing identifiers.
    #[must_use]
    pub fn receive_usage(&self) -> ReceiveUsage {
        self.receive_usage
    }

    /// Borrows disjoint send/receive state on the original selected socket.
    /// No allocation, socket duplication, new nonce or peer change occurs.
    pub fn split(&mut self) -> ChannelParts<'_, 'storage> {
        ChannelParts {
            sender: ClientSender {
                socket: &self.io.socket,
                state: &mut self.send,
                usage: &mut self.send_usage,
            },
            receiver: ClientReceiver {
                socket: &self.io.socket,
                incoming: &mut self.io.incoming,
                state: &mut self.receive,
                qport: self.context.summary().qport(),
                usage: &mut self.receive_usage,
            },
        }
    }
}

impl fmt::Debug for ClientChannel<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientChannel")
            .field("send_usage", &self.send_usage)
            .field("receive_usage", &self.receive_usage)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for ChannelParts<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ChannelParts")
            .field("sender", &self.sender)
            .field("receiver", &self.receiver)
            .finish_non_exhaustive()
    }
}
