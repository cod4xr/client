// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt;

use super::{FRAME_PREFIX_BYTES, FrameError, ReliableFrame, body_length, required_frame_bytes};

/// Work for one bounded push. Unconsumed input belongs to the next application frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct FrameProgress {
    /// Stream bytes admitted from the input prefix, including any length-header bytes.
    pub consumed: usize,
    /// One complete frame is retained until explicitly consumed by the host.
    pub complete: bool,
}

/// Incremental application framing over fixed caller storage, at most one frame per push.
/// Partial length bytes and body bytes remain private until a whole frame is complete.
pub struct FrameAssembler<'storage> {
    body: &'storage mut [u8],
    prefix: [u8; FRAME_PREFIX_BYTES],
    prefix_bytes: usize,
    expected: Option<usize>,
    filled: usize,
    ready: bool,
    failed: bool,
}

impl<'storage> FrameAssembler<'storage> {
    /// Starts fresh framing without modifying caller body storage.
    /// The storage length is the explicit body budget, never a peer's claimed length.
    ///
    /// # Errors
    /// Rejects host storage whose complete encoded size is not representable.
    pub fn new(body: &'storage mut [u8]) -> Result<Self, FrameError> {
        required_frame_bytes(body.len())?;
        Ok(Self {
            body,
            prefix: [0; FRAME_PREFIX_BYTES],
            prefix_bytes: 0,
            expected: None,
            filled: 0,
            ready: false,
            failed: false,
        })
    }

    /// Admits bytes for at most one frame. A retained complete frame consumes no new input.
    /// The caller must preserve any unconsumed suffix and enforce a per-tick frame quota.
    /// Partial headers of one to three bytes are retained across calls.
    ///
    /// # Errors
    /// Negative/unrepresentable/over-budget lengths poison the stream. No resynchronization
    /// is attempted: later calls return `Failed` and no incomplete frame is exposed.
    pub fn push(&mut self, input: &[u8]) -> Result<FrameProgress, FrameError> {
        if self.failed {
            return Err(FrameError::Failed);
        }
        if self.ready {
            return Ok(FrameProgress {
                consumed: 0,
                complete: true,
            });
        }
        let take = (FRAME_PREFIX_BYTES - self.prefix_bytes).min(input.len());
        self.prefix[self.prefix_bytes..self.prefix_bytes + take].copy_from_slice(&input[..take]);
        self.prefix_bytes += take;
        let mut consumed = take;
        if self.prefix_bytes < FRAME_PREFIX_BYTES {
            return Ok(FrameProgress {
                consumed,
                complete: false,
            });
        }
        let expected = if let Some(expected) = self.expected {
            expected
        } else {
            let expected = match body_length(self.prefix) {
                Ok(expected) => expected,
                Err(error) => {
                    self.failed = true;
                    return Err(error);
                }
            };
            if expected > self.body.len() {
                self.failed = true;
                return Err(FrameError::BodyCapacity);
            }
            self.expected = Some(expected);
            expected
        };
        let take = (expected - self.filled).min(input.len() - consumed);
        self.body[self.filled..self.filled + take]
            .copy_from_slice(&input[consumed..consumed + take]);
        self.filled += take;
        consumed += take;
        self.ready = self.filled == expected;
        Ok(FrameProgress {
            consumed,
            complete: self.ready,
        })
    }

    /// Borrows only a complete body, without exposing dirty bytes or an incomplete prefix.
    /// The borrow prevents storage reuse while its frame is inspected.
    ///
    /// ```compile_fail
    /// use cod4xr_reliable::framing::FrameAssembler;
    /// let mut storage = [0; 8];
    /// let mut stream = FrameAssembler::new(&mut storage)?;
    /// stream.push(&[1, 0, 0, 0, 42])?;
    /// let frame = stream.frame().ok_or("incomplete")?;
    /// assert!(stream.consume_ready());
    /// assert_eq!(frame.body(), &[42]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    #[must_use]
    pub fn frame(&self) -> Option<ReliableFrame<'_>> {
        self.ready.then(|| ReliableFrame {
            body: &self.body[..self.filled],
        })
    }

    /// Releases a complete frame for the next length prefix. Incomplete/failed streams
    /// are unchanged. Returns whether a retained complete frame was released.
    #[must_use]
    pub fn consume_ready(&mut self) -> bool {
        if !self.ready || self.failed {
            return false;
        }
        self.prefix_bytes = 0;
        self.expected = None;
        self.filled = 0;
        self.ready = false;
        true
    }

    /// Returns the host-selected body budget without inspecting peer metadata.
    #[must_use]
    pub fn capacity(&self) -> usize {
        self.body.len()
    }
    /// Returns admitted length/header bytes, including incomplete private body storage.
    #[must_use]
    pub fn buffered_bytes(&self) -> usize {
        self.prefix_bytes + self.filled
    }
    /// Returns whether an invalid length permanently failed this stream instance.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.failed
    }
}

impl fmt::Debug for FrameAssembler<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FrameAssembler")
            .field("capacity", &self.capacity())
            .field("buffered_bytes", &self.buffered_bytes())
            .field("ready", &self.ready)
            .field("failed", &self.failed)
            .finish_non_exhaustive()
    }
}
