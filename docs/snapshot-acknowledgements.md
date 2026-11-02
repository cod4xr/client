# Original-owner snapshot ACK preparation

`ServerPayloadDecoder::prepare_snapshot_ack` validates a complete snapshot-only
ordinary payload while keeping its original outgoing command owner exclusively
borrowed. `SnapshotDecodeStorage` supplies private transform/decompression buffers;
`SnapshotPayloadRequest` supplies explicit world/reference context, inspection
budget and prediction choice. The packet and sequence still come from the decoder's
original established peer. No network I/O, clock, command execution or auth path is
introduced.

The existing decoder and the new preparation share one header inspector, preserving
the ordinary sequence/input/header/empty-body checks. The plaintext ACK is checked
against the original command owner **before** decoding or snapshot output writes.
`PreparedClientAcknowledgement` holds that exact owner and its checked key position.
The decoder then uses an immutable view through this exclusive lease, retaining
the same canonical compatibility key while transforming/decompressing input.

Actual payload profile/sequence binding, complete NOP/snapshot/EOF framing, sparse
source/storage limits and explicit prediction resolution all precede cooked-output
publication. Any preparation error preserves both complete final output buffers
and the original ACK/key history. Decode workspaces may contain untrusted private
intermediate data, following the existing decoder contract. No fallible operation
follows successful cooked-data construction inside preparation.

Successful preparation returns `PreparedSnapshotAcknowledgement`. The host borrows
its `plan` to preflight session/content/history resources. Wire data, decode buffers,
prediction sample and prior references may already be released; only cooked output
storage and the exclusive original command owner remain borrowed. Dropping this
successful preparation cancels **ACK release**; the successfully cooked buffer data
remains valid. It does not roll back already-successful cooked-data validation or
apply global snapshot history, content or simulation state.

After all fallible host preflight, consuming `commit` copies the prechecked canonical
key and advances the ACK without allocation, I/O or another fallible check. The key
is copied before its pending ring slot becomes reusable, and the cooked payload plan
is returned. The original owner cannot be replaced or mutated while prepared. A
repeated ACK performs no extra key copy/release. Existing
`LegacyClientCommands::commit_acknowledgement` delegates to the same prepared
implementation and preserves its previous validation/return contract.

Eight integration cases cover cancellation/retry, key copying before slot reuse,
zero/repeated/unsent/regressing ACKs, both profiles through actual shared decoding,
original-owner commit after inspection, source/workspace/sample lifetimes, explicit
native fallback and private Debug. Shared-header, workspace, late-service and
prediction failures preserve both outputs and the original key. Two compile-fail
examples protect the standalone and snapshot ACK owner leases. Existing payload
goldens, history cases and snapshot suites remain in the full workspace checks.

This is a prepared outgoing-command ACK plus cooked snapshot carrier. Combined
command/identity/snapshot routing, a connection-wide host/session/history owner,
typed gameplay and ordinary peer captures remain open. The host must still provide
the actual current world/baseline association and complete its resource preflight.
