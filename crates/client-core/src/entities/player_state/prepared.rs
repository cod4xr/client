// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    LivePlayerState, PlayerStateError, PredictionProvenance, PredictionRequest, PredictionSample,
    PredictionWords, state::DecodedPlayerState,
};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Exclusive whole-player-state transaction. Drop cancels all reader movement.
/// No decoded state escapes before prediction selection and reader commitment.
/// The lease excludes reader access until it is consumed or dropped:
///
/// ```compile_fail,E0499
/// use cod4xr_client_core::{entities::EntityDecodeContext, message_body::BodyReader,
///     player_state::LivePlayerState};
/// let mut reader = BodyReader::try_new(&[1, 0, 0], 3).unwrap();
/// let prepared = LivePlayerState::prepare(&mut reader, EntityDecodeContext::Cod4x21, 0, None).unwrap();
/// reader.read_bits(1).unwrap();
/// prepared.commit_transmitted().unwrap();
/// ```
/// Commitment consumes the lease, preventing a second publication:
///
/// ```compile_fail,E0382
/// use cod4xr_client_core::{entities::EntityDecodeContext, message_body::BodyReader,
///     player_state::LivePlayerState};
/// let mut reader = BodyReader::try_new(&[1, 0, 0], 3).unwrap();
/// let prepared = LivePlayerState::prepare(&mut reader, EntityDecodeContext::Cod4x21, 0, None).unwrap();
/// prepared.commit_transmitted().unwrap();
/// prepared.commit_transmitted().unwrap();
/// ```
#[must_use = "Dropping the prepared player state cancels the complete read."]
pub struct PreparedPlayerState<'reader, 'body> {
    pub(super) reader: &'reader mut BodyReader<'body>,
    pub(super) candidate: BodyReader<'body>,
    pub(super) decoded: DecodedPlayerState,
    pub(super) request: Option<PredictionRequest>,
    pub(super) fallback: PredictionWords,
    pub(super) consumed_bytes: usize,
}
impl PreparedPlayerState<'_, '_> {
    /// Returns the exact lookup request, if the wire selected unsent origin mode.
    /// Profile/time matching does not prove a connection/map epoch; the caller must
    /// scope its prediction queue to the current session.
    #[must_use]
    pub fn prediction_request(&self) -> Option<PredictionRequest> {
        self.request
    }
    /// Returns the explicitly selected baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.decoded.context.profile()
    }
    /// Returns newly consumed physical bytes; cached bits may also be consumed.
    #[must_use]
    pub fn consumed_bytes(&self) -> usize {
        self.consumed_bytes
    }
    /// Commits a state whose wire mode supplied origin/velocity words.
    ///
    /// # Errors
    /// An outstanding prediction request cancels the entire read.
    pub fn commit_transmitted(self) -> Result<LivePlayerState, PlayerStateError> {
        if self.request.is_some() {
            return Err(PlayerStateError::PredictionRequired);
        }
        Ok(self.finish(PredictionProvenance::Transmitted))
    }
    /// Replaces exactly eleven native words from a matching caller-owned sample.
    ///
    /// # Errors
    /// Missing request, profile mismatch or command-time mismatch cancels the entire
    /// read. The sample remains inert and is never retained by the result.
    pub fn commit_with_prediction(
        mut self,
        sample: &PredictionSample,
    ) -> Result<LivePlayerState, PlayerStateError> {
        let request = self.request.ok_or(PlayerStateError::UnexpectedPrediction)?;
        if sample.profile() != request.profile() {
            return Err(PlayerStateError::PredictionProfile);
        }
        if sample.command_time() != request.command_time() {
            return Err(PlayerStateError::PredictionTime);
        }
        sample.words.apply(&mut self.decoded.scalars);
        Ok(self.finish(PredictionProvenance::CommandTimeMatched))
    }
    /// Explicitly applies native prior fallback to the eleven prediction words.
    /// Without a prior, those words become zero. New command time, other scalar
    /// words and all decoded arrays remain intact, even when the wire carried
    /// different origin/view-angle values before fallback.
    ///
    /// # Errors
    /// Transmitted-origin mode rejects fallback and cancels the entire read.
    pub fn commit_native_prior_fallback(mut self) -> Result<LivePlayerState, PlayerStateError> {
        if self.request.is_none() {
            return Err(PlayerStateError::UnexpectedPrediction);
        }
        self.fallback.apply(&mut self.decoded.scalars);
        Ok(self.finish(PredictionProvenance::ExplicitPriorFallback))
    }
    fn finish(self, prediction: PredictionProvenance) -> LivePlayerState {
        *self.reader = self.candidate;
        LivePlayerState {
            decoded: self.decoded,
            prediction,
        }
    }
}
impl fmt::Debug for PreparedPlayerState<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedPlayerState")
            .field("profile", &self.profile())
            .field("consumed_bytes", &self.consumed_bytes)
            .field("needs_prediction", &self.request.is_some())
            .finish_non_exhaustive()
    }
}
