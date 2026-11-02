// SPDX-License-Identifier: AGPL-3.0-or-later
use super::super::{SNAPSHOT_OPCODE, SnapshotInput, SnapshotOutput, prepare as snapshot};
use super::{PreparedSnapshotServices, SnapshotServiceError};
use crate::{message_body::BodyReader, service_controls::ServiceControlBudget};

struct Inspection {
    maximum: u32,
    records: u32,
}
impl Inspection {
    fn next(&mut self, reader: &mut BodyReader<'_>) -> Result<u8, SnapshotServiceError> {
        if self.records >= self.maximum {
            return Err(SnapshotServiceError::RecordBudget);
        }
        if reader.remaining_bytes() == 0 {
            return Err(SnapshotServiceError::MissingEof);
        }
        let opcode = reader.read_u8().map_err(SnapshotServiceError::Body)?;
        self.records += 1;
        Ok(opcode)
    }
}
pub(super) fn prepare<'reader, 'body, 'source, 'storage>(
    reader: &'reader mut BodyReader<'body>,
    input: SnapshotInput<'source>,
    budget: ServiceControlBudget,
    output: SnapshotOutput<'storage>,
) -> Result<PreparedSnapshotServices<'reader, 'body, 'source, 'storage>, SnapshotServiceError> {
    if budget.max_records == 0 {
        return Err(SnapshotServiceError::InvalidBudget);
    }
    let start = reader.byte_position();
    let mut candidate = reader.clone();
    let mut inspection = Inspection {
        maximum: budget.max_records,
        records: 0,
    };
    loop {
        match inspection.next(&mut candidate)? {
            0 => {}
            SNAPSHOT_OPCODE => break,
            7 => return Err(SnapshotServiceError::MissingSnapshot),
            opcode => return Err(SnapshotServiceError::UnsupportedService(opcode)),
        }
    }
    let header_start = candidate.byte_position();
    let mut prepared = snapshot::prepare_at(reader, candidate, input, output)
        .map_err(SnapshotServiceError::Snapshot)?;
    let snapshot_bytes = prepared.transaction.candidate.byte_position() - header_start;
    let mut tail = prepared.transaction.candidate.clone();
    loop {
        match inspection.next(&mut tail)? {
            0 => {}
            7 => break,
            opcode => return Err(SnapshotServiceError::UnsupportedService(opcode)),
        }
    }
    let consumed_bytes = tail.byte_position() - start;
    prepared.transaction.candidate = tail;
    prepared.transaction.consumed_bytes = snapshot_bytes;
    Ok(PreparedSnapshotServices {
        snapshot: prepared,
        record_count: inspection.records,
        consumed_bytes,
    })
}
