// SPDX-License-Identifier: AGPL-3.0-or-later
#![allow(
    clippy::expect_used,
    reason = "Independent cached-cursor command vectors."
)]
use super::{AbsoluteUserCommandPlan, MovementAxis, UserCommandInput, UserCommandMelee};
use cod4x_protocol::{
    MessageReader, MessageWriter,
    user_command::{self, LegacyUserCommand},
};

#[test]
fn private_packet_writer_preserves_prefix_cache_and_following_byte_bit_fields() {
    for width in 0..8 {
        let prefix = 0x55 & ((1_u32 << width) - 1);
        let input = UserCommandInput {
            server_time: i32::MIN,
            buttons: 0x10005,
            angles: [0x1234, 0xffff, 0x8000],
            weapon: 127,
            offhand: 126,
            movement: [MovementAxis::Negative, MovementAxis::Positive],
            selected_location: Some([0x92, 0xff]),
            melee: Some(UserCommandMelee {
                yaw: 0xc000,
                distance: 255,
            }),
        };
        let plan = AbsoluteUserCommandPlan::new(0xdead_beef, input).expect("Checked command");
        let mut bytes = [0xa5; 64];
        let mut writer = MessageWriter::try_new(&mut bytes).expect("Private packet scratch");
        writer
            .write_bits(prefix, width)
            .expect("Existing bit prefix");
        writer.write_u8(0x7d).expect("Intervening byte");
        plan.write(&mut writer).expect("Shared packet writer");
        writer.write_bits(5, 3).expect("Following cached bits");
        writer.write_u8(0x6e).expect("Following byte");
        let mut reader = MessageReader::try_new(writer.as_bytes()).expect("Complete stream");
        assert_eq!(reader.read_bits(width).expect("Prefix"), prefix);
        assert_eq!(reader.read_u8().expect("Prefix byte"), 0x7d);
        let command =
            user_command::read_delta(&mut reader, 0xdead_beef, &LegacyUserCommand::default())
                .expect("Shared decoder");
        assert_eq!(command.server_time, i32::MIN);
        assert_eq!(command.angles, input.angles);
        assert_eq!(command.buttons, input.buttons);
        assert_eq!((command.forward_move, command.right_move), (-127, 127));
        assert_eq!(command.selected_location, [0x92, 0xff]);
        assert_eq!(command.melee_charge_yaw.to_bits(), (-90.0_f32).to_bits());
        assert_eq!(reader.read_bits(3).expect("Tail bits"), 5);
        assert_eq!(reader.read_u8().expect("Tail byte"), 0x6e);
    }
}
