# Complete snapshot entity substreams

`snapshot_entities::SnapshotEntityPlan` validates an entire native entity substream
and merges it with the prior snapshot before publishing caller-owned final cells.
The shared wire implementation now lives in `cod4xr_client_core::entities`, used by both
gamestate and snapshot code. Existing `gamestate::entity_baselines` and root
`entity_deltas` imports remain available as compatibility re-exports. The generated
field tables moved with the implementation and still reproduce from their locked
sources; numeric values and native layouts are unchanged.

`EntityDecodeContext` explicitly selects protocol 21 or protocol 6 with checked
`LegacyMapCenter`. Baseline and delta owners use the same context representation.
The parser starts a fresh checked entity-index stream, preserving the original
reader's existing cached bits. Reserved native number 1023 terminates this stream
before any removal/delta bits; it can arrive through the normal absolute or
increment encoding. Neither unused cached bits nor following bytes are consumed
as another stream. The containing native snapshot resets its index cache before
entities and again before its following client-state stream.

For each new wire number, unmentioned smaller prior entities copy unchanged.
Matching numbers decode against the exact prior state; numbers absent from the
prior snapshot decode against their initial baseline or zero if no baseline exists.
Removals omit that result, including harmless removal of a number absent from the
prior source. After the terminator, all remaining prior entities copy unchanged.
The resulting numbers remain strictly ordered. Individual value/field/type rules
come from the same [entity record decoder](entity-deltas.md), including explicit
legacy origin context and message-time bits.

`EntityStateSource` borrows caller states or a previously cooked plan's final cells.
It carries no snapshot sequence, session epoch or peer admission authority. Before
reading any stream bytes, parsing checks every input source's host budget, unique
order and native namespace, including unused baselines. The prior snapshot cannot
contain the terminator as an entity; an unused baseline with that number is inert.
Binary lookup finds new-number baselines without allocating by peer identifier.

`EntityStreamBudget` separately bounds prior-source inspection, baseline inspection,
wire records including terminator and merged output entities. Final storage has its
own capacity. An empty stream can retain many unchanged prior entities, so update
count never substitutes for output retention policy. Current native identifiers
remain wire limits; no fixed engine table or global parse-entity ring is allocated.

A private validation walker first proves all records, references, budgets and final
extents through the terminator and retained tail. The same walker then replays the
immutable input/sources/context into final cells. Only closed internal sinks run;
no external callbacks, I/O or source mutation can invalidate preflight. Success
commits both original reader cursors and returns a plan borrowing only final cells.
Every error leaves the entire final buffer, unused tail and original reader intact.
Private decoded candidates never modify prior/baseline states.

`entities()` borrows the merged ordered states. `as_source()` returns a cooked view
that outlives the parent plan, permitting the next parse without repacking states.
It retains only final cells, not input bytes or previous/baseline sources. A
compile-fail example prevents mutably republishing those same cells while they are
borrowed as input. Counts and profile metadata are available for host counters;
debug/errors contain no raw entity words or coordinates.

Fifteen integration tests cover independent physical terminators, maximum-number
increment termination, existing cached bits and following fields, sparse mixed
updates/removals, prior-versus-baseline precedence, zero fallback, legacy context,
all tested truncations, late invalid records, duplicate/order/source failures,
independent budgets/capacities, unchanged tails, source/cooked lifetimes and repeated
cooked-source reuse. A 128-trace ordered-map model checks sparse merge behavior.
Caller-funded tests retain 512 entities and six independent results containing
3,072 entities together, exceeding the native global 2,048-entry ring without a
replacement global limit. They are entity substream results, not complete snapshots.

[Audit metadata](snapshot-entity-reader-evidence.json) binds these rules to the
previously reviewed installed image and its native snapshot/packet-entity readers.
The public [assembly entry point](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/callbacks.asm#L1160)
and [individual entity writer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L2517)
remain hash-locked. No executable bytes or disassembly are included.

Player-state and client-state streams, complete snapshot framing, source epoch/
sequence ownership, message ACK admission, interpolation/prediction and engine
application remain separate gates. A containing parser must preflight those sibling
resources before treating this cooked data as an accepted snapshot. No native peer
session or playable client is claimed by these synthetic substream checks.
