// SPDX-License-Identifier: AGPL-3.0-or-later
use super::ReliableEndpoint;
use crate::framing::{FrameError, FrameProgress};

/// One drain's accounted work, preserved even when the frame prefix is invalid.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DrainUsage {
    /// Contiguous stream bytes consumed and copied into the frame assembler.
    pub bytes: usize,
    /// Fully consumed fragments, including empty ones.
    pub fragments: usize,
    /// Inspected ready fragments, counting a partial head on each read.
    pub examined: usize,
}

/// Non-error frame status at a bounded drain boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrainStatus {
    /// Needs more admitted stream bytes or another host work budget.
    Pending,
    /// One complete body awaits host validation and consumption.
    Ready,
}

/// Bounded work plus outcome, including terminal framing failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DrainRun {
    /// Exact work charged before the outcome.
    pub usage: DrainUsage,
    /// Completion, pending progress, or a permanently invalid frame prefix.
    pub outcome: Result<DrainStatus, FrameError>,
}

impl ReliableEndpoint<'_> {
    /// Moves bounded stream bytes toward one complete frame without reading past its end.
    /// Caps every read at the remaining header/body need, caller scratch, byte budget and
    /// fragment-inspection quota. Ready frames stop all further stream consumption.
    /// Empty scratch or zero quota does no work; empty fragments still charge inspections.
    /// Invalid prefixes terminally fail the endpoint; usage remains available on failure.
    #[must_use]
    pub fn drain(
        &mut self,
        scratch: &mut [u8],
        max_bytes: usize,
        max_fragments: usize,
    ) -> DrainRun {
        let mut usage = DrainUsage::default();
        let outcome = self.drain_inner(scratch, max_bytes, max_fragments, &mut usage);
        DrainRun { usage, outcome }
    }

    fn drain_inner(
        &mut self,
        scratch: &mut [u8],
        max_bytes: usize,
        max_fragments: usize,
        usage: &mut DrainUsage,
    ) -> Result<DrainStatus, FrameError> {
        if self.frames.is_failed() {
            return Err(FrameError::Failed);
        }
        while usage.bytes < max_bytes && usage.examined < max_fragments && !scratch.is_empty() {
            let need = self.frames.next_input_bytes()?;
            if need == 0 {
                break;
            }
            let length = need.min(scratch.len()).min(max_bytes - usage.bytes);
            let read = self
                .incoming
                .read(&mut scratch[..length], max_fragments - usage.examined);
            usage.bytes += read.bytes;
            usage.fragments += read.fragments;
            usage.examined += read.examined;
            if read.examined == 0 {
                break;
            }
            let FrameProgress { consumed, .. } = self.frames.push(&scratch[..read.bytes])?;
            debug_assert_eq!(
                consumed, read.bytes,
                "drain never reads across a frame boundary"
            );
        }
        Ok(if self.frames.frame().is_some() {
            DrainStatus::Ready
        } else {
            DrainStatus::Pending
        })
    }
}
