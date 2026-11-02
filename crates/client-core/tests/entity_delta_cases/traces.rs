// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn independent_player_field_traces_preserve_prior_words_across_new_record_streams() {
    for seed in 0..32_u32 {
        let mut state = reference(17);
        let mut random = seed ^ 0x7654_3210;
        let mut stream = decoder();
        for step in 0..32 {
            let next = random.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            random = next;
            let coordinates = [next, next.rotate_left(13), next.rotate_right(7)];
            let signed_byte = next.to_le_bytes()[1];
            let signed = i32::from(i8::from_le_bytes([signed_byte])).cast_unsigned();
            let angle = u16::from_le_bytes([next.to_le_bytes()[0], next.to_le_bytes()[3]]);
            let angle_bits = (f32::from(angle.cast_signed()) * (360.0 / 65536.0)).to_bits();
            let count = if step % 5 == 0 {
                None
            } else if step % 3 == 0 {
                Some(4)
            } else {
                Some(6)
            };
            let bytes = fixture(17, count, |writer| {
                if count.is_none() {
                    return;
                }
                kind(writer, 1, 1);
                writer
                    .write_i32(coordinates[0].cast_signed())
                    .expect("Raw X");
                writer
                    .write_i32(coordinates[1].cast_signed())
                    .expect("Raw Y");
                signed_field(writer, state.word(W::Union1), signed, signed_byte);
                if count == Some(6) {
                    angle_field(writer, angle);
                    writer.write_bits(1, 1).expect("Changed Z");
                    writer
                        .write_i32(coordinates[2].cast_signed())
                        .expect("Raw Z");
                }
            });
            // Each host snapshot starts its own ordered entity stream. This loop
            // exercises only individual record semantics, not snapshot framing.
            stream.reset();
            let mut reader = BodyReader::try_new(&bytes, bytes.len()).expect("Next stream");
            let updated = present(
                stream
                    .prepare(&mut reader)
                    .expect("Number lookup")
                    .decode(Some(&state), next)
                    .expect("Complete record"),
            );
            assert_eq!(updated.number(), 17);
            assert_eq!(updated.word(W::Kind), 1);
            assert_eq!(
                updated.word(W::PositionX),
                if count.is_some() {
                    coordinates[0]
                } else {
                    state.word(W::PositionX)
                }
            );
            assert_eq!(
                updated.word(W::PositionY),
                if count.is_some() {
                    coordinates[1]
                } else {
                    state.word(W::PositionY)
                }
            );
            assert_eq!(
                updated.word(W::Union1),
                if count.is_some() {
                    signed
                } else {
                    state.word(W::Union1)
                }
            );
            assert_eq!(
                updated.word(W::AngleY),
                if count == Some(6) {
                    angle_bits
                } else {
                    state.word(W::AngleY)
                }
            );
            assert_eq!(
                updated.word(W::PositionZ),
                if count == Some(6) {
                    coordinates[2]
                } else {
                    state.word(W::PositionZ)
                }
            );
            assert_eq!(updated.word(W::Part3), state.word(W::Part3));
            state = updated;
        }
    }
}

fn signed_field(writer: &mut MessageWriter<'_>, prior: u32, signed: u32, signed_byte: u8) {
    writer
        .write_bits(u32::from(prior != signed), 1)
        .expect("Signed outer flag");
    if prior != signed {
        writer
            .write_bits(u32::from(signed != 0), 1)
            .expect("Signed nonzero");
        if signed != 0 {
            writer
                .write_u8(prior.to_le_bytes()[0] ^ signed_byte)
                .expect("Signed XOR byte");
        }
    }
}

fn angle_field(writer: &mut MessageWriter<'_>, angle: u16) {
    writer.write_bits(1, 1).expect("Changed angle");
    writer
        .write_bits(u32::from(angle != 0), 1)
        .expect("Angle nonzero");
    if angle != 0 {
        writer
            .write_i16(angle.cast_signed())
            .expect("Absolute angle");
    }
}
