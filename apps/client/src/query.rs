// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    io,
    net::UdpSocket,
    time::{Duration, Instant},
};

use cod4xr_client_core::{
    INFO_REQUEST_BYTES, InfoResponse, MAX_INFO_DATAGRAM_BYTES, QueryChallenge,
};

use crate::{arguments::QueryOptions, error::ClientError};

pub(crate) struct QueryReply {
    bytes: [u8; MAX_INFO_DATAGRAM_BYTES],
    length: usize,
    pub(crate) elapsed: Duration,
    pub(crate) received: u32,
    pub(crate) rejected: u32,
}

impl QueryReply {
    pub(crate) fn info(&self, challenge: QueryChallenge) -> Result<InfoResponse<'_>, ClientError> {
        InfoResponse::parse(&self.bytes[..self.length], challenge)
            .map_err(|_| ClientError::InvalidResponse)
    }
}

pub(crate) fn run(
    options: &QueryOptions,
    challenge: QueryChallenge,
) -> Result<QueryReply, ClientError> {
    let bind = if options.peer.is_ipv4() {
        "0.0.0.0:0"
    } else {
        "[::]:0"
    };
    let socket = UdpSocket::bind(bind)?;
    // A connected UDP socket filters replies from other endpoints in the OS.
    socket.connect(options.peer)?;
    socket.set_write_timeout(Some(options.timeout))?;
    let started = Instant::now();
    let deadline = started
        .checked_add(options.timeout)
        .ok_or(ClientError::Timeout)?;
    let mut request = [0; INFO_REQUEST_BYTES];
    let length = challenge
        .write_request(&mut request)
        .map_err(|_| ClientError::InvalidResponse)?;
    if socket.send(&request[..length])? != length {
        return Err(io::Error::new(io::ErrorKind::WriteZero, "incomplete UDP query").into());
    }
    // One full-size UDP buffer avoids platform-specific truncation acceptance.
    // Accepted normal-info replies are separately bounded to ~1 KiB by the core.
    let mut incoming = Vec::new();
    incoming
        .try_reserve_exact(65_535)
        .map_err(|_| ClientError::Allocation)?;
    incoming.resize(65_535, 0);
    let mut rejected = 0;
    for received in 1..=options.max_datagrams {
        let length = receive(&socket, &mut incoming, deadline)?;
        if InfoResponse::parse(&incoming[..length], challenge).is_err() {
            rejected += 1;
            continue;
        }
        let mut reply = QueryReply {
            bytes: [0; MAX_INFO_DATAGRAM_BYTES],
            length,
            elapsed: started.elapsed(),
            received,
            rejected,
        };
        reply.bytes[..length].copy_from_slice(&incoming[..length]);
        return Ok(reply);
    }
    Err(ClientError::ReceiveBudget)
}

fn receive(socket: &UdpSocket, output: &mut [u8], deadline: Instant) -> Result<usize, ClientError> {
    loop {
        let remaining = deadline
            .checked_duration_since(Instant::now())
            .filter(|duration| !duration.is_zero())
            .ok_or(ClientError::Timeout)?;
        socket.set_read_timeout(Some(remaining))?;
        match socket.recv(output) {
            Ok(length) => return Ok(length),
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
