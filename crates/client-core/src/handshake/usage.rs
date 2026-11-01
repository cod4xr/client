// SPDX-License-Identifier: AGPL-3.0-or-later
use super::ChallengeRejection;

/// Cumulative bounded probe accounting; request contents never appear here.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ChallengeUsage {
    /// Successfully admitted request encodings; the runtime owns actual socket sends.
    pub attempts: u32,
    /// All inspected datagrams, including rejected packets.
    pub received: u32,
    /// Inspected packets that failed endpoint/profile/nonce/state validation.
    pub rejected: u32,
    /// Most recent fixed rejection category, without raw peer input.
    pub last_rejection: Option<ChallengeRejection>,
}
