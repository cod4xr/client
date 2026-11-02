// SPDX-License-Identifier: AGPL-3.0-or-later
//! Owned client identity fixtures and complete-record rollback across both carriers.
#![allow(
    clippy::expect_used,
    reason = "Finite checked native identity fixtures."
)]

#[path = "client_data_cases/failures.rs"]
mod failures;
#[path = "client_data_cases/vectors.rs"]
mod vectors;

use cod4xr_client_core::{
    BaselineProtocol,
    client_data::{ClientData, ClientDataError, ClientDataUpdate},
    message_body::BodyReader,
};

fn identity(index: u8, name: &[u8], tag: &[u8]) -> Vec<u8> {
    let mut input = vec![index];
    input.extend_from_slice(name);
    input.push(0);
    input.extend_from_slice(tag);
    input.push(0);
    input
}

fn carrier(record: &[u8], sequenced: bool) -> Vec<u8> {
    let mut input = vec![0xdd]; // Existing bit cursor is independent of byte fields.
    if sequenced {
        input.extend_from_slice(&42_i32.to_le_bytes());
    }
    input.extend_from_slice(record);
    input
}

fn state(reader: &BodyReader<'_>) -> (usize, usize, u8) {
    (
        reader.byte_position(),
        reader.remaining_bytes(),
        reader.remaining_cached_bits(),
    )
}

fn parse(
    reader: &mut BodyReader<'_>,
    profile: BaselineProtocol,
    sequenced: bool,
) -> Result<ClientData, ClientDataError> {
    if sequenced {
        ClientDataUpdate::parse(reader, profile).map(|update| update.data().clone())
    } else {
        ClientData::parse(reader, profile)
    }
}
