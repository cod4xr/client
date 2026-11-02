// SPDX-License-Identifier: AGPL-3.0-or-later
use super::{
    BodyReader, LivePlayerState, PredictionProvenance, contexts, decode, empty, encoded, mixed,
    word,
};
use cod4xr_client_core::player_hud::HudWord;

#[test]
fn physical_empty_state_vector_and_following_cached_bits_and_bytes_match_both_profiles() {
    for context in contexts() {
        let bytes = [1, 0, 0xc0, 0x5a];
        let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Body");
        let prepared = LivePlayerState::prepare(&mut reader, context, 0, None).expect("State");
        assert_eq!(prepared.consumed_bytes(), 3);
        assert_eq!(prepared.profile(), context.profile());
        assert_eq!(prepared.prediction_request(), None);
        let state = prepared.commit_transmitted().expect("Transmitted");
        assert_eq!(reader.byte_position(), 3);
        assert_eq!(reader.remaining_cached_bits(), 2);
        assert_eq!(reader.read_u8().expect("Following byte"), 0x5a);
        assert_eq!(reader.read_bits(2).expect("Following cached bits"), 3);
        assert_eq!(state.profile(), context.profile());
        assert_eq!(state.context(), context);
        assert_eq!(state.prediction(), PredictionProvenance::Transmitted);
        assert_eq!(state.scalars().word(word::session::COMMAND_TIME), 0);
        assert_eq!(state.stats().values(), &[0; 5]);
        assert_eq!(state.ammunition().stored_values(), &[0; 64]);
        assert_eq!(state.ammunition().clip_values(), &[0; 128]);
        assert!(
            state
                .objectives()
                .objectives()
                .all(|slot| slot.status() == 0)
        );
        assert_eq!(state.models().values(), &[0; 128]);
    }
}
#[test]
fn writer_empty_vector_has_exact_native_twenty_two_bit_shape() {
    assert_eq!(encoded(|writer| empty(writer, true)), [1, 0, 0]);
    assert_eq!(encoded(|writer| empty(writer, false)), [0, 0, 0]);
}
#[test]
fn all_sections_compose_in_native_order_and_model_bytes_remain_absolute() {
    for context in contexts() {
        let bytes = encoded(|writer| mixed(writer, true, 0, 0xffff_fffe));
        let state = decode(context, &bytes, None);
        assert_eq!(
            state.scalars().word(word::session::COMMAND_TIME),
            0xffff_fffe
        );
        assert_eq!(
            state.scalars().word(word::motion::ORIGIN_X),
            123.5_f32.to_bits()
        );
        assert_eq!(
            state.scalars().word(word::view::ANGLE_Y),
            (-180.0_f32).to_bits()
        );
        assert_eq!(state.scalars().word(word::weapons::TIME), 0xffff_fff9);
        assert_eq!(state.stats().values(), &[123, 0, 0, 0, 0]);
        assert_eq!(state.ammunition().stored(0), Some(7));
        assert_eq!(state.ammunition().clip_values(), &[0; 128]);
        assert!(
            state
                .objectives()
                .objectives()
                .all(|slot| slot.status() == 1)
        );
        assert_eq!(state.hud().archival()[0].word(HudWord::Color), 0xff00_0000);
        assert_eq!(state.hud().current()[0].word(HudWord::Color), 0);
        for (index, &value) in state.models().values().iter().enumerate() {
            assert_eq!(value, u8::try_from(index).expect("Index") ^ 0xa5);
        }
        let next = decode(context, &[1, 0, 0], Some(&state));
        assert_eq!(next, state);
    }
}
