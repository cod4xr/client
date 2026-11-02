# Caller-funded whole-snapshot history

`snapshots::history::SnapshotHistory` retains coherent cooked player/entity/client
state under an explicit host world epoch and baseline profile. It has no socket,
clock, allocator, native global ring, command execution or engine ownership.
Retention is a separate child resource preparation during whole-message admission.

The host supplies a nonempty slice of `SnapshotHistorySlot` values. Each slot has
independently sized entity/client backing slices and inline owned player metadata.
Snapshot count and sparse capacities are independent choices. Zero sparse capacity
is valid for empty snapshots. No array is allocated by a remote identifier, message
sequence or native ring constant. Each committed slot owns its complete state;
packet bytes, decode scratch, prediction samples, previous snapshots and source
output buffers may already have expired.

`prepare` checks epoch/profile, strictly newer containing-message order and both
capacities of the next insertion-order slot. A successful exclusive lease holds the
original history and immutable cooked snapshot. Drop cancels every history write.
After **all** sibling resources preflight, `commit` copies complete state and updates
metadata without parsing, allocation, callbacks, I/O or another fallible operation.
Only used prefixes are copied; all unused backing tails stay unchanged. A failed
preflight never evicts an existing snapshot or advances the insertion cursor.

Commit replaces the oldest insertion once all slots are occupied. Message gaps
consume no extra slots. Exact tags are binary-searched in logical circular order,
so lookup is logarithmic in host-selected capacity and physical wrap cannot alias
an unrelated message. Summary data reports the committed sequence, evicted sequence
and retained count. Debug/errors expose fixed categories and capacities, excluding
raw player/client/entity values, server time, flags and world labels.

`source` returns an exact retained `SnapshotSource`; component parts cannot be mixed.
`header` preserves the original opaque header. `reference` checks a separately parsed
header against the current epoch/profile/new-message binding: zero distance needs
no prior, while a nonzero distance must subtract to a positive exact retained message.
All distances through 255 are supported. Missing/evicted references return an error;
no older, zero or implicit full-snapshot fallback is selected. The complete snapshot
reader still validates that the actual wire header and selected prior agree.

Borrowed sources prevent eviction/reset until their decoding use ends. Returned
cooked plans keep no prior-history borrow, so the same history can immediately
preflight their retention. `reset` requires a strictly larger local epoch, invalidates
all metadata and can change profile; it leaves raw cells unchanged and permits a
new message sequence. Constructing a new owner also invalidates old metadata. The
host must supply unique world labels across reconstructed owners; these labels do
not authenticate peers or replace connection/map/gamestate lifecycle handling.

Eight integration cases cover both profiles, exact delta decoding after original
output lifetime, 40 retained snapshots, distance 255, occupied-slot capacity failure,
drop cancellation, unchanged tails, stale/world/profile rejection, checked arithmetic,
epoch/profile replacement and mixed-message sibling-owner cancellation. An independent
queue model runs 256 operations for each of five capacities in both profiles, with
sequence gaps, cancellation, circular eviction, lookups, resets and reconstructed
owners. Two compile-fail examples protect original-owner and source borrows. The
existing joint mixed ACK test now preflights and commits snapshot retention alongside
incoming commands and identities before outgoing key release.

This is deterministic storage and synthetic wire evidence. Connection-wide routing,
actual header discovery within broader mixed carriers, prediction production,
gamestate/content lifecycle, typed simulation, rendering and native peer sessions
remain separate work. It does not make the CLI playable or widen legacy wire IDs.
