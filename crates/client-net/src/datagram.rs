// SPDX-License-Identifier: AGPL-3.0-or-later
//! Blocking UDP primitives using complete receive buffers and absolute deadlines.

use std::{
    io,
    net::{SocketAddr, UdpSocket},
    time::Instant,
};

use crate::NetworkError;

/// Binds an ephemeral socket of the peer's address family and selects that peer.
/// OS filtering supplements the deterministic phase checks; no DNS is performed.
///
/// # Errors
/// Rejects unspecified/multicast/zero-port peers and propagates socket errors.
pub fn connect(peer: SocketAddr) -> Result<UdpSocket, NetworkError> {
    if peer.port() == 0 || peer.ip().is_unspecified() || peer.ip().is_multicast() {
        return Err(NetworkError::InvalidPeer);
    }
    let bind = if peer.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)?;
    socket.connect(peer)?;
    Ok(socket)
}

/// Allocates one complete UDP receive buffer, independently of remote lengths.
///
/// # Errors
/// Reports allocation failure before returning any partial buffer.
pub fn receive_buffer() -> Result<Vec<u8>, NetworkError> {
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(65_535)
        .map_err(|_| NetworkError::Allocation)?;
    buffer.resize(65_535, 0);
    Ok(buffer)
}

/// Sends exactly one datagram before the unchanged absolute deadline.
///
/// # Errors
/// Reports expiry, socket failure or an incomplete datagram send.
pub fn send(socket: &UdpSocket, bytes: &[u8], deadline: Instant) -> Result<(), NetworkError> {
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or(NetworkError::Timeout)?;
        socket.set_write_timeout(Some(remaining))?;
        match socket.send(bytes) {
            Ok(length) if length == bytes.len() => return Ok(()),
            Ok(_) => {
                return Err(
                    io::Error::new(io::ErrorKind::WriteZero, "incomplete UDP request").into(),
                );
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(error.into()),
        }
    }
}

/// Receives one packet, or returns `None` after the absolute wake point.
/// Interruptions and early OS timeouts do not restart the wait duration.
///
/// # Errors
/// Reports socket configuration and non-timeout receive failures.
pub fn receive_until(
    socket: &UdpSocket,
    output: &mut [u8],
    wake_at: Instant,
) -> Result<Option<usize>, NetworkError> {
    loop {
        let Some(remaining) = wake_at
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
        else {
            return Ok(None);
        };
        socket.set_read_timeout(Some(remaining))?;
        match socket.recv(output) {
            Ok(length) => return Ok(Some(length)),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted
                        | io::ErrorKind::WouldBlock
                        | io::ErrorKind::TimedOut
                ) => {}
            Err(error) => return Err(error.into()),
        }
    }
}
