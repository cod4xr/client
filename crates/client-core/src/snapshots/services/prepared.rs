// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{PreparedSnapshot, SnapshotHeader};
use super::{SnapshotServiceError, SnapshotServicePlan};
use crate::player_state::{PredictionRequest, PredictionSample};
use std::fmt;

/// Exclusive snapshot-only carrier lease, with EOF already checked before publication.
/// Drop, framing errors and prediction failures preserve reader and both outputs.
/// The enclosing carrier keeps the original reader exclusively borrowed:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{message_body::BodyReader, service_controls::ServiceControlBudget,
///     snapshots::{SnapshotInput, SnapshotOutput, services::{SnapshotServicePlan, SnapshotServiceError}}};
/// fn carrier_lease(reader: &mut BodyReader<'_>, input: SnapshotInput<'_>, output: SnapshotOutput<'_>)
///     -> Result<(), SnapshotServiceError> {
///     let prepared = SnapshotServicePlan::prepare(reader, input, ServiceControlBudget::default(), output)?;
///     reader.read_u8().unwrap();
///     prepared.commit_transmitted()?;
///     Ok(())
/// }
/// ```
#[must_use = "Dropping prepared snapshot services cancels the complete carrier."]
pub struct PreparedSnapshotServices<'reader, 'body, 'source, 'storage> {
    pub(super) snapshot: PreparedSnapshot<'reader, 'body, 'source, 'storage>,
    pub(super) record_count: u32,
    pub(super) consumed_bytes: usize,
}
impl<'storage> PreparedSnapshotServices<'_, '_, '_, 'storage> {
    /// Returns inert header metadata without exposing decoded candidate state.
    #[must_use]
    pub fn header(&self) -> SnapshotHeader {
        self.snapshot.header()
    }
    /// Returns an exact prediction request after the entire carrier passed EOF checks.
    #[must_use]
    pub fn prediction_request(&self) -> Option<PredictionRequest> {
        self.snapshot.prediction_request()
    }
    /// Returns bounded inspected-service evidence, including EOF.
    #[must_use]
    pub fn record_count(&self) -> u32 {
        self.record_count
    }
    /// Returns candidate physical bytes through logical EOF, before reader commit.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
    /// Commits transmitted mode only after complete service/snapshot validation.
    ///
    /// # Errors
    /// Outstanding prediction cancels the carrier before any output write.
    pub fn commit_transmitted(self) -> Result<SnapshotServicePlan<'storage>, SnapshotServiceError> {
        let snapshot = self
            .snapshot
            .commit_transmitted()
            .map_err(SnapshotServiceError::Snapshot)?;
        Ok(SnapshotServicePlan {
            snapshot,
            record_count: self.record_count,
            consumed_bytes: self.consumed_bytes,
        })
    }
    /// Commits the carrier using an exact caller-owned prediction sample.
    ///
    /// # Errors
    /// Missing request or profile/time mismatch cancels all publication.
    pub fn commit_with_prediction(
        self,
        sample: &PredictionSample,
    ) -> Result<SnapshotServicePlan<'storage>, SnapshotServiceError> {
        let snapshot = self
            .snapshot
            .commit_with_prediction(sample)
            .map_err(SnapshotServiceError::Snapshot)?;
        Ok(SnapshotServicePlan {
            snapshot,
            record_count: self.record_count,
            consumed_bytes: self.consumed_bytes,
        })
    }
    /// Commits the carrier using explicit native prior fallback.
    ///
    /// # Errors
    /// Transmitted-origin mode rejects fallback before all publication.
    pub fn commit_native_prior_fallback(
        self,
    ) -> Result<SnapshotServicePlan<'storage>, SnapshotServiceError> {
        let snapshot = self
            .snapshot
            .commit_native_prior_fallback()
            .map_err(SnapshotServiceError::Snapshot)?;
        Ok(SnapshotServicePlan {
            snapshot,
            record_count: self.record_count,
            consumed_bytes: self.consumed_bytes,
        })
    }
}
impl fmt::Debug for PreparedSnapshotServices<'_, '_, '_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedSnapshotServices")
            .field("profile", &self.header().profile())
            .field("record_count", &self.record_count)
            .field("consumed_bytes", &self.consumed_bytes)
            .field("needs_prediction", &self.prediction_request().is_some())
            .finish_non_exhaustive()
    }
}
