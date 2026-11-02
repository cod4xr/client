# Complete snapshot client-state substreams

`snapshot_clients::SnapshotClientPlan::parse` validates and merges the complete
native snapshot client-state substream for an explicitly selected baseline profile.
It consumes the separate zero end bit without consuming following byte or cached
bit fields. Client 63 remains an ordinary record. It does not admit player state,
an enclosing snapshot/message or a transport acknowledgment.

The caller supplies an immutable prior `ClientStateSource`, independent inspection,
wire-record and retention budgets, and final `SnapshotClientSlot` storage. Every
prior entry is checked for strict ordering, uniqueness and six-bit representability
before wire reads, even if no update mentions it. Wire work includes the end bit;
64 explicit clients plus termination require 65 record headers. These are native
wire constraints, independent of engine-wide storage and future extension profiles.

The sparse merge follows the reviewed native reader: unmentioned prior clients
before each update are retained, a matching prior state supplies delta references,
new clients use zero words, removals omit output, and the end bit retains the
remaining prior tail. Client-state records never use entity baselines. A closed
ordered-source validator and merge cursor are shared with entity substreams, while
the distinct framing, field schemas and reference policies remain explicit.

A private validation pass checks all records and the complete final extent without
publishing anything. An identical immutable replay writes final cells, then both
original reader cursors commit. Invalid late records, missing framing, source
errors and independent budget/capacity failures leave the entire final buffer,
including unused tails, and all original cached bits unchanged. No external
callbacks, I/O, peer allocations or global rings run between the two passes.

The returned plan borrows only its initialized final output. It retains no wire or
prior-source borrow. `as_source` reuses cooked cells without repacking and can
outlive the parent plan; Rust prevents mutating their backing storage while the
source is still used. The next parse revalidates the whole source against its
current budgets. Host-selected connection, map/gamestate generation and snapshot
sequence binding remain separate requirements before application.

Thirteen integration tests cover native physical framing, client 63, prior/zero
reference selection, sparse updates/removals, retained tails, complete prior-source
validation, independent budgets, late failures and dirty-output rollback, cached
bits, every tested truncated record prefix, missing end framing at a byte boundary,
cooked-only lifetimes, repeated source reuse and 128 ordered-map traces per profile.
Forty independent results retain 2,560 states simultaneously, beyond the native
2,048-entry global ring; this demonstrates codec storage ownership, not admitted
snapshot history. A compile-fail example protects borrowed cooked cells.

[Native audit metadata](snapshot-client-state-reader-evidence.json) identifies the
reviewed reader, zero-reference path, prior-prefix and tail copies. The existing
[record decoder](snapshot-client-states.md) supplies the reproduced 24-field catalog
and shared value rules. Player-state sections, epoch-bound reference history,
whole-message admission, native peer captures, resources and gameplay remain open.
