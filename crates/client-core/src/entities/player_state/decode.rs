// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{LivePlayerState, PendingPlayerState, PlayerStateError, PreparedPlayerState};
use crate::{entities::EntityDecodeContext, message_body::BodyReader};

pub(super) fn prepare<'reader, 'body>(
    reader: &'reader mut BodyReader<'body>,
    context: EntityDecodeContext,
    time: u32,
    reference: Option<&LivePlayerState>,
) -> Result<PreparedPlayerState<'reader, 'body>, PlayerStateError> {
    let mut candidate = reader.clone();
    let pending = PendingPlayerState::decode_candidate(&mut candidate, context, time, reference)?;
    Ok(PreparedPlayerState {
        consumed_bytes: candidate.byte_position() - reader.byte_position(),
        reader,
        candidate,
        pending,
    })
}
