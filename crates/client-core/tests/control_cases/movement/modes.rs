// SPDX-License-Identifier: AGPL-3.0-or-later
use super::support::{body, build, commands, snapshot};
use cod4x_protocol::MessageReader;
use cod4xr_client_core::BaselineProtocol;

#[test]
fn only_an_exact_current_message_snapshot_selects_delta_and_frame_outlives_its_source() {
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let input = commands(1);
        for (sequence, expected_opcode) in [(6, 1), (7, 0)] {
            let frame = snapshot(profile, sequence, |source| {
                build(profile, 7, b"Private.After", &input, &[], Some(source))
            });
            let decoded = body(&frame, profile, 7, b"Private.After");
            let mut reader =
                MessageReader::try_new(&decoded).expect("Body after snapshot storage expired");
            assert_eq!(
                reader.read_bits(3).expect("Native movement mode"),
                expected_opcode
            );
            assert_eq!(reader.read_u8().expect("Count"), 1);
        }
    }
}
