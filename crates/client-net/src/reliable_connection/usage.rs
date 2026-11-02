// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{ReceiveUsage, SendUsage};
use cod4xr_reliable::endpoint::EndpointState;
use std::time::Duration;

/// Saturating counters above shared envelope/channel I/O, without remote bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReliableUsage {
    /// Whole reliable packets admitted by the endpoint.
    pub accepted_datagrams: u64,
    /// Elapsed host receipt time of the last fully admitted reliable packet.
    pub last_accepted_at: Option<Duration>,
    /// Reliable body or receive/ACK failures rejected without partial admission.
    pub rejected_datagrams: u64,
    /// Stream bytes drained, including work before terminal framing failure.
    pub drained_bytes: u64,
    /// Fragment inspections charged during bounded drains.
    pub fragment_inspections: u64,
    /// Complete opaque frames explicitly released by the host.
    pub released_frames: u64,
    /// Transitions into terminal framing failure, counted once per connection.
    pub framing_failures: u64,
    /// Typed gamestate decode attempts on a complete ready frame, including retries.
    pub gamestate_decode_attempts: u64,
    /// Attempts rejected by body, host budget or caller capacity validation.
    pub gamestate_decode_failures: u64,
    /// Prepared gamestate frames explicitly committed and released by the host.
    pub committed_gamestates: u64,
}

/// Copyable connection evidence for logging and host resource decisions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReliableConnectionStats {
    /// Ordinary/auxiliary send I/O totals and reliable subset counters.
    pub send: SendUsage,
    /// Shared receive I/O/classification totals.
    pub receive: ReceiveUsage,
    /// Endpoint admission/framing counters.
    pub reliable: ReliableUsage,
    /// Current bounded endpoint resource and sequence state.
    pub endpoint: EndpointState,
}
