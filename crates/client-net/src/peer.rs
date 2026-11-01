// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

use cod4xr_client_core::connect::MAX_CONNECT_REPLY_BYTES;

use crate::{NetworkError, datagram};

pub(crate) struct PeerIo {
    pub(crate) socket: UdpSocket,
    pub(crate) peer: SocketAddr,
    pub(crate) started: Instant,
    pub(crate) incoming: Vec<u8>,
    pub(crate) terminal: Vec<u8>,
}

impl PeerIo {
    pub(crate) fn new(peer: SocketAddr) -> Result<Self, NetworkError> {
        let started = Instant::now();
        let socket = datagram::connect(peer)?;
        let incoming = datagram::receive_buffer()?;
        let mut terminal = Vec::new();
        // Reserve terminal storage before the first request can admit us remotely.
        terminal
            .try_reserve_exact(MAX_CONNECT_REPLY_BYTES)
            .map_err(|_| NetworkError::Allocation)?;
        Ok(Self {
            socket,
            peer,
            started,
            incoming,
            terminal,
        })
    }

    pub(crate) fn at(&self, offset: Duration) -> Result<Instant, NetworkError> {
        self.started
            .checked_add(offset)
            .ok_or(NetworkError::ClockOverflow)
    }
}
