// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{Counts, GameStateBudget, GameStateError, Layout, sink::Sink, storage::Capacities};
use crate::{
    BaselineProtocol,
    client_data::{BASELINE_CLIENT_SLOTS, ClientData},
    gamestate::{
        configstrings::{ConfigStringBudget, scan},
        entity_baselines::EntityBaselineDecoder,
    },
    message_body::BodyReader,
};

pub(super) fn walk(
    reader: &mut BodyReader<'_>,
    profile: BaselineProtocol,
    budget: GameStateBudget,
    capacities: Capacities,
    scratch: &mut [u8],
    sink: &mut impl Sink,
) -> Result<Layout, GameStateError> {
    let start = reader.byte_position();
    let server_command_sequence = word(reader)?;
    let mut counts = Counts::default();
    let mut entities = EntityBaselineDecoder::new(profile)?;
    loop {
        if counts.services >= budget.max_services {
            return Err(GameStateError::ServiceBudget);
        }
        counts.services += 1;
        match reader.read_u8()? {
            2 => configstrings(
                reader,
                profile,
                budget,
                capacities,
                scratch,
                &mut counts,
                sink,
            )?,
            3 => {
                if counts.entities >= budget.max_entities {
                    return Err(GameStateError::EntityBudget);
                }
                if counts.entities >= capacities.entities {
                    return Err(GameStateError::EntityCapacity);
                }
                let baseline = entities.parse(reader)?;
                counts.entities += 1;
                sink.entity(baseline);
            }
            11 => {
                if counts.clients >= budget.max_client_records {
                    return Err(GameStateError::ClientBudget);
                }
                if counts.clients >= capacities.clients {
                    return Err(GameStateError::ClientCapacity);
                }
                let data = ClientData::parse(reader, profile)?;
                counts.clients += 1;
                sink.client(data);
            }
            7 => break,
            opcode => return Err(GameStateError::UnsupportedService(opcode)),
        }
    }
    let config_data_sequence = word(reader)?;
    let client_index = u32::try_from(reader.read_i32()?)
        .ok()
        .filter(|&index| index < BASELINE_CLIENT_SLOTS)
        .ok_or(GameStateError::ClientIndex)?;
    let checksum_feed = word(reader)?;
    Ok(Layout {
        counts,
        server_command_sequence,
        config_data_sequence,
        client_index,
        checksum_feed,
        consumed_bytes: reader.byte_position() - start,
    })
}

fn word(reader: &mut BodyReader<'_>) -> Result<u32, GameStateError> {
    Ok(u32::from_le_bytes(reader.read_i32()?.to_le_bytes()))
}

fn configstrings(
    reader: &mut BodyReader<'_>,
    profile: BaselineProtocol,
    budget: GameStateBudget,
    capacities: Capacities,
    scratch: &mut [u8],
    counts: &mut Counts,
    sink: &mut impl Sink,
) -> Result<(), GameStateError> {
    // Previous admitted counts never exceed their capacities/budgets. The complete
    // block's count is checked against both remaining record budgets before entries.
    let remaining = ConfigStringBudget {
        max_records: budget.max_configstrings - counts.configstrings,
        max_text_bytes: budget.max_text_bytes - counts.text_bytes,
    };
    let count = scan::count(
        reader,
        profile,
        remaining,
        capacities.configstrings - counts.configstrings,
    )?;
    let mut previous = None;
    for _ in 0..count {
        let (index, length) = scan::entry(reader, profile, &mut previous, scratch)?;
        counts.text_bytes = scan::text_size(
            counts.text_bytes,
            length,
            ConfigStringBudget {
                max_records: budget.max_configstrings,
                max_text_bytes: budget.max_text_bytes,
            },
            capacities.text,
        )?;
        counts.configstrings += 1;
        sink.configstring(index, &scratch[..length]);
    }
    Ok(())
}
