// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientDataUpdateSlot, ServiceControlError};
use crate::{BaselineProtocol, client_data::ClientDataUpdate, message_body::BodyReader};

pub(super) enum Clients<'scratch> {
    Reject,
    Accept {
        maximum: usize,
        capacity: usize,
        scratch: &'scratch mut [ClientDataUpdateSlot],
    },
}
impl Clients<'_> {
    pub fn read(
        &mut self,
        index: usize,
        reader: &mut BodyReader<'_>,
    ) -> Result<(), ServiceControlError> {
        match self {
            Self::Reject => Err(ServiceControlError::UnsupportedService(11)),
            Self::Accept {
                maximum,
                capacity,
                scratch,
            } => {
                if index >= *maximum {
                    return Err(ServiceControlError::ClientDataBudget);
                }
                if index >= scratch.len() {
                    return Err(ServiceControlError::ClientDataScratch);
                }
                if index >= *capacity {
                    return Err(ServiceControlError::ClientDataCapacity);
                }
                let update = ClientDataUpdate::parse(reader, BaselineProtocol::Cod4x21)
                    .map_err(ServiceControlError::ClientData)?;
                scratch[index].0 = Some(update);
                Ok(())
            }
        }
    }
}
