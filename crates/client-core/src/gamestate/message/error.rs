// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{
    client_data::ClientDataError,
    gamestate::{configstrings::ConfigStringError, entity_baselines::EntityBaselineError},
    message_body::BodyReadError,
};
use std::{error::Error, fmt};

/// Fixed whole-gamestate validation errors, excluding private peer state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GameStateError {
    /// The explicitly selected profile has no complete reviewed decoder here.
    UnsupportedProfile,
    /// Service inspection work exceeds the host budget before an admitted EOF.
    ServiceBudget,
    /// Entity records exceed the host budget.
    EntityBudget,
    /// Client identity records exceed the host budget.
    ClientBudget,
    /// Caller entity storage cannot retain another complete record.
    EntityCapacity,
    /// Caller client identity storage cannot retain another complete record.
    ClientCapacity,
    /// The trailer's own-client identifier is outside the baseline namespace.
    ClientIndex,
    /// This service is not a valid reviewed gamestate constituent.
    UnsupportedService(u8),
    /// A configstring block or its caller capacities failed validation.
    ConfigStrings(ConfigStringError),
    /// An entity baseline failed validation.
    Entity(EntityBaselineError),
    /// A client identity record failed validation.
    Client(ClientDataError),
    /// A header, opcode or trailer was invalid/truncated.
    Body(BodyReadError),
}

impl fmt::Display for GameStateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedProfile => "gamestate unsupported by selected profile",
            Self::ServiceBudget => "gamestate service budget exhausted",
            Self::EntityBudget => "gamestate entity budget exhausted",
            Self::ClientBudget => "gamestate client record budget exhausted",
            Self::EntityCapacity => "gamestate entity storage capacity insufficient",
            Self::ClientCapacity => "gamestate client storage capacity insufficient",
            Self::ClientIndex => "gamestate own-client index outside baseline namespace",
            Self::UnsupportedService(_) => "unsupported gamestate service",
            Self::ConfigStrings(_) => "invalid gamestate configstring block",
            Self::Entity(_) => "invalid gamestate entity baseline",
            Self::Client(_) => "invalid gamestate client identity",
            Self::Body(_) => "invalid or truncated gamestate framing",
        })
    }
}
impl Error for GameStateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::ConfigStrings(error) => Some(error),
            Self::Entity(error) => Some(error),
            Self::Client(error) => Some(error),
            Self::Body(error) => Some(error),
            _ => None,
        }
    }
}
impl From<BodyReadError> for GameStateError {
    fn from(value: BodyReadError) -> Self {
        Self::Body(value)
    }
}
impl From<ConfigStringError> for GameStateError {
    fn from(value: ConfigStringError) -> Self {
        Self::ConfigStrings(value)
    }
}
impl From<EntityBaselineError> for GameStateError {
    fn from(value: EntityBaselineError) -> Self {
        Self::Entity(value)
    }
}
impl From<ClientDataError> for GameStateError {
    fn from(value: ClientDataError) -> Self {
        Self::Client(value)
    }
}
