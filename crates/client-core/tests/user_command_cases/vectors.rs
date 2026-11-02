// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    AbsoluteUserCommandPlan, MAX_ABSOLUTE_COMMAND_BYTES, MessageReader, MovementAxis,
    UserCommandMelee, baseline, decode, user_command, zero,
};

#[test]
fn independent_full_zero_vector_overwrites_common_fields_and_preserves_disabled_inert_fields() {
    // Absolute time; keyed full-form bits; forced common-field writes. Byte fields
    // interrupt the physical bit bytes, including the twenty-bit button payload.
    let expected = [0x76, 0, 0, 0, 0, 0, 0, 0, 0, 0x18, 0, 0, 0, 0, 2, 2, 0];
    let plan = AbsoluteUserCommandPlan::new(0, zero()).expect("Zero full command");
    let mut output = [0xa5; MAX_ABSOLUTE_COMMAND_BYTES + 4];
    assert_eq!(
        plan.encode(&mut output).expect("Exact vector"),
        expected.len()
    );
    assert_eq!(output[..expected.len()], expected);
    assert_eq!(output[expected.len()..], [0xa5; 9]);
    let old = baseline();
    let mut reader = MessageReader::try_new(&expected).expect("Independent physical bytes");
    let actual = user_command::read_delta(&mut reader, 0, &old).expect("Shared decoder");
    assert_eq!(
        (actual.server_time, actual.buttons, actual.angles),
        (0, 0, [0; 3])
    );
    assert_eq!(
        (
            actual.weapon,
            actual.offhand,
            actual.forward_move,
            actual.right_move
        ),
        (0, 0, 0, 0)
    );
    assert_eq!(
        actual.melee_charge_yaw.to_bits(),
        old.melee_charge_yaw.to_bits()
    );
    assert_eq!(actual.melee_charge_distance, old.melee_charge_distance);
    assert_eq!(actual.selected_location, old.selected_location);
}
#[test]
fn every_key_bit_and_both_time_signs_reconstruct_all_common_native_fields() {
    let old = baseline();
    for key in [0, u32::MAX, 0xdead_beef]
        .into_iter()
        .chain((0..32).map(|bit| 1_u32 << bit))
    {
        for time in [0, -1, i32::MIN, i32::MAX, 0x1234_5678] {
            let mut input = zero();
            input.server_time = time;
            input.buttons = 0x1f_fff9;
            input.angles = [0x1234, 0xffff, 0x8000];
            input.weapon = 127;
            input.offhand = 126;
            input.selected_location = Some([0x92, 255]);
            let actual = decode(key, input, &old);
            assert_eq!(
                (actual.server_time, actual.buttons, actual.angles),
                (time, input.buttons, input.angles)
            );
            assert_eq!((actual.weapon, actual.offhand), (127, 126));
            assert_eq!(actual.selected_location, [0x92, 255]);
            assert_eq!(
                actual.melee_charge_yaw.to_bits(),
                old.melee_charge_yaw.to_bits()
            );
        }
    }
}
#[test]
fn all_canonical_axis_pairs_and_conditional_action_forms_match_the_shared_receiver() {
    let old = baseline();
    let axes = [
        (MovementAxis::Negative, -127),
        (MovementAxis::Stationary, 0),
        (MovementAxis::Positive, 127),
    ];
    for (forward, expected_forward) in axes {
        for (right, expected_right) in axes {
            for (yaw, expected_yaw) in [
                (0, 0.0_f32),
                (0x4000, 90.0),
                (0x8000, -180.0),
                (0xc000, -90.0),
            ] {
                for mode in 0..4 {
                    let mut input = zero();
                    input.server_time = -999;
                    input.movement = [forward, right];
                    if mode & 1 != 0 {
                        input.buttons |= 0x10000;
                        input.selected_location = Some([0xff, 0x92]);
                    }
                    if mode & 2 != 0 {
                        input.buttons |= 4;
                        input.melee = Some(UserCommandMelee { yaw, distance: 255 });
                    }
                    let actual = decode(0xa5b6_c7d8, input, &old);
                    assert_eq!(
                        (actual.forward_move, actual.right_move),
                        (expected_forward, expected_right)
                    );
                    assert_eq!(actual.buttons, input.buttons);
                    assert_eq!(
                        actual.selected_location,
                        input.selected_location.unwrap_or(old.selected_location)
                    );
                    let (expected_yaw, distance) = if mode & 2 != 0 {
                        (expected_yaw, 255)
                    } else {
                        (old.melee_charge_yaw, old.melee_charge_distance)
                    };
                    assert_eq!(actual.melee_charge_yaw.to_bits(), expected_yaw.to_bits());
                    assert_eq!(actual.melee_charge_distance, distance);
                }
            }
        }
    }
}
