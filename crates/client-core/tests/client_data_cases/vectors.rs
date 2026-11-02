// SPDX-License-Identifier: AGPL-3.0-or-later
use crate::{carrier, identity, parse};
use cod4x_protocol::MAX_MESSAGE_BYTES;
use cod4xr_client_core::{
    BaselineProtocol,
    client_data::{ClientData, ClientDataUpdate},
    message_body::BodyReader,
};

#[test]
fn independent_owned_identity_survives_input_release_and_keeps_text_out_of_debug() {
    for padding in [0, MAX_MESSAGE_BYTES + 7] {
        let data = {
            let mut input = vec![0xfe; padding];
            input.extend_from_slice(&[63, b'P', b'%', 0x92, 0, b'X', b'%', 0, 7]);
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            reader.read_bytes(&mut vec![0; padding]).expect("Prefix");
            let data = ClientData::parse(&mut reader, BaselineProtocol::Cod4x21)
                .expect("Independent native byte fields");
            assert_eq!(reader.read_u8(), Ok(7));
            assert_eq!(reader.remaining_bytes(), 0);
            data
        };
        assert_eq!(data.index(), 63);
        assert_eq!(data.name(), b"P.\x92");
        assert_eq!(data.clan_tag(), b"X.");
        let clone = data.clone();
        assert_eq!(data, clone);
        let debug = format!("{data:?}");
        assert!(!debug.contains("P."));
        assert!(!debug.contains("X."));
        assert!(!debug.contains("146"));
        assert!(!debug.contains("[80"));
    }
}

#[test]
fn empty_and_maximum_native_fields_consume_terminators_preserving_existing_cached_bits() {
    for sequenced in [false, true] {
        for (name, tag) in [(&b""[..], &b""[..]), (&[b'%'; 32][..], &[0x92; 12][..])] {
            for index in [0, 63] {
                let mut input = carrier(&identity(index, name, tag), sequenced);
                input.push(7);
                let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
                assert_eq!(reader.read_bits(3), Ok(5));
                let data = parse(&mut reader, BaselineProtocol::Cod4x21, sequenced)
                    .expect("Exact field capacities");
                assert_eq!(data.index(), u32::from(index));
                assert_eq!(data.name(), vec![b'.'; name.len()]);
                assert_eq!(data.clan_tag(), tag);
                assert_eq!(reader.read_u8(), Ok(7));
                assert_eq!(reader.read_bits(5), Ok(27));
                assert_eq!(reader.remaining_bytes(), 0);
            }
        }
    }
}

#[test]
fn sequenced_updates_retain_all_wire_bits_without_granting_ordering_or_application() {
    for (bytes, expected) in [
        ([0, 0, 0, 0], 0),
        ([1, 0, 0, 0], 1),
        ([0xff, 0xff, 0xff, 0x7f], 0x7fff_ffff),
        ([0, 0, 0, 0x80], 0x8000_0000),
        ([0xff; 4], u32::MAX),
    ] {
        let update = {
            let mut input = bytes.to_vec();
            input.extend_from_slice(&[3, b'S', b'%', 0, b'Q', b'%', 0, 7]);
            let mut reader = BodyReader::try_new(&input, input.len()).expect("Body");
            let update = ClientDataUpdate::parse(&mut reader, BaselineProtocol::Cod4x21)
                .expect("Independent sequence and identity fields");
            assert_eq!(reader.read_u8(), Ok(7));
            update
        };
        assert_eq!(update.sequence(), expected);
        assert_eq!(update.data().index(), 3);
        assert_eq!(update.data().name(), b"S.");
        assert_eq!(update.data().clan_tag(), b"Q.");
        let debug = format!("{update:?}");
        assert!(!debug.contains("S."));
        assert!(!debug.contains("Q."));
    }
}
