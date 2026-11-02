// SPDX-License-Identifier: AGPL-3.0-or-later
use super::PredictionWords;
use crate::BaselineProtocol;
use std::fmt;

/// How the eleven native live prediction words were resolved.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredictionProvenance {
    /// The wire mode selected transmitted origin/velocity state.
    Transmitted,
    /// A caller-owned prediction sample matched profile and command time.
    CommandTimeMatched,
    /// The caller explicitly selected the native prior-state fallback.
    ExplicitPriorFallback,
}
/// Exact lookup key; the host must additionally bind its sample queue to the session.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredictionRequest {
    pub(super) profile: BaselineProtocol,
    pub(super) command_time: u32,
}
impl PredictionRequest {
    /// Returns the selected baseline profile, not proof of a session epoch.
    #[must_use]
    pub fn profile(self) -> BaselineProtocol {
        self.profile
    }
    /// Returns the exact decoded wrapping command-time word.
    #[must_use]
    pub fn command_time(self) -> u32 {
        self.command_time
    }
}
impl fmt::Debug for PredictionRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionRequest")
            .field("profile", &self.profile)
            .finish_non_exhaustive()
    }
}
/// Caller-owned inert prediction words; construction alone confers no session authority.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PredictionSample {
    pub(super) request: PredictionRequest,
    pub(super) words: PredictionWords,
}
impl PredictionSample {
    /// Binds eleven words to an exact command time and explicit baseline profile.
    #[must_use]
    pub fn new(profile: BaselineProtocol, command_time: u32, words: PredictionWords) -> Self {
        Self {
            request: PredictionRequest {
                profile,
                command_time,
            },
            words,
        }
    }
    /// Returns the sample's explicit baseline profile.
    #[must_use]
    pub fn profile(&self) -> BaselineProtocol {
        self.request.profile
    }
    /// Returns the sample's complete wrapping command-time word.
    #[must_use]
    pub fn command_time(&self) -> u32 {
        self.request.command_time
    }
    /// Returns the inert prediction words without applying them.
    #[must_use]
    pub fn words(&self) -> PredictionWords {
        self.words
    }
}
impl fmt::Debug for PredictionSample {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PredictionSample")
            .field("profile", &self.profile())
            .finish_non_exhaustive()
    }
}
