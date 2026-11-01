// SPDX-License-Identifier: AGPL-3.0-or-later
/// Opaque caller-owned metadata for one sequence-tagged server-command slot.
/// Construct with `Default`; history initialization resets tags without exposing dirty bytes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ServerCommandSlot {
    pub(super) sequence: u32,
    pub(super) length: usize,
}
