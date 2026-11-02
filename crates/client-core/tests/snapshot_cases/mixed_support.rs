// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    Change, EntityDecodeContext, MessageWriter, SnapshotClientSlot, SnapshotEntitySlot,
    SnapshotInput, SnapshotMessage, client, dirty, encoded, entity, input, prefix,
};
use cod4xr_client_core::{
    BaselineProtocol,
    service_controls::{ClientDataUpdateSlot, ClientServiceBudget, ClientServiceStorage},
    snapshots::mixed_services::MixedSnapshotStorage,
};

pub(super) const LIMITS: ClientServiceBudget = ClientServiceBudget {
    max_records: 32,
    max_client_updates: 4,
};
pub(super) fn selected(context: EntityDecodeContext, sequence: u32) -> SnapshotInput<'static> {
    let mut result = input(context, sequence, None);
    result.message = SnapshotMessage::new(7, sequence, 3).expect("Initial host command prefix");
    result
}
pub(super) fn command(writer: &mut MessageWriter<'_>, sequence: i32, text: &[u8]) {
    writer.write_u8(4).expect("Command service");
    writer.write_i32(sequence).expect("Command sequence");
    string(writer, text);
}
pub(super) fn identity(writer: &mut MessageWriter<'_>, sequence: i32, name: &[u8]) {
    writer.write_u8(11).expect("Identity service");
    writer.write_i32(sequence).expect("Identity sequence");
    writer.write_u8(2).expect("Client index");
    string(writer, name);
    string(writer, b"TAG");
}
fn string(writer: &mut MessageWriter<'_>, text: &[u8]) {
    for &byte in text {
        writer.write_u8(byte).expect("Text byte");
    }
    writer.write_u8(0).expect("Native terminator");
}
pub(super) fn snapshot(writer: &mut MessageWriter<'_>, distance: u8, transmitted: bool) {
    writer.write_u8(6).expect("Snapshot service");
    prefix(writer, distance, transmitted);
    if distance == 0 {
        entity(writer, Change::Set(1, 0, 13));
    } else {
        entity(writer, Change::Keep(1));
    }
    writer.write_bits(0xffe, 12).expect("Entity terminator");
    if distance == 0 {
        client(writer, Change::Set(63, 0, 511));
    } else {
        client(writer, Change::Keep(63));
    }
    writer.write_bits(0, 1).expect("Client end");
}
pub(super) fn carrier(profile: BaselineProtocol, distance: u8, transmitted: bool) -> Vec<u8> {
    encoded(|writer| {
        writer.write_u8(0).expect("NOP");
        command(writer, 4, b"alpha%");
        command(writer, 2, b"older");
        if profile == BaselineProtocol::Cod4x21 {
            identity(writer, 1, b"Before%");
        }
        snapshot(writer, distance, transmitted);
        if profile == BaselineProtocol::Cod4x21 {
            identity(writer, 2, b"After");
        }
        command(writer, 9, b"omega");
        writer.write_u8(0).expect("NOP");
        writer.write_u8(7).expect("EOF");
    })
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Finals {
    pub text: Vec<u8>,
    pub identities: Vec<ClientDataUpdateSlot>,
    pub entities: Vec<SnapshotEntitySlot>,
    pub clients: Vec<SnapshotClientSlot>,
}
impl Finals {
    pub fn new() -> Self {
        let (entities, clients) = dirty();
        Self {
            text: vec![0xa5; 128],
            identities: vec![ClientDataUpdateSlot::default(); 4],
            entities,
            clients,
        }
    }
    pub fn storage<'scratch>(
        &mut self,
        text_scratch: &'scratch mut [u8],
        identity_scratch: &'scratch mut [ClientDataUpdateSlot],
    ) -> MixedSnapshotStorage<'scratch, '_> {
        MixedSnapshotStorage {
            text_scratch,
            controls: ClientServiceStorage {
                client_scratch: identity_scratch,
                client_updates: &mut self.identities,
                command_bytes: &mut self.text,
            },
            snapshot: super::output(&mut self.entities, &mut self.clients),
        }
    }
}
