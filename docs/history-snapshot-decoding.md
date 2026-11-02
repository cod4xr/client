# Snapshot reference selection from the actual packet

`ServerPayloadDecoder::prepare_mixed_snapshot_history_ack` selects a prior from
`SnapshotHistory` using the current packet's actual native header. Callers supply
`MixedSnapshotHistoryRequest`: current coordinate context, host message/world label,
initial baselines, sparse/service budgets, explicit prediction and the retained
history owner. They supply no separate guessed previous snapshot or delta distance.
The existing caller-selected-reference adapter remains available.

The exclusive original outgoing ACK/key lease predates decoding. Shared transform
and decompression run once. A private prefix inspection uses the same incremental
control scanner as complete mixed parsing, with the original text/identity scratch,
destination capacities, service limits and protocol rules. Actual payload profile
and message binding checks precede that work. The scanner locates the snapshot and
reads its six-byte header; it writes no final data and grants no EOF, history or ACK
admission. No public partial-packet token or generic callback framework is added.

History then checks current epoch/profile/message order and selects the exact
positive message sequence from the wire distance. Full snapshots select no prior,
even when history is populated. Missing/evicted entries never select the newest,
older or zero reference. These failures use the fixed `History` error category.
Malformed prefix/header failures retain the existing payload error categories.

Complete mixed parsing repeats bounded prefix validation over the same immutable
decoded body, checks the actual header/reference again and validates every state,
tail control, EOF, output capacity and prediction choice before any final write.
Header selection alone cannot publish a partial snapshot. Every failure preserves
all four final buffers, outgoing ACK/key state and retained snapshots; private
decode/control scratch remains unspecified.

Successful results retain neither packet/workspace nor prior-history borrows.
The host can immediately preflight mutable snapshot retention alongside command,
identity and other resources, commit infallible children, then consume the original
outgoing ACK holder. Dropping a successful holder cancels ACK release; valid cooked
output remains separate from host-history/engine admission.

Seven core cases cover both profiles through actual compression/obfuscation,
automatic exact references, full snapshots with existing history, cancellation and
retry, source lifetimes, missing/evicted/stale/wrong-world/profile references,
checked distance arithmetic, binding/prefix error order, private Debug, independent
prefix/tail budgets and late wire/prediction failure with whole-output preservation.

The real protocol-21 UDP case receives a malformed message, then a full snapshot at
message eight and a delta at eleven referencing eight by distance three. Receive
callbacks infer the prior from the packet and return prepared results after private
scratch expires. Host resource rejection/retry precedes joint command/identity/
snapshot commitment and outgoing ACK release. One history slot is reused only after
the delta has decoded. Duplicate controls in the delta preserve the accepted key
and identity sequence; the independent responder verifies both subsequent actual
control packets, outgoing key reuse and all message/command/identity ACK words.

This supports the reviewed command/NOP/protocol-21-identity/one-snapshot carrier.
Wider service routing, connection/gamestate/content lifecycle, prediction production,
typed simulation, rendering and native peer sessions remain open. The CLI is still
a discovery/challenge/content inspector.
