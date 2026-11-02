# Snapshot carriers bound to decoded ordinary payloads

`snapshots::services::SnapshotPayloadPlan::parse` binds the specialized snapshot
carrier to a `DecodedServerPayload`. The snapshot context's profile must equal the
original decoded profile, and its message association must equal the actual
decoded ordinary-channel sequence. Both checks precede service inspection and
output writes. The original decoder remains responsible for the correct peer,
compatibility cookie, retained command key and shared ordinary byte bound.

The adapter then validates the complete NOP/snapshot/EOF carrier and requires an
explicit `SnapshotPrediction` choice: transmitted mode, an exact sample, or native
prior fallback. There is no default or implicit fallback. This one-shot adapter is
for callers that already have a prediction choice; the reader-based prepared API
remains available when a host first needs to inspect the exact lookup request.
Unsupported mixed services remain rejected, without skipping their bodies.

All failures preserve both complete cooked-output buffers and outgoing command
history. A successful result owns its metadata/player words and borrows only final
output storage. It retains no source payload, decoder scratch, prediction sample,
prior or outgoing-key-history borrow. The host can retain cooked data independently
of the temporary decompressed body and later preflight its session/history resources.

`client_command_acknowledge` returns the original payload's already-checked ACK
number without applying it. Parsing never releases the retained outgoing key or
advances a connection's ACK. Apply it only to the same original connection's owner
after complete host resource preflight. This is cooked-data validation, not joint
history/engine/message admission. The shared 128-KiB ordinary profile is unchanged;
larger reliable carriers and future negotiated profiles remain separate.

Six cases use the existing shared compression/obfuscation fixture and actual
payload decoder, covering both profiles, a positive transmitted-command ACK,
profile/sequence mismatch before service work, late unsupported services, failed
prediction, exact sample/fallback choices, valid retry, owned lifetimes and private
Debug output. They demonstrate that ACK/key state stays unchanged during successful
and failed decoding, then permit a separate explicit same-owner ACK commit. Existing
independent payload codec goldens, network tests and snapshot framing suites remain.

Complete command/identity/snapshot service routing, connection-owned joint ACK and
history publication, prediction production, simulation and playable sessions remain
open. The standalone project has no mod-specific behavior or required auth path.
