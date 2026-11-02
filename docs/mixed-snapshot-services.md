# Mixed control and snapshot service carriers

`snapshots::mixed_services::MixedSnapshotPayloadPlan::parse` validates a complete
decoded ordinary carrier containing NOPs, commands, optional protocol-21 identity
updates and exactly one snapshot, through native logical EOF. Commands/identities
may precede or follow the snapshot. Unsupported services and a second snapshot
are rejected without skipping their bodies; this specialized carrier is not a
claim that the entire wire protocol forbids multiple snapshots.

Actual decoded profile/message binding precedes service inspection. The existing
control scanner now provides private incremental steps, preserving its original
primitive/string/sequence codecs. The original controls-only entry points use those
same steps. Command descriptors retain byte offsets into the immutable original
body; snapshot bit caches never align or rewrite surrounding byte-oriented records.
No projected body copy, peer-selected allocation or caller callback is introduced.

`MixedSnapshotStorage` separates private text/identity scratch from four final
outputs: normalized command bytes, owned identity updates, entities and client
states. The complete prefix, snapshot, suffix and EOF are checked before any final
write. Prediction is resolved before sparse replay or control publication. Closed
publication repeats the exact immutable normalization and sparse reads already
proven during validation; no source mutation or external callback occurs between
passes. All failure paths preserve all four complete final buffers and unused tails.
Private scratch remains unspecified on failure.

The input message's server-command sequence is the caller's actual initial host
high-water mark. Native command-prefix association uses the maximum admitted
positive sequence before the snapshot, retaining duplicate/older command records
for the normal host deduplication path. Tail commands remain in the cooked control
plan but do not change the snapshot's saved prefix. Locked public `CL_ParseCommandString`
advances this high-water mark only for newer commands, and the audited native
snapshot reader captures it before reading the header.

The result contains an ordinary `LegacyServicePlan` and coherent `SnapshotPlan` for
the same actual message/ACK metadata. Both outlive payload, decode buffers and
private scratch. Existing command/identity owners can preflight the cooked controls,
cancel or commit using their existing APIs. Parsing itself changes no command or
identity history, outgoing ACK, world state or renderer. Strings and identity names
remain inert, without peer execution or UTF-8 guesses. Debug omits private text.

Both profiles support commands plus snapshot data with explicit coordinate context.
Identity opcode eleven uses only the reviewed protocol-21 schema; the legacy
variant remains unsupported. Service inspection includes the snapshot opcode and
EOF, with independent identity, text and sparse-stream capacities/budgets. Logical
EOF ignores later native Huffman padding. Existing named baseline command-window
constraints remain distinct from future engine queues and negotiated extensions.

Ten integration cases cover normalization and identities on both sides, native
prefix-only command association, existing-owner preflight/cancellation/commit,
actual payload binding, late control/prediction failure, all four output tails,
independent limits, exact inspection boundaries, padding, unsupported/missing
framing, owned lifetimes, coherent delta reuse and private Debug. Existing control,
identity, pre-game and snapshot suites remain in the full workspace checks. The
empty identity-baseline fixture is shared once by the control and snapshot suites.

Next gates are an original-owner mixed-carrier ACK adapter, connection-wide host
history/session ownership, typed command/session effects, simulation and ordinary
peer captures. The core does not execute native command strings or install plugins.
