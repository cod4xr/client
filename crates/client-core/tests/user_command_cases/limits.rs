// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    AbsoluteUserCommandPlan, MAX_ABSOLUTE_COMMAND_BYTES, UserCommandEncodeError, UserCommandMelee,
    zero,
};

#[test]
fn all_native_width_and_conditional_presence_failures_are_rejected_before_encoding() {
    for mode in 0..9 {
        let mut input = zero();
        let expected = match mode {
            0 => {
                input.buttons = 0x0020_0000;
                UserCommandEncodeError::Buttons
            }
            1 => {
                input.buttons = u32::MAX;
                UserCommandEncodeError::Buttons
            }
            2 => {
                input.weapon = 128;
                UserCommandEncodeError::Weapon
            }
            3 => {
                input.offhand = 255;
                UserCommandEncodeError::Offhand
            }
            4 => {
                input.buttons = 0x10000;
                UserCommandEncodeError::SelectedLocation
            }
            5 => {
                input.selected_location = Some([0; 2]);
                UserCommandEncodeError::SelectedLocation
            }
            6 => {
                input.buttons = 4;
                UserCommandEncodeError::Melee
            }
            7 => {
                input.melee = Some(UserCommandMelee {
                    yaw: 0,
                    distance: 0,
                });
                UserCommandEncodeError::Melee
            }
            _ => {
                input.weapon = 255;
                UserCommandEncodeError::Weapon
            }
        };
        assert_eq!(
            AbsoluteUserCommandPlan::new(0xdead_beef, input).expect_err("Native input validation"),
            expected
        );
    }
}
#[test]
fn every_output_capacity_and_all_action_lengths_preserve_the_entire_final_buffer_on_failure() {
    for mode in 0..4 {
        let mut input = zero();
        if mode & 1 != 0 {
            input.buttons |= 0x10000;
            input.selected_location = Some([255; 2]);
        }
        if mode & 2 != 0 {
            input.buttons |= 4;
            input.melee = Some(UserCommandMelee {
                yaw: 0xffff,
                distance: 255,
            });
        }
        let plan = AbsoluteUserCommandPlan::new(u32::MAX, input).expect("Bounded plan");
        let length = plan.encoded_len();
        assert!(length <= MAX_ABSOLUTE_COMMAND_BYTES);
        for capacity in 0..length {
            let mut output = [0xa5; MAX_ABSOLUTE_COMMAND_BYTES + 4];
            assert_eq!(
                plan.encode(&mut output[..capacity]),
                Err(UserCommandEncodeError::OutputCapacity)
            );
            assert_eq!(output, [0xa5; MAX_ABSOLUTE_COMMAND_BYTES + 4]);
        }
        let mut output = [0xa5; MAX_ABSOLUTE_COMMAND_BYTES + 4];
        assert_eq!(
            plan.encode(&mut output[..length]).expect("Exact capacity"),
            length
        );
        assert!(output[length..].iter().all(|&byte| byte == 0xa5));
        if mode == 3 {
            assert_eq!(length, MAX_ABSOLUTE_COMMAND_BYTES);
        }
    }
}
#[test]
fn input_plan_melee_and_errors_keep_private_fields_and_keys_out_of_debug() {
    let mut input = zero();
    input.server_time = 1_987_654_321;
    input.weapon = 119;
    input.buttons = 4;
    input.melee = Some(UserCommandMelee {
        yaw: 0xdead,
        distance: 231,
    });
    let plan = AbsoluteUserCommandPlan::new(0xdead_beef, input).expect("Private command");
    let text = format!("{input:?} {plan:?} {:?}", input.melee);
    for private in ["1987654321", "3735928559", "57005", "119", "231"] {
        assert!(!text.contains(private));
    }
    assert!(!text.contains("server_time"));
}
