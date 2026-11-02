// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{ClientState, ClientStateDecoder, ClientStateDelta};
use super::{
    ClientStateSource, ClientStreamBudget, ClientStreamError, ClientStreamStats, sink::Sink,
};
use crate::{BaselineProtocol, entities::ordered::Cursor, message_body::BodyReader};

#[derive(Clone, Copy)]
pub(crate) struct Input<'source> {
    pub profile: BaselineProtocol,
    pub previous: ClientStateSource<'source>,
    pub budget: ClientStreamBudget,
    pub capacity: usize,
}
pub(crate) fn walk(
    reader: &mut BodyReader<'_>,
    input: Input<'_>,
    sink: &mut impl Sink,
) -> Result<ClientStreamStats, ClientStreamError> {
    let mut decoder = ClientStateDecoder::new(input.profile);
    let mut cursor = Cursor::new(input.previous);
    let mut stats = ClientStreamStats::default();
    loop {
        if stats.wire_records >= input.budget.max_wire_records {
            return Err(ClientStreamError::RecordBudget);
        }
        let prepared = decoder.prepare(reader)?;
        stats.wire_records += 1;
        let Some(prepared) = prepared else { break };
        let number = prepared.number();
        while let Some(previous) = cursor.before(number) {
            emit(previous.clone(), &input, &mut stats, sink)?;
            stats.unchanged_clients += 1;
        }
        let reference = cursor.matching(number);
        if reference.is_none() {
            stats.zero_references += 1;
        }
        stats.updates += 1;
        match prepared.decode(reference)? {
            ClientStateDelta::State(client) => emit(client, &input, &mut stats, sink)?,
            ClientStateDelta::Removed { .. } => stats.removals += 1,
        }
    }
    while let Some(previous) = cursor.remaining() {
        emit(previous.clone(), &input, &mut stats, sink)?;
        stats.unchanged_clients += 1;
    }
    Ok(stats)
}
fn emit(
    client: ClientState,
    input: &Input<'_>,
    stats: &mut ClientStreamStats,
    sink: &mut impl Sink,
) -> Result<(), ClientStreamError> {
    if stats.output_clients >= input.budget.max_clients {
        return Err(ClientStreamError::ClientBudget);
    }
    if stats.output_clients >= input.capacity {
        return Err(ClientStreamError::OutputCapacity);
    }
    sink.client(client);
    stats.output_clients += 1;
    Ok(())
}
