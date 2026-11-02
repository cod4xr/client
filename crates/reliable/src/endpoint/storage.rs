// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use crate::{
    receive::ReceiveSlot,
    schedule::ScheduleConfig,
    send::{NATIVE_SEND_WINDOW_FRAGMENTS, SendSlot},
};

/// Explicit endpoint timing and outgoing in-flight policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EndpointConfig {
    /// Funded outgoing flight span, independent of total queue capacity.
    pub send_window: u16,
    /// Host timing policy; no peer metadata can change it.
    pub schedule: ScheduleConfig,
}

impl Default for EndpointConfig {
    fn default() -> Self {
        Self {
            send_window: NATIVE_SEND_WINDOW_FRAGMENTS,
            schedule: ScheduleConfig::default(),
        }
    }
}

/// Exclusive buffers for a fresh endpoint. All geometry is preflighted before tag resets.
/// No remote length controls allocation. Dirty bytes are not exposed by construction.
pub struct EndpointStorage<'buffers> {
    /// Exact outgoing fragment-slot bytes.
    pub send_bytes: &'buffers mut [u8],
    /// Opaque outgoing fragment metadata.
    pub send_slots: &'buffers mut [SendSlot],
    /// Exact incoming fragment-slot bytes.
    pub receive_bytes: &'buffers mut [u8],
    /// Opaque incoming fragment metadata.
    pub receive_slots: &'buffers mut [ReceiveSlot],
    /// Complete application body budget, excluding the four-byte length prefix.
    pub body: &'buffers mut [u8],
}

impl fmt::Debug for EndpointStorage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EndpointStorage")
            .field("send_bytes", &self.send_bytes.len())
            .field("send_slots", &self.send_slots.len())
            .field("receive_bytes", &self.receive_bytes.len())
            .field("receive_slots", &self.receive_slots.len())
            .field("body_bytes", &self.body.len())
            .finish_non_exhaustive()
    }
}
