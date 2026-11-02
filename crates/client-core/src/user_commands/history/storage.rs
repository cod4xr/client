// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{super::UserCommandInput, InputCommandId};
use std::fmt;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) struct Record {
    pub id: InputCommandId,
    pub input: UserCommandInput,
}
/// Caller-funded opaque input cell; old contents carry no validity in a new owner.
/// Logical retirement/reset does not promise secure erasure of private input data.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct InputCommandSlot(pub(super) Option<Record>);
impl fmt::Debug for InputCommandSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InputCommandSlot").finish_non_exhaustive()
    }
}
