// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    BaselineProtocol,
    message_body::{BodyReadError, BodyReader},
};
use std::{fmt, num::NonZeroU8};

/// Raw native reference selector, before message-sequence/history validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotDelta {
    /// A zero distance selects a wire full snapshot without a prior snapshot.
    Full,
    /// Nonzero distance is relative to the containing server message sequence.
    /// It is not relative to the last received snapshot or an arrival counter.
    Previous(NonZeroU8),
}
impl SnapshotDelta {
    /// Returns the exact wire byte, including zero for a full snapshot.
    #[must_use]
    pub fn distance(self) -> u8 {
        match self {
            Self::Full => 0,
            Self::Previous(distance) => distance.get(),
        }
    }
}
/// Owned six-byte native header with opaque time/flag values.
/// Reading this constituent neither resolves references nor admits a snapshot.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SnapshotHeader {
    profile: BaselineProtocol,
    server_time: u32,
    delta: SnapshotDelta,
    flags: u8,
}
impl SnapshotHeader {
    /// Parses immediately after the caller has selected snapshot service opcode six.
    /// The native fields are little-endian time, one delta-distance byte and one
    /// flag byte. There is no area-mask length or area-mask payload here. Byte
    /// operations preserve any cached bits for the following player-state reader.
    /// No message sequence, reference ring or global server-command state is read.
    ///
    /// # Errors
    /// Truncation anywhere preserves the original byte cursor and cached bits.
    /// Success commits only this header, not following player/entity/client state.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
    ) -> Result<Self, BodyReadError> {
        let mut candidate = reader.clone();
        let server_time = candidate.read_i32()?.cast_unsigned();
        let delta = NonZeroU8::new(candidate.read_u8()?)
            .map_or(SnapshotDelta::Full, SnapshotDelta::Previous);
        let flags = candidate.read_u8()?;
        *reader = candidate;
        Ok(Self {
            profile,
            server_time,
            delta,
            flags,
        })
    }
    /// Returns the caller-selected profile, without inferring a gameplay dialect.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
    /// Returns all 32 wrapping server-time bits without signed/clock interpretation.
    #[must_use]
    pub fn server_time(self) -> u32 {
        self.server_time
    }
    /// Returns the exact full/delta selector before history admission.
    #[must_use]
    pub fn delta(self) -> SnapshotDelta {
        self.delta
    }
    /// Returns all eight raw flags, preserving bits without inventing engine effects.
    #[must_use]
    pub fn flags(self) -> u8 {
        self.flags
    }
}
impl fmt::Debug for SnapshotHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotHeader")
            .field("profile", &self.profile)
            .field("delta", &self.delta)
            .finish_non_exhaustive()
    }
}
