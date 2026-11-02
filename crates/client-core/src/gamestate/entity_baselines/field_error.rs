// SPDX-License-Identifier: AGPL-3.0-or-later
use super::EntityBaselineError;
use crate::message_body::BodyReadError;

#[derive(Debug)]
pub(super) enum FieldReadError {
    Body(BodyReadError),
    Encoding,
    ReferenceFloat,
}
impl From<BodyReadError> for FieldReadError {
    fn from(error: BodyReadError) -> Self {
        Self::Body(error)
    }
}
impl From<FieldReadError> for EntityBaselineError {
    fn from(error: FieldReadError) -> Self {
        match error {
            FieldReadError::Body(error) => Self::Body(error),
            FieldReadError::Encoding | FieldReadError::ReferenceFloat => Self::FieldEncoding,
        }
    }
}
