// SPDX-License-Identifier: AGPL-3.0-or-later
use super::EntityBaselineError;
use std::fmt;

/// Explicit checked native coordinate biases for protocol-6 initial baselines.
/// Values are inert geometry metadata, not authority to load a map or select a profile.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct LegacyMapCenter {
    pub(super) biases: [i32; 3],
}
impl LegacyMapCenter {
    /// Checks the three native map-center floats and their half-unit bias/truncation.
    /// The reviewed x87 reader keeps the addition precise until integer conversion;
    /// binary64 reproduces it exactly throughout the accepted signed-32 range.
    /// The host supplies the reviewed current center; no remote configstring, locale,
    /// global setting or zero-center fallback is implicitly consulted.
    ///
    /// # Errors
    /// Rejects nonfinite values or a rounded component outside signed-32 conversion.
    /// No partial context escapes when a later component fails.
    pub fn new(center: [f32; 3]) -> Result<Self, EntityBaselineError> {
        let mut biases = [0; 3];
        for (target, value) in biases.iter_mut().zip(center) {
            *target = bias(value)?;
        }
        Ok(Self { biases })
    }
    /// Returns the three exact native integer biases as inert data, not log text.
    #[must_use]
    pub fn biases(self) -> [i32; 3] {
        self.biases
    }
}
#[allow(
    clippy::cast_possible_truncation,
    reason = "Finite truncated native bias is checked against exact signed-32 bounds first."
)]
fn bias(value: f32) -> Result<i32, EntityBaselineError> {
    let truncated = (f64::from(value) + 0.5).trunc();
    if !truncated.is_finite() || !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&truncated) {
        return Err(EntityBaselineError::InvalidMapCenter);
    }
    Ok(truncated as i32)
}
impl fmt::Debug for LegacyMapCenter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LegacyMapCenter").finish_non_exhaustive()
    }
}
