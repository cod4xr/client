// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{BASELINE_ENTITY_WORDS, EntityBaseline, EntityBaselineError, fields, tables};
use crate::{BaselineProtocol, message_body::BodyReader};
use cod4x_protocol::{LegacyIndexDecoder, LegacyIndexKind};
use std::fmt;

/// Ordered protocol-21 baseline index state for one gamestate.
/// Successful complete records alone advance both shared index and message cursors.
pub struct EntityBaselineDecoder {
    indices: LegacyIndexDecoder,
}

impl EntityBaselineDecoder {
    /// Starts a reviewed baseline stream, independent of discovery advertisements.
    ///
    /// # Errors
    /// Rejects protocol 6 until its coordinate encoding has separate conformance evidence.
    pub fn new(profile: BaselineProtocol) -> Result<Self, EntityBaselineError> {
        if profile != BaselineProtocol::Cod4x21 {
            return Err(EntityBaselineError::UnsupportedProfile);
        }
        Ok(Self {
            indices: LegacyIndexDecoder::new(LegacyIndexKind::Entity),
        })
    }
    /// Restarts index ordering for an explicitly begun new gamestate.
    pub fn reset(&mut self) {
        self.indices.reset();
    }
    /// Parses immediately after a gamestate `svc_baseline` opcode.
    /// Source bytes, field words and engine state are never logged or executed.
    ///
    /// # Errors
    /// Rejects index order/namespace failures, removal records, invalid field counts
    /// and truncated fields. Reader and index decoder remain unchanged on all failures.
    pub fn parse(
        &mut self,
        reader: &mut BodyReader<'_>,
    ) -> Result<EntityBaseline, EntityBaselineError> {
        let mut candidate = reader.clone();
        let mut indices = self.indices;
        let number = u32::from(candidate.read_index(&mut indices)?);
        if candidate.read_bits(1)? != 0 {
            return Err(EntityBaselineError::Removed);
        }
        let mut baseline = EntityBaseline {
            number,
            words: [0; BASELINE_ENTITY_WORDS],
        };
        if candidate.read_bits(1)? != 0 {
            let count = usize::try_from(candidate.read_bits(6)?)
                .map_err(|_| EntityBaselineError::FieldCount)?;
            if count == 0 {
                return Err(EntityBaselineError::FieldCount);
            }
            // All native lists begin with the same changed/8-bit type field. The
            // decoded type selects subsequent field order; unknown types use list 17.
            let first = tables::select(0)[0];
            let kind = fields::read(&mut candidate, first)?;
            baseline.words[first.word as usize] = kind;
            let table = tables::select(kind);
            if count > table.len() {
                return Err(EntityBaselineError::FieldCount);
            }
            for &field in &table[1..count] {
                baseline.words[field.word as usize] = fields::read(&mut candidate, field)?;
            }
        }
        self.indices = indices;
        *reader = candidate;
        Ok(baseline)
    }
}

impl fmt::Debug for EntityBaselineDecoder {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EntityBaselineDecoder")
            .finish_non_exhaustive()
    }
}
