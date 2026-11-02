# Complete native snapshot wire transactions

`snapshots::SnapshotPlan::prepare` validates the complete native snapshot body after
service opcode six: header, live player state, entity substream and client-state
substream. Both explicit baseline profiles share this order. There is no area mask
or extra wire trailer. Following service bytes and cached bits remain available.
This produces a complete wire projection, before host history, engine or enclosing
message/ACK admission.

`SnapshotInput` supplies explicit coordinate context, containing-message metadata,
optional cooked prior, initial baselines and independent entity/client budgets.
`SnapshotMessage::new` checks a positive ordinary message sequence through the
shared packet codec and preserves the caller's full server-command sequence word.
The command association is host metadata; it is not an extra snapshot wire field.
The caller owns a 64-bit world epoch and must change it across connection, map or
gamestate replacement. Labels do not authenticate a peer or prove actual host
history/command admission.

Baseline epoch/profile bindings and every source's order/namespace/inspection
budget are checked before wire reads, including unmentioned entries. The zero
header distance requires no prior. A nonzero distance requires the exact positive
containing-message sequence minus distance, matching profile and world epoch.
Distance measures server **messages**, rather than received snapshots. Missing,
mismatched or underflowing references never silently become zero state. The native
reader can treat nonpositive computed references as full snapshots; this owner
rejects such nonzero selectors instead of making an implicit substitution.

Only `SnapshotPlan::as_source` constructs `SnapshotSource`. The view binds player,
entity and client state to one cooked plan and keeps its backing storage immutable;
callers cannot assemble a source from unrelated component parts. New entities can
use explicitly labeled initial baselines; new client states use zero references.
Same-profile coordinate-center replacement is explicit and does not relabel prior
semantic words. The host remains responsible for supplying the actual current
gamestate baseline/epoch association and for retaining desired cooked plans.

Preparation decodes the complete player state privately and runs the existing
entity/client sparse walkers with closed no-op sinks. The original reader and
**both entire final output buffers**, including unused tails, remain unchanged.
The returned `PreparedSnapshot` exclusively borrows reader/output storage and
retains immutable replay sources. It exposes header/request/counter metadata but
no decoded candidate state. Dropping it cancels the transaction.

Commitment first resolves transmitted mode, an exact prediction sample, or explicit
native prior fallback using the same private pending-player implementation as the
standalone player-state transaction. Any prediction failure precedes all output
writes. Closed replay then publishes both sparse outputs from the exact validated
bytes, cursor, sources, context, budgets and capacities, without caller callbacks,
I/O or mutation between validation and replay. Only then does the original reader
advance and the cooked plan become available. No source/wire/prior borrow survives
in the final result. Player words are owned; sparse results borrow final storage.

No global parse ring, peer-selected allocation or fixed history size is added.
Host-funded storage can retain more than the native 32 snapshots, and wire distance
255 can resolve to a retained exact reference. Native field/index widths remain
wire constraints; wider engine data and future negotiated extensions stay separate.
Body-byte funding remains the `BodyReader` owner's responsibility. Sparse source,
record, retention and capacity limits remain independent.

Twenty-two integration cases cover independent physical empty bodies, both
profiles, full/delta merges, cached-bit continuity, following services, initial
baseline references, complete output-tail preservation, every tested truncation,
late client errors, independent budgets/capacity, binding failures, prediction
selection/retry/fallback, drop cancellation, owned lifetimes and private formatting.
Forty retained cooked plans and distance 255 demonstrate caller-controlled
retention. Three compile-fail examples enforce exclusive reader/output leases and
immutable cooked-reference backing. Constituent modeled/field suites remain intact.
[Audit metadata](snapshot-body-reader-evidence.json) records native ordering and
the point where wire reads end.

Next integration gates are enclosing multi-service message validation, joint
server-command/identity/snapshot/ACK publication, a host session/history owner,
prediction production, typed content/simulation and ordinary peer captures. The
command-line client remains a probe/content inspector, without playable joins.
