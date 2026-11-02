// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{PreGameText, lexer};
use crate::service_controls::ServerCommand;

/// Typed inert notice. The host owns state gating, resource validation and effects.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreGameAction<'command> {
    /// Native `w`: terminate this connection under host policy, optionally displaying
    /// a checked/escaped reason. Missing reason differs from an explicit empty token.
    Disconnect {
        /// Normalized first argument, with its native 1023-byte copy boundary.
        reason: Option<PreGameText<'command>>,
        /// Native third-token `PB` classification, matched without ASCII case.
        /// This is peer-supplied metadata, never proof of an anti-cheat decision.
        punkbuster: bool,
    },
    /// Native `l`: inspect a requested map/gametype without opening files or running
    /// commands. Missing arguments are empty, matching native argument lookup.
    LoadMap {
        /// Inert map token; resolve through checked virtual content policy later.
        map: PreGameText<'command>,
        /// Inert gametype token; no script or console execution is implied.
        gametype: PreGameText<'command>,
    },
    /// Native `m`: request fast restart only if the host is in an active session.
    /// A classifier does not establish that phase or apply restart effects.
    FastRestart,
}

/// One recognized newly admitted command with its original positive sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreGameEvent<'command> {
    sequence: u32,
    action: PreGameAction<'command>,
}
impl<'command> PreGameEvent<'command> {
    pub(super) fn inspect(command: ServerCommand<'command>) -> Option<Self> {
        let bytes = command.as_bytes();
        let action = match bytes.first()? {
            b'm' => PreGameAction::FastRestart,
            b'w' => {
                let arguments = lexer::first_three(bytes);
                PreGameAction::Disconnect {
                    reason: arguments[1].map(|reason| PreGameText::new(reason, 1023)),
                    punkbuster: arguments[2].is_some_and(|kind| {
                        kind[..kind.len().min(1023)].eq_ignore_ascii_case(b"PB")
                    }),
                }
            }
            b'l' => {
                let arguments = lexer::first_three(bytes);
                PreGameAction::LoadMap {
                    map: PreGameText::new(arguments[1].unwrap_or_default(), 255),
                    gametype: PreGameText::new(arguments[2].unwrap_or_default(), 255),
                }
            }
            _ => return None,
        };
        Some(Self {
            sequence: command.sequence(),
            action,
        })
    }
    /// Returns the exact command sequence selected by history preparation.
    #[must_use]
    pub fn sequence(self) -> u32 {
        self.sequence
    }
    /// Returns the typed notice with borrowed inert text and no side effects.
    #[must_use]
    pub fn action(self) -> PreGameAction<'command> {
        self.action
    }
}
