// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PlayerScalarError, PlayerScalars, decode};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Complete live scalar prefix before following arrays and prediction reconciliation.
/// The native origin/velocity mode is retained explicitly, without applying prediction.
pub struct PlayerScalarPrefix {
    scalars: PlayerScalars,
    origins_sent: bool,
    field_count: u8,
}
impl PlayerScalarPrefix {
    /// Reads one live snapshot scalar prefix into privately owned candidate words.
    /// `None` supplies zero state. Omitted fields retain full prior values; live
    /// hint-3 fields inside a transmitted prefix use zero references when selected.
    /// Archived player-state carriers are not selected or claimed by this method.
    ///
    /// # Errors
    /// Rejects reference-profile mismatch, invalid counts, malformed compact floats
    /// and truncation. Every failure preserves the immutable prior and both original
    /// reader cursors, including cached bits. Success admits only this prefix, not
    /// arrays, prediction samples, a complete player state, snapshot or message ACK.
    pub fn parse_live(
        reader: &mut BodyReader<'_>,
        profile: BaselineProtocol,
        time: u32,
        reference: Option<&PlayerScalars>,
    ) -> Result<Self, PlayerScalarError> {
        if reference.is_some_and(|prior| prior.profile() != profile) {
            return Err(PlayerScalarError::ReferenceProfile);
        }
        let mut candidate = reader.clone();
        let origins_sent = candidate.read_bits(1)? != 0;
        let field_count =
            u8::try_from(candidate.read_bits(8)?).map_err(|_| PlayerScalarError::FieldCount)?;
        let mut scalars = reference
            .cloned()
            .unwrap_or_else(|| PlayerScalars::zero(profile));
        decode::read(
            &mut candidate,
            time,
            origins_sent,
            usize::from(field_count),
            &mut scalars,
        )?;
        *reader = candidate;
        Ok(Self {
            scalars,
            origins_sent,
            field_count,
        })
    }
    /// Borrows the owned inert scalar words; prediction and containing data are absent.
    #[must_use]
    pub fn scalars(&self) -> &PlayerScalars {
        &self.scalars
    }
    /// Returns the native origin/velocity transmission mode, not an all-fields-present claim.
    #[must_use]
    pub fn origins_transmitted(&self) -> bool {
        self.origins_sent
    }
    /// Returns whether the native live reader requests command-time prediction lookup.
    /// A later whole-state owner must resolve that request or choose explicit fallback.
    #[must_use]
    pub fn needs_prediction(&self) -> bool {
        !self.origins_sent
    }
    /// Returns the exact field-prefix count, including zero; it is not a last index.
    #[must_use]
    pub fn changed_fields(&self) -> usize {
        usize::from(self.field_count)
    }
    /// Returns owned raw scalar words without admitting a reconciled player state.
    #[must_use]
    pub fn into_scalars(self) -> PlayerScalars {
        self.scalars
    }
}
impl fmt::Debug for PlayerScalarPrefix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlayerScalarPrefix")
            .field("profile", &self.scalars.profile())
            .field("field_count", &self.field_count)
            .field("origins_transmitted", &self.origins_sent)
            .finish_non_exhaustive()
    }
}
