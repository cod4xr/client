// SPDX-License-Identifier: AGPL-3.0-or-later
use std::{error::Error, fmt};

/// Fixed virtual file-key failures without source names or components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AssetKeyError {
    /// A relative file key cannot be empty.
    Empty,
    /// Original bytes exceed the host inspection budget.
    ByteBudget,
    /// Relative components exceed the host inspection budget.
    ComponentBudget,
    /// Final caller storage cannot hold the complete canonical bytes.
    OutputCapacity,
    /// A name begins with a separator rather than a relative file component.
    Rooted,
    /// A name ends with a separator and therefore is not a file key.
    Directory,
    /// Repeated separators introduce an ambiguous empty component.
    EmptyComponent,
    /// A dot component or native-forbidden double-dot substring is present.
    DotComponent,
    /// NUL, ASCII controls, DEL or ambiguous colon separators are present.
    Character,
}
impl fmt::Display for AssetKeyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Empty => "virtual file key empty",
            Self::ByteBudget => "virtual file-key byte budget exhausted",
            Self::ComponentBudget => "virtual file-key component budget exhausted",
            Self::OutputCapacity => "virtual file-key output capacity insufficient",
            Self::Rooted => "virtual file key is rooted",
            Self::Directory => "virtual file key has a directory marker",
            Self::EmptyComponent => "virtual file key has an empty component",
            Self::DotComponent => "virtual file key has an ambiguous dot component",
            Self::Character => "virtual file key has a forbidden character",
        })
    }
}
impl Error for AssetKeyError {}
