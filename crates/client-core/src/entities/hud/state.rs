// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::EntityDecodeContext;
use super::{HUD_SLOTS, HUD_WORDS, HudError, HudWord, decode};
use crate::{BaselineProtocol, message_body::BodyReader};
use std::fmt;

/// Owned native HUD words before typed presentation or resource validation.
#[derive(Clone, Eq, PartialEq)]
pub struct HudElement {
    pub(super) words: [u32; HUD_WORDS],
}
impl Default for HudElement {
    fn default() -> Self {
        Self {
            words: [0; HUD_WORDS],
        }
    }
}
impl HudElement {
    /// Returns one complete inert semantic word without resource or string lookup.
    #[must_use]
    pub fn word(&self, word: HudWord) -> u32 {
        self.words[word as usize]
    }
}
impl fmt::Debug for HudElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HudElement")
            .field("words", &self.words.len())
            .finish_non_exhaustive()
    }
}

/// Complete archival/current native HUD storage; free-slot words can remain inert.
/// These fixed wire arrays are separate from wider engine/UI storage and admission.
#[derive(Clone, Eq, PartialEq)]
pub struct SnapshotHud {
    context: EntityDecodeContext,
    archival: [HudElement; HUD_SLOTS],
    current: [HudElement; HUD_SLOTS],
}
impl SnapshotHud {
    /// Constructs an explicit zero reference with the caller's checked context.
    #[must_use]
    pub fn zero(context: EntityDecodeContext) -> Self {
        Self {
            context,
            archival: std::array::from_fn(|_| HudElement::default()),
            current: std::array::from_fn(|_| HudElement::default()),
        }
    }
    /// Decodes the entire outer changed bit and both arrays before cursor commit.
    /// Archival comes first. `None` supplies zero words; explicit legacy context may
    /// change between same-profile sections, without changing prior semantic words.
    /// This applies no rendering, CVar/resource policy or complete snapshot admission.
    ///
    /// # Errors
    /// Rejects profile mismatch, invalid last-field indices, bad compact/legacy
    /// references and truncation. Failure preserves both original cursors, cached bits
    /// and all prior words, including inactive tails and the other HUD array.
    pub fn parse(
        reader: &mut BodyReader<'_>,
        context: EntityDecodeContext,
        time: u32,
        reference: Option<&Self>,
    ) -> Result<Self, HudError> {
        if reference.is_some_and(|prior| prior.profile() != context.profile()) {
            return Err(HudError::ReferenceProfile);
        }
        let mut result = reference.cloned().unwrap_or_else(|| Self::zero(context));
        result.context = context;
        let mut candidate = reader.clone();
        if candidate.read_bits(1)? != 0 {
            decode::array(&mut candidate, context, time, &mut result.archival)?;
            decode::array(&mut candidate, context, time, &mut result.current)?;
        }
        *reader = candidate;
        Ok(result)
    }
    /// Returns the current explicit coordinate context, without inferring a dialect.
    #[must_use]
    pub fn context(&self) -> EntityDecodeContext {
        self.context
    }
    /// Returns the explicit baseline profile of the current context.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.context.profile()
    }
    /// Borrows every archival slot, including free/inert entries needed by deltas.
    #[must_use]
    pub fn archival(&self) -> &[HudElement; HUD_SLOTS] {
        &self.archival
    }
    /// Borrows every current slot; serialized prefixes do not imply active UI elements.
    #[must_use]
    pub fn current(&self) -> &[HudElement; HUD_SLOTS] {
        &self.current
    }
}
impl fmt::Debug for SnapshotHud {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SnapshotHud")
            .field("profile", &self.profile())
            .field("slots_per_array", &HUD_SLOTS)
            .finish_non_exhaustive()
    }
}
