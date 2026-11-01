// SPDX-License-Identifier: AGPL-3.0-or-later
/// Saturating lifetime send counters without payloads or routing identifiers.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SendUsage {
    /// Ordinary sequences reserved, including messages later cancelled or failed.
    pub messages_reserved: u64,
    /// Complete UDP datagrams accepted by the OS, not acknowledged by a peer.
    pub datagrams_sent: u64,
    /// Bytes accepted by the OS, including channel headers.
    pub bytes_sent: u64,
    /// Failed datagram I/O operations, including deadline expiry.
    pub failed_datagrams: u64,
    /// Auxiliary reliable datagrams accepted by the OS, included in total datagrams.
    pub reliable_datagrams_sent: u64,
    /// Failed auxiliary reliable sends, included in total failed datagrams.
    pub reliable_failed_datagrams: u64,
}

/// Saturating lifetime receive counters; each receive call inspects at most one packet.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ReceiveUsage {
    /// All datagrams read from the selected peer.
    pub datagrams: u64,
    /// Bytes read, including headers and rejected/auxiliary traffic.
    pub bytes: u64,
    /// Complete ordinary messages delivered to the callback.
    pub messages: u64,
    /// Accepted full fragments that retained partial state without delivering a message.
    pub pending_fragments: u64,
    /// Malformed, stale, out-of-order, wrong-qport or over-capacity packets.
    pub dropped: u64,
    /// Classified non-ordinary packets; their transports are not implemented here.
    pub auxiliary: u64,
}
