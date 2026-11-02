// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::PredictionSample;
use std::fmt;

/// Opaque caller-funded prediction lookup cell, without peer-selected allocation.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct PredictionSlot(pub(super) Option<PredictionSample>);

impl fmt::Debug for PredictionSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("PredictionSlot { .. }")
    }
}
