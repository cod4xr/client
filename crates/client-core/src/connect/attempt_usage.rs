// SPDX-License-Identifier: AGPL-3.0-or-later
/// Fixed reason a datagram did not advance pending connect state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConnectRejection {
    /// No connect request has been admitted yet.
    Unsolicited,
    /// Complete source IP/port differs from the checked plan's peer.
    WrongPeer,
    /// The command or payload is malformed or exceeds its named profile.
    Malformed,
    /// The command is outside this selected connect-reply profile.
    Unsupported,
}

/// Cumulative phase work without private names, routing values or payloads.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ConnectUsage {
    /// Admitted complete request encodings; the host owns actual I/O.
    pub attempts: u32,
    /// All inspected datagrams, including wrong peers, notices and malformed input.
    pub received: u32,
    /// Inspected packets rejected by peer/phase/profile validation.
    pub rejected: u32,
    /// Valid informational print packets that did not admit or refuse a client.
    pub notices: u32,
    /// Latest fixed rejection category without raw received bytes.
    pub last_rejection: Option<ConnectRejection>,
}
