# Retained prediction in complete snapshot admission

`SnapshotPrediction::History` explicitly selects an original prediction owner's
read-only `PredictionHistorySource`. Its private fields retain the world/profile
and borrowed lookup cells together. The source cannot be assembled from unrelated
cells or supplied labels, and borrowing it prevents owner reset/replacement until
its final use. A compile-fail example protects that contract.

`PreparedSnapshot::commit_from_history` checks the actual prepared snapshot's
world/profile against this source, then resolves its decoded prediction request.
The native player command-time word selects the exact sample, including a time
inherited from an exact delta reference. Header server time is not substituted.
Transmitted-origin mode needs no sample but still checks source world/profile.

No caller-supplied lookup time, nearest-time fallback, separate partial player
parser or public intermediate authority is introduced. Missing samples return a
fixed-category `SnapshotError::PredictionHistory` before output writes or reader
publication. Native prior fallback remains an explicit, independent policy.

## Complete carriers and original ownership

The snapshot-only service transaction exposes the same `commit_from_history`
operation after complete EOF validation. Actual decoded payloads and mixed service
carriers dispatch the explicit history policy through these existing transactions.
Mixed commands, identities, player state, sparse streams and tail/EOF validation
finish before prediction lookup and all four final output writes.

The history-assisted ACK path continues to hold the original outgoing command key,
transform/decompress once and select the exact prior snapshot from its actual
header. It then uses the same complete carrier parser and prediction dispatch.
No additional header/prefix scan is needed for prediction. Host command, identity
and snapshot-retention preflight still precede final outgoing ACK commitment.

The cooked result owns prediction words and retains neither the cache/source nor
decode workspace. Later replay replacement does not alter already-decoded state.
Parsing never inserts, replaces, removes or retires prediction samples or inputs.

## Evidence and remaining work

Six integration cases cover both profiles, command times zero/one/high-bit/max,
actual and inherited delta times, nearby-sample rejection, wrong world/profile,
complete reader/output rollback, transmitted mode with an empty bound cache,
snapshot-only EOF precedence, actual payload dispatch, original ACK cancellation/
retry, all-four-output preservation and cooked-result lifetimes. Shared synthetic
ACK fixtures remain registered once in the existing core harness.

The existing protocol-21 UDP snapshot case now requests prediction for its full
and delta snapshots. A malformed tail is rejected while the prediction cache is
empty. After the valid full snapshot is decoded, explicit replay updates that exact
sample; the next delta receives the corrected words while the earlier cooked state
stays intact. Incoming command/identity/snapshot resource preflight and original
socket/key/ACK checks remain active.

This is synthetic transport and transaction evidence. Prediction production,
collision/physics, replay scheduling, authoritative input reconciliation, live
session lifecycle, rendering and ordinary native peer sessions remain open.
