# Complete live player-state transactions

`player_state::LivePlayerState::prepare` composes the complete native live wire
projection: scalar prefix, stats, stored/clip ammunition, objectives, archival and
current HUD, then weapon-model bytes. Both reviewed baseline profiles share this
ordering. The caller supplies an explicit `EntityDecodeContext`, message time and
optional immutable prior state. A missing prior supplies zero references; a profile
mismatch rejects before reading. A new same-profile legacy center is explicit.

The prepared transaction owns every candidate section and a copy of the prior's
eleven prediction words. It exclusively borrows the original `BodyReader`, but
retains no borrow of the prior. No decoded `LivePlayerState` is exposed yet.
Dropping the transaction or failing any section or commit choice preserves both
original reader cursors, including cached bits. Successful commitment publishes
the complete candidate reader and returns an owned result that outlives input,
prior state and prediction samples. There are no peer-selected allocations.

The native origin-mode bit determines the required commit choice:

| Wire mode | Commit | Effect |
| --- | --- | --- |
| Transmitted | `commit_transmitted` | Keeps the decoded scalar projection, including omitted prior words |
| Unsent | `commit_with_prediction` | Requires exact profile/command-time matching, then replaces eleven words |
| Unsent | `commit_native_prior_fallback` | Explicitly replaces those words with prior values, or zeros without a prior |

The eleven words are origin XYZ, velocity XYZ, view angles XYZ, bob cycle and
movement direction. New command time, other scalar words and every array remain
intact. Fallback replaces even different origin/angle words carried in the current
prefix. `PredictionProvenance` distinguishes these three choices. Raw sample words
remain inert: NaNs, signed integers and resource/status bits require later typed
validation. Debug/error formatting excludes command times, vectors and inventory.

`prediction_request` returns the exact decoded command time and selected profile.
These are lookup metadata, not a connection/map epoch proof. The host must scope
its prediction queue to the current session and decide whether to supply a sample
or explicitly select fallback. The core has no prediction ring, sockets, logging,
clock or global session state. It does not silently accept an older sample.

The installed native reader calls prediction lookup before arrays; applying the
same eleven disjoint scalar replacements after privately decoding arrays preserves
the final projection while permitting whole-state rollback. Native lookup scans
its ring for an at-or-before command time and diagnoses a nonexact match before
using it. This API deliberately requires exact sample matching; native ring search
and older-sample policy are not implemented here. Caller-selected prior fallback
implements the native no-sample replacement, rather than silently returning a
partly reconciled state. [Audit metadata](live-player-state-reader-evidence.json)
binds those branches and the section order to the reviewed image hash.

Fifteen integration cases cover independent physical empty-state vectors, all
sections together, following byte/cached-bit operations, both profiles, wrapping
command times, signed/raw samples, failed-choice retry, explicit fallback, every
truncated mixed body, late current-HUD failure, profile rejection, drop cancellation,
owned lifetimes, context replacement and privacy. Two compile-fail examples enforce
the reader lease and consuming commitment. Constituent codec tests separately
cover every field, complete arrays and modeled update traces.

This is a complete live **wire projection**, not the entire native C structure or
typed simulation. Wider engine storage remains independent of native wire widths.
Whole snapshot framing/history, session/reference epochs, complete message/ACK
admission, prediction production, input replay and playable joins remain open.
