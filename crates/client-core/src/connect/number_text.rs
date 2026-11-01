// SPDX-License-Identifier: AGPL-3.0-or-later
use std::fmt::{self, Write};

use super::ConnectPlanError;

pub(super) struct NumberText {
    bytes: [u8; 11],
    length: usize,
}

impl NumberText {
    pub(super) fn new(value: i32) -> Result<Self, ConnectPlanError> {
        let mut text = Self {
            bytes: [0; 11],
            length: 0,
        };
        write!(&mut text, "{value}").map_err(|_| ConnectPlanError::Encoding)?;
        Ok(text)
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes[..self.length]
    }
}

impl fmt::Write for NumberText {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        let end = self.length.checked_add(text.len()).ok_or(fmt::Error)?;
        let destination = self.bytes.get_mut(self.length..end).ok_or(fmt::Error)?;
        destination.copy_from_slice(text.as_bytes());
        self.length = end;
        Ok(())
    }
}
