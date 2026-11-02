// SPDX-License-Identifier: AGPL-3.0-or-later
use super::*;

#[test]
fn dispatch_uses_the_raw_case_sensitive_first_byte_in_both_reviewed_profiles() {
    for bytes in [
        b"".as_slice(),
        b" w reason",
        b"\"w\" reason",
        b"W reason",
        b"L map dm",
        b"M",
        b"x",
    ] {
        assert_eq!(inspect(bytes), None);
    }
    for profile in [BaselineProtocol::Legacy6, BaselineProtocol::Cod4x21] {
        let mut output = [0; 128];
        let plan = plan(
            &[
                (1, b"load map dm"),
                (2, b"whatever reason"),
                (3, b"magic ignored"),
            ],
            7,
            profile,
            &mut output,
        );
        let mut bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
        let mut slots = [ServerCommandSlot::default()];
        let mut history =
            ServerCommandHistory::new(profile, &mut bytes, &mut slots).expect("History");
        let committed = history
            .prepare(&plan)
            .expect("Prepared")
            .with_pre_game(BUDGET)
            .expect("Typed")
            .commit();
        let events: Vec<_> = committed
            .events()
            .map(|event| (event.sequence(), owned(event.action())))
            .collect();
        assert_eq!(
            events,
            [
                (1, Notice::Map(b"map".to_vec(), b"dm".to_vec())),
                (2, Notice::Disconnect(Some(b"reason".to_vec()), false)),
                (3, Notice::Restart)
            ]
        );
    }
}

#[test]
fn disconnect_distinguishes_missing_empty_and_punkbuster_metadata_without_authentication() {
    type DropCase<'a> = (&'a [u8], Option<&'a [u8]>, bool);
    let cases: &[DropCase<'_>] = &[
        (b"w", None, false),
        (b"w // reason", None, false),
        (b"w \"\"", Some(b""), false),
        (b"w \"Private Reason\"", Some(b"Private Reason"), false),
        (b"w reason PB", Some(b"reason"), true),
        (b"w reason pB ignored", Some(b"reason"), true),
        (b"w reason \"PB\"", Some(b"reason"), true),
        (b"w reason PBx", Some(b"reason"), false),
        (b"w PB", Some(b"PB"), false),
        (b"w reason \"PB \"", Some(b"reason"), false),
        (b"w \"\" PB", Some(b""), true),
    ];
    for &(bytes, reason, punkbuster) in cases {
        assert_eq!(
            inspect(bytes),
            Some(Notice::Disconnect(reason.map(<[u8]>::to_vec), punkbuster))
        );
    }
}

#[test]
fn map_fields_preserve_empty_raw_and_normalized_tokens_without_path_authority() {
    for (bytes, map, gametype) in [
        (b"l".as_slice(), b"".as_slice(), b"".as_slice()),
        (b"l mp_map", b"mp_map", b""),
        (b"l \"\" \"\"", b"", b""),
        (b"l \"map with spaces\" dm", b"map with spaces", b"dm"),
        (
            b"l ../Private%Map\xff gametype%",
            b"../Private.Map\xff",
            b"gametype.",
        ),
        (b"l map dm ; quit", b"map", b"dm"),
    ] {
        assert_eq!(
            inspect(bytes),
            Some(Notice::Map(map.to_vec(), gametype.to_vec()))
        );
    }
}

#[test]
fn full_fields_survive_explicit_native_copy_boundaries_without_silent_truncation() {
    for (prefix, length, limit) in [
        (b'l', 255, 255),
        (b'l', 256, 255),
        (b'w', 1023, 1023),
        (b'w', 1024, 1023),
    ] {
        let mut command = vec![prefix, b' '];
        command.extend(std::iter::repeat_n(b'x', length));
        command.extend_from_slice(b" PB");
        let mut output = vec![0; 8191];
        let plan = plan(&[(1, &command)], 7, BaselineProtocol::Cod4x21, &mut output);
        let mut bytes = vec![0; SERVER_COMMAND_SLOT_BYTES];
        let mut slots = [ServerCommandSlot::default()];
        let mut history =
            ServerCommandHistory::new(BaselineProtocol::Cod4x21, &mut bytes, &mut slots)
                .expect("History");
        let committed = history
            .prepare(&plan)
            .expect("Commands")
            .with_pre_game(BUDGET)
            .expect("Typed")
            .commit();
        let action = committed.events().next().expect("Event").action();
        let text = match action {
            PreGameAction::Disconnect { reason, punkbuster } => {
                assert!(punkbuster);
                reason.expect("Reason")
            }
            PreGameAction::LoadMap { map, gametype } => {
                assert_eq!(gametype.as_bytes(), b"PB");
                map
            }
            PreGameAction::FastRestart => unreachable!("Fixture dispatch"),
        };
        assert_eq!(text.as_bytes().len(), length);
        assert_eq!(text.legacy_bytes().len(), limit.min(length));
        assert_eq!(text.legacy_was_truncated(), length > limit);
    }
}

#[test]
fn fast_restart_is_an_inert_notice_even_when_followed_by_unparsed_text() {
    for bytes in [
        b"m".as_slice(),
        b"m \"unterminated",
        b"m /* ignored",
        b"m;quit",
        b"m\xff",
    ] {
        assert_eq!(inspect(bytes), Some(Notice::Restart));
    }
}
