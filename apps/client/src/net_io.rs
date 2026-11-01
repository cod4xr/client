// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    io,
    net::{SocketAddr, UdpSocket},
    time::Instant,
};

use crate::error::ClientError;

pub(crate) fn connect(peer: SocketAddr) -> Result<UdpSocket, ClientError> {
    let bind = if peer.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)?;
    // Filtering is done by the OS and independently by state that accepts packets.
    socket.connect(peer)?;
    Ok(socket)
}

pub(crate) fn receive_buffer() -> Result<Vec<u8>, ClientError> {
    let mut buffer = Vec::new();
    buffer
        .try_reserve_exact(65_535)
        .map_err(|_| ClientError::Allocation)?;
    buffer.resize(65_535, 0);
    Ok(buffer)
}

pub(crate) fn send(socket: &UdpSocket, bytes: &[u8], deadline: Instant) -> Result<(), ClientError> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .filter(|duration| !duration.is_zero())
        .ok_or(ClientError::Timeout)?;
    socket.set_write_timeout(Some(remaining))?;
    if socket.send(bytes)? != bytes.len() {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "incomplete UDP request").into());
    }
    Ok(())
}

pub(crate) fn receive_until(
    socket: &UdpSocket,
    output: &mut [u8],
    wake_at: Instant,
) -> Result<Option<usize>, ClientError> {
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
