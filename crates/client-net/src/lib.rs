// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned, bounded blocking client network phases. Run these on an I/O worker.
//! Acknowledgement is pre-channel state; no playable connection is claimed.

mod challenge;
mod channel;
mod channel_context;
mod channel_event;
mod channel_receive;
mod channel_reliable_send;
mod channel_send;
mod channel_usage;
mod connect;
pub mod datagram;
mod error;
mod outcome;
mod peer;
mod reliable_connection;

pub use challenge::{ChallengeRun, ChallengedSocket, challenge};
pub use channel::{ChannelParts, ClientChannel};
pub use channel_context::ChannelContext;
pub use channel_event::{
    AuxiliaryKind, AuxiliaryMessage, ChannelDrop, ChannelEvent, ChannelMessage,
};
pub use channel_receive::ClientReceiver;
pub use channel_reliable_send::SentReliableDatagram;
pub use channel_send::{ClientSender, OutboundMessage, SentDatagram};
pub use channel_usage::{ReceiveUsage, SendUsage};
pub use connect::{ConnectRun, PendingConnect};
pub use error::NetworkError;
pub use outcome::{AcknowledgedSocket, ConnectOutcome, ConnectSummary};
pub use reliable_connection::{
    ConnectionReceive, ConnectionSend, GameStateFrameError, OrdinaryConnectionSender,
    PreparedGameStateFrame, PreparedIdentityGameStateFrame, ReliableConnection,
    ReliableConnectionStats, ReliableRejection, ReliableUsage,
};
