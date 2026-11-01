// SPDX-License-Identifier: AGPL-3.0-or-later
use cod4xr_reliable::send::{MAX_SEND_WINDOW_FRAGMENTS, SendError, SendWindow};

use crate::{packet, sent, storage};

#[test]
fn maximum_explicit_span_admits_deduplicated_selective_metadata_then_frees_only_a_sent_prefix() {
    let span = MAX_SEND_WINDOW_FRAGMENTS;
    let (mut bytes, mut slots) = storage(usize::from(span));
    let mut window = SendWindow::new(&mut bytes, &mut slots, span).expect("Largest host span");
    for sequence in 0..u32::from(span) {
        let _ = window
            .enqueue(&[sequence.to_le_bytes()[0]])
            .expect("One original fragment");
        sent(&mut window, sequence);
    }
    let claims = [(0, span); 3]; // Deliberate overlap must not triple-count or free storage.
    let summary = window
        .prepare_acknowledgement(packet(None, 0, &claims))
        .expect("Largest valid ranges")
        .commit();
    assert_eq!(summary.newly_selective, usize::from(span));
    assert_eq!(
        (
            summary.released_fragments,
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (0, usize::from(span), usize::from(span))
    );
    assert_eq!(
        window
            .prepare_transmission(u32::from(span) - 1)
            .expect_err("SACK suppresses retry"),
        SendError::Selective
    );
    let summary = window
        .prepare_acknowledgement(packet(None, u32::from(span), &[]))
        .expect("Complete sent prefix")
        .commit();
    assert_eq!(
        (summary.released_fragments, summary.released_bytes),
        (usize::from(span), usize::from(span))
    );
    assert_eq!(
        (
            window.acknowledge(),
            window.queued_fragments(),
            window.buffered_bytes()
        ),
        (u32::from(span), 0, 0)
    );
}
