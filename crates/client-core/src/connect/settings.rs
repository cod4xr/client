// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use cod4x_protocol::InfoString;

/// Explicit caller-selected connect settings, independent of challenge marker hints.
/// There is no authentication service or automatic credential/identity generation.
#[derive(Clone, Copy)]
pub struct NoAuthConnectSettings<'userinfo> {
    /// Positive gameplay wire protocol selected by a reviewed compatibility profile.
    /// This is not inferred from discovery protocol 6 or the xproto 18 marker.
    pub protocol: i32,
    /// Client-selected raw 16-bit NAT/channel routing value, encoded as a signed short.
    pub qport: u16,
    /// Explicit nonempty compatibility-version bytes for the legacy xver field.
    pub version: &'userinfo [u8],
    /// Validated caller userinfo. Unknown nonempty fields retain their byte values;
    /// empty values are omitted through the shared legacy setter's deletion rule.
    /// Names/passwords/identity claims are private; never log this view.
    /// Protocol, challenge, qport and xver keys must be supplied by typed fields only.
    pub userinfo: InfoString<'userinfo>,
}

impl fmt::Debug for NoAuthConnectSettings<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("NoAuthConnectSettings")
            .field("userinfo_bytes", &self.userinfo.as_bytes().len())
            .finish_non_exhaustive()
    }
}
