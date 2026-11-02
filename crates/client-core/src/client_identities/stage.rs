// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{ClientIdentityBudget, ClientIdentityError, ClientIdentitySlot};
use crate::client_data::ClientData;

pub(super) fn inspect(
    records: usize,
    budget: ClientIdentityBudget,
) -> Result<(), ClientIdentityError> {
    if records > budget.max_records {
        return Err(ClientIdentityError::RecordBudget);
    }
    Ok(())
}
pub(super) fn upsert(
    slots: &mut [ClientIdentitySlot],
    used: &mut usize,
    data: &ClientData,
    budget: ClientIdentityBudget,
    output_capacity: usize,
) -> Result<(), ClientIdentityError> {
    match slots[..*used].binary_search_by_key(&data.index(), ClientIdentitySlot::key) {
        Ok(index) => slots[index].0 = Some(data.clone()),
        Err(index) => {
            if *used >= budget.max_clients {
                return Err(ClientIdentityError::ClientBudget);
            }
            if *used >= slots.len() {
                return Err(ClientIdentityError::ScratchCapacity);
            }
            if *used >= output_capacity {
                return Err(ClientIdentityError::OutputCapacity);
            }
            // Only private staging changes. Preserve sorted unique keys without allocation.
            slots[index..=*used].rotate_right(1);
            slots[index].0 = Some(data.clone());
            *used += 1;
        }
    }
    Ok(())
}
pub(super) fn generation(current: u64) -> Result<u64, ClientIdentityError> {
    current
        .checked_add(1)
        .ok_or(ClientIdentityError::GenerationExhausted)
}
#[cfg(test)]
mod tests {
    use super::{ClientIdentityError, generation};
    #[test]
    fn local_generation_exhaustion_never_wraps_to_a_reused_baseline() {
        assert_eq!(generation(0), Ok(1));
        assert_eq!(generation(u64::MAX - 1), Ok(u64::MAX));
        assert_eq!(
            generation(u64::MAX),
            Err(ClientIdentityError::GenerationExhausted)
        );
    }
}
