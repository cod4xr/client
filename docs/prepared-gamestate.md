# Prepared gamestate frames

`ReliableConnection::prepare_game_state` binds whole-body decoding to the original
socket owner's ready frame and explicitly selected profile. It rejects terminal
framing failure, unsupported original profiles and missing frames before decoding.
It performs no I/O, advances no transport receipt clock and changes no byte ACK.

Whole-body validation still precedes any final caller writes. Success returns
`PreparedGameStateFrame`, which exclusively borrows the connection and owns the
checked plan over caller output. The host can inspect `plan()` for bounded resource
or session preflight. While that handle lives, ordinary sending, receiving, draining,
frame replacement and manual release cannot borrow the same connection mutably.

Dropping the handle cancels release and commit. The frame stays ready, and already
checked cooked output remains in caller storage; no session or engine state is
changed. `commit()` releases exactly that frame, records the explicit commit and
returns a plan borrowing only cooked output. The original frame and socket owner
may then be released without invalidating the decoded data. Commit contains no
fallible operation: the exclusive borrow preserves the ready-frame invariant.

Raw frame inspection/explicit discard remain available outside a preparation.
Malformed application bodies are not silently discarded or reclassified as
terminal transport framing errors. A host may inspect an error, explicitly discard
that frame and continue to another frame according to its application policy.

Saturating `ReliableUsage` counters distinguish decode attempts, decode failures
and committed gamestates. Retries after cancellation count work again; only explicit
commit counts admission. Empty/terminal preconditions charge no decode work. Failed
decodes include caller budget/capacity exhaustion, so they are not automatically
claims of peer misconduct. Existing released-frame counters also include raw host
discard, separately from typed commits. Debug formatting exposes plan/count metadata,
without checksum words, text or entity state. Libraries install no global logger.

Three real UDP tests cover empty-frame rejection without work, cancellation and
retry, exact-once commit/release, cooked data after socket/buffer release, malformed
body preservation and explicit discard followed by valid admission, terminal framing
preconditions, unchanged receipt time, private debug output and the new counters.
A compile-fail documentation test demonstrates that a live preparation prevents
manual replacement/release of its frame. Session epochs, native defaults/content,
engine updates and an ongoing playable loop remain open.
