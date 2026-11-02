# Snapshot-only service carriers

`snapshots::services::SnapshotServicePlan::prepare` validates a specialized ordinary
service carrier: any NOPs, one snapshot, any NOPs, then native logical EOF. It starts
at a service opcode in an explicit `BodyReader`, retaining the existing snapshot
input's world/profile/message/reference bindings and caller-funded output buffers.
Both baseline profiles use NOP zero, snapshot six and EOF seven.

The complete snapshot and service tail are checked privately before returning a
prepared lease. Missing EOF, an unsupported trailing service, a second snapshot,
exhausted inspection budget or malformed constituent leaves the original reader
and both entire output buffers unchanged. Other service bodies are never skipped
or executed. This entry point is intentionally for the named carrier; commands,
identity updates, gamestate, downloads and multiple snapshots require a broader
combined decoder. Their rejection is not a claim about a universal wire limit.

`ServiceControlBudget::max_records` counts every NOP, the snapshot service and EOF.
Zero rejects before reading; the minimal carrier needs two records. Native Huffman
padding after logical EOF remains uninterpreted, matching the existing controls
parser. Service/header byte operations preserve cached snapshot bits without
alignment. Service consumption includes all inspected opcodes; the inner cooked
snapshot retains its separate native body-only physical byte count. Cached bits
can make these counts differ from a standalone byte-aligned fixture.

The prepared carrier privately owns the whole-snapshot reader/output lease. It
exposes only header, prediction-request and bounded service-count metadata.
Transmitted, matching-sample and explicit-prior-fallback commitments delegate to
the existing snapshot transaction after EOF validation. Prediction failure or drop
cancels the whole carrier. Successful commitment advances through EOF and returns
an input-independent cooked plan whose inner snapshot can supply a coherent future
reference. Debug/error output excludes raw state words and peer text.

Nine cases cover independent physical framing, exact EOF/padding boundaries,
cached-bit continuity, following bytes, owned cooked references, every tested
truncation, unsupported prefix/tail services, missing snapshot/EOF, every inspection
boundary, cancellation and failed/successful prediction choices. A compile-fail
example protects the enclosing reader lease. The existing snapshot, player-state
and sparse-stream suites remain unchanged.

The service dispatch/EOF grammar comes from locked public `src/cl_main.c` and
`src/qcommon.h`; native snapshot constituents retain their separate reader audits.
No runtime C implementation is copied. A complete mixed-service message decoder,
binding to the actual decoded payload/ACK owner, joint command/identity/history
publication, host gameplay ownership and ordinary peer sessions remain open.
