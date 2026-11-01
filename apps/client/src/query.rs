// SPDX-License-Identifier: AGPL-3.0-or-later
use std::time::{Duration, Instant};

use cod4xr_client_core::{
    INFO_REQUEST_BYTES, InfoResponse, MAX_INFO_DATAGRAM_BYTES, QueryChallenge,
};

use crate::{arguments::RuntimeOptions, error::ClientError, net_io};

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
    options: &RuntimeOptions,
    challenge: QueryChallenge,
) -> Result<QueryReply, ClientError> {
    let socket = net_io::connect(options.peer)?;
    let started = Instant::now();
    let deadline = started
        .checked_add(options.timeout)
        .ok_or(ClientError::Timeout)?;
    let mut request = [0; INFO_REQUEST_BYTES];
    let length = challenge
        .write_request(&mut request)
        .map_err(|_| ClientError::InvalidResponse)?;
    net_io::send(&socket, &request[..length], deadline)?;
    // One full-size UDP buffer avoids platform-specific truncation acceptance.
    // Accepted normal-info replies are separately bounded to ~1 KiB by the core.
    let mut incoming = net_io::receive_buffer()?;
    let mut rejected = 0;
    for received in 1..=options.max_datagrams {
        let length =
            net_io::receive_until(&socket, &mut incoming, deadline)?.ok_or(ClientError::Timeout)?;
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
