# Original-owner ACK preparation for mixed snapshots

`ServerPayloadDecoder::prepare_mixed_snapshot_ack` combines the existing original
command-key lease with complete mixed control/snapshot decoding. It reuses the
shared header inspector, retained-key validation, compatibility transform and
decompression, then the mixed carrier's actual binding, full EOF, independent
buffer/budget checks and explicit prediction resolution. It adds no wire codec,
peer callback, allocation policy, auth path or command execution.

`MixedSnapshotRequest` supplies current world/reference context, total service and
identity inspection limits, and a mandatory prediction choice. Private decode
buffers use `SnapshotDecodeStorage`; private control workspaces and four final
buffers use `MixedSnapshotStorage`. Header/ACK, decode, control/snapshot/EOF and
prediction failures preserve all four complete final buffers and the original
outgoing ACK/key owner. Private scratch remains unspecified on failure.

The exclusive original outgoing-owner lease exists before decoding. A successful
`PreparedMixedSnapshotAcknowledgement` owns complete cooked controls/snapshot data,
borrowing only final output storage and that original owner. Packet bytes, decode
and control scratch, prediction samples and prior references may already be
released. Dropping a successful holder cancels ACK release; its already-valid
cooked data is not host-history or engine admission.

The host uses `plan` to preflight every incoming command, identity, session/content
and snapshot-history resource. Existing command preparations borrow the cooked
control plan, so after **all fallible preflight**, commit infallible child command
and identity preparations before consuming the outgoing ACK holder. Each commit
performs no I/O or fallible validation. The final consuming ACK commit copies the
checked original key before its pending slot becomes reusable and returns the
complete cooked mixed plan. No incoming owner is mutated by parsing itself.

A resource-budget rejection after successful wire parsing must drop all sibling
preparations and the ACK holder. This differs from wire failure: successful cooked
buffer data remains valid, while command/identity histories and outgoing ACK remain
unchanged. Identity sequence gaps retain the existing native ignore semantics;
this adapter does not reinterpret them as wire or admission errors.

Four integration cases cover both profiles, original-key cancellation/retry and
slot reuse, packet/workspace/control-scratch lifetimes, private Debug, command and
identity preflight followed by commit, actual identity resource-budget rejection,
and late wire/prediction failures preserving all four outputs and original key.
A compile-fail example prevents replacing the original owner while prepared.
Existing native framing, codecs and owner suites remain in the workspace checks.

The supported mixed carrier contains one snapshot, commands, NOPs, reviewed
protocol-21 identities and EOF. Wider service routing, legacy identity schema,
connection-wide gameplay/history ownership, typed session effects, simulation,
rendering and ordinary peer captures remain separate integration gates.
