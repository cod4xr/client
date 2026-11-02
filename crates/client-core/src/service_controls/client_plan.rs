// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    ClientServiceBudget, ClientServiceStorage, LegacyServicePlan, ServiceControlBudget,
    ServiceControlError, client_scan::Clients, publish, scan,
};
use crate::{BaselineProtocol, server_payload::DecodedServerPayload};

impl<'output> LegacyServicePlan<'output> {
    /// Validates complete mixed protocol-21 command/identity controls through EOF.
    /// Both final command bytes and identity cells remain unchanged on every error.
    /// Private text/update scratch is unspecified on failure. Ordinary compression
    /// profile limits stay unchanged; snapshots/downloads/gamestate need their own
    /// decoders and are never skipped. No histories, sequence owners or ACKs mutate.
    ///
    /// # Errors
    /// Rejects unsupported profiles/services, malformed records, missing EOF,
    /// record/identity/command budgets and all insufficient caller capacities.
    pub fn parse_with_client_data(
        payload: &DecodedServerPayload<'_, '_>,
        budget: ClientServiceBudget,
        scratch: &mut [u8],
        storage: ClientServiceStorage<'_, 'output>,
    ) -> Result<Self, ServiceControlError> {
        if payload.profile() != BaselineProtocol::Cod4x21 {
            return Err(ServiceControlError::Profile);
        }
        let ClientServiceStorage {
            client_scratch,
            client_updates,
            command_bytes,
        } = storage;
        let layout = scan::scan(
            payload.body(),
            ServiceControlBudget {
                max_records: budget.max_records,
            },
            scratch,
            command_bytes.len(),
            &mut Clients::Accept {
                maximum: budget.max_client_updates,
                capacity: client_updates.len(),
                scratch: client_scratch,
            },
        )?;
        publish::commands(payload.body(), &layout, command_bytes)?;
        let count = layout.client_update_count;
        client_updates[..count].clone_from_slice(&client_scratch[..count]);
        Ok(Self::from_layout(
            payload,
            layout,
            command_bytes,
            &client_updates[..count],
        ))
    }
}
