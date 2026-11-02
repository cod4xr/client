// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{SnapshotInput, SnapshotOutput};
use super::{SnapshotPayloadError, SnapshotServicePlan};
use crate::{
    message_body::BodyReader, player_state::PredictionSample, server_payload::DecodedServerPayload,
    service_controls::ServiceControlBudget,
};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use std::fmt;

/// Explicit one-shot prediction policy; no sample lookup or silent fallback occurs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPrediction<'sample> {
    /// Require native transmitted-origin mode.
    Transmitted,
    /// Require unsent-origin mode and an exact caller-owned profile/time sample.
    Sample(&'sample PredictionSample),
    /// Require unsent-origin mode and explicitly select native prior fallback.
    NativePriorFallback,
}
/// Complete cooked snapshot carrier bound to an actual decoded ordinary payload.
/// This contains checked ACK metadata, not authority to mutate another owner.
pub struct SnapshotPayloadPlan<'storage> {
    services: SnapshotServicePlan<'storage>,
    client_acknowledge: u32,
}
impl<'storage> SnapshotPayloadPlan<'storage> {
    /// Binds context to the original payload profile/sequence before any output write,
    /// then validates the complete specialized carrier and explicit prediction choice.
    /// The result retains neither payload/key-history nor sample/source borrows.
    /// Use the reader-based service preparation when the host needs to inspect an
    /// exact prediction request before choosing a sample. No ACK/history mutation,
    /// peer execution, socket access or profile expansion occurs here.
    ///
    /// # Errors
    /// Rejects profile/sequence mismatch, body bounds, malformed/unsupported services,
    /// inspection/retention budgets and prediction failure. Both complete output
    /// buffers and the outgoing command history remain unchanged on failure.
    pub fn parse(
        payload: &DecodedServerPayload<'_, '_>,
        input: SnapshotInput<'_>,
        budget: ServiceControlBudget,
        output: SnapshotOutput<'storage>,
        prediction: SnapshotPrediction<'_>,
    ) -> Result<Self, SnapshotPayloadError> {
        if input.context.profile() != payload.profile() {
            return Err(SnapshotPayloadError::Profile);
        }
        if input.message.sequence() != payload.sequence() {
            return Err(SnapshotPayloadError::MessageSequence);
        }
        let mut reader = BodyReader::try_new(payload.body(), MAX_MESSAGE_BYTES)
            .map_err(SnapshotPayloadError::Body)?;
        let prepared = SnapshotServicePlan::prepare(&mut reader, input, budget, output)
            .map_err(SnapshotPayloadError::Services)?;
        let services = match prediction {
            SnapshotPrediction::Transmitted => prepared.commit_transmitted(),
            SnapshotPrediction::Sample(sample) => prepared.commit_with_prediction(sample),
            SnapshotPrediction::NativePriorFallback => prepared.commit_native_prior_fallback(),
        }
        .map_err(SnapshotPayloadError::Services)?;
        Ok(Self {
            services,
            client_acknowledge: payload.client_command_acknowledge(),
        })
    }
    /// Borrows complete cooked services and their coherent snapshot reference.
    #[must_use]
    pub fn services(&self) -> &SnapshotServicePlan<'storage> {
        &self.services
    }
    /// Returns the actual decoded ordinary-message sequence validated by this plan.
    #[must_use]
    pub fn message_sequence(&self) -> u32 {
        self.services.snapshot().message().sequence()
    }
    /// Returns the original payload's checked client-command ACK without applying it.
    /// Preflight all host resources and commit only to the same connection's owner.
    #[must_use]
    pub fn client_command_acknowledge(&self) -> u32 {
        self.client_acknowledge
    }
}
impl fmt::Debug for SnapshotPayloadPlan<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotPayloadPlan")
            .field("profile", &self.services.snapshot().header().profile())
            .field("message_sequence", &self.message_sequence())
            .field("client_acknowledge", &self.client_acknowledge)
            .finish_non_exhaustive()
    }
}
