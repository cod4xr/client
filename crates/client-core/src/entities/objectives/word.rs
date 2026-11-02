// SPDX-License-Identifier: AGPL-3.0-or-later
/// Inert objective field words independent of native C offsets and wire order.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum ObjectiveWord {
    /// Origin X as raw binary32 bits.
    OriginX,
    /// Origin Y as raw binary32 bits.
    OriginY,
    /// Origin Z as raw binary32 bits.
    OriginZ,
    /// Entity reference; no entity lookup or engine admission is implied.
    Entity,
    /// Team identifier; no gameplay policy is applied.
    Team,
    /// Icon resource index; no asset resolution is performed.
    Icon,
}
