// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{
    borrow::Cow,
    io::{self, Write},
    net::SocketAddr,
};

use cod4xr_client_core::InfoResponse;

use crate::{error::ClientError, query::QueryReply};

pub(crate) fn write(
    peer: SocketAddr,
    reply: &QueryReply,
    info: &InfoResponse<'_>,
) -> Result<(), ClientError> {
    let text = |key| {
        info.field(key)
            .map_or(Cow::Borrowed(""), String::from_utf8_lossy)
    };
    let value = serde_json::json!({
        "project": "CoD4XR",
        "phase": "discovery",
        "peer": peer.to_string(),
        "round_trip_micros": reply.elapsed.as_micros(),
        "advertised_protocol": info.advertised_protocol(),
        "hostname": text(b"hostname"),
        "map": text(b"mapname"),
        "game_type": text(b"gametype"),
        "short_version": text(b"shortversion"),
        "clients": text(b"clients"),
        "max_clients": text(b"sv_maxclients"),
        "received_datagrams": reply.received,
        "rejected_datagrams": reply.rejected,
        "gameplay_compatibility": "unverified"
    });
    let mut stdout = io::stdout().lock();
    serde_json::to_writer(&mut stdout, &value).map_err(ClientError::Output)?;
    writeln!(stdout)?;
    Ok(())
}
