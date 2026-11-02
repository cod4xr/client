# Owned native movement packets

`ControlMessagePlan::with_movement` builds a complete owned native packet from the
existing explicit header, original challenge, pending reliable-command history and
`MovementBatch`. It returns the same packet type used by `ClientSender::begin_control`,
preserving original protocol/challenge checks and complete-I/O admission. The idle/
reliable-only constructor keeps its existing wire vectors and behavior.

The batch contains one through thirty-two native input records, the original
gamestate checksum feed, an explicit prediction sample and an optional current-session
admitted snapshot. All command fields and profile/reference bindings are checked
before packet construction. Counts are rejected rather than clamped. These per-packet
wire bounds do not limit engine input queues or future negotiated formats.

The packet key combines checksum feed, admitted message ACK and the native hash of
the first thirty-two retained server-command bytes. The pinned public C function
uses **signed** x86 characters and arithmetic signed shifts, folding the original
sum in all three terms. Incoming 0x92 stays unchanged; it is not outgoing command
string cleaning. Rust expresses the native byte/shift semantics explicitly on
every host. The original obfuscation transform remains a separate shared operation.
Keys, checksum feed, input and prediction values stay out of Debug and fixed errors.

A supplied snapshot must match the header profile and cannot be newer than its
admitted message ACK. An exact message match selects native delta opcode zero;
an absent or older reference selects no-delta opcode one. The host remains
responsible for current connection/world/gamestate provenance. These representation
checks do not authenticate a peer or substitute for session lifecycle ownership.

One private message writer composes reliable commands, movement opcode/count, all
checked full-form commands, three raw predicted-origin words, the explicit prediction
time and logical EOF. The sample profile must match the header. Report time is
supplied explicitly; encoding does not infer it from arrival order or wall time.
IEEE origin words, including negative zero and opaque nonfinite bits, are preserved
without float conversion. Prediction production and physics policy remain separate.

Native bit bytes span intervening byte fields and adjacent commands. Prefix sizing
therefore sums actual bit counts and byte-field counts across the whole packet,
including opcode/EOF bits, before rounding the shared bit extent. Movement retains
its complete batch while the oldest fitting reliable prefix is selected. If Huffman
expansion exceeds the encoded budget, the builder retries a shorter reliable prefix,
including zero when movement is present. Deferred reliable commands remain pending;
no encoded bytes or input records are truncated. Existing native headers, Huffman
and compatibility transforms are reused.

`command_count` and `last_command_sequence` still describe reliable commands.
`user_command_count` separately reports movement records. Construction marks neither
kind transmitted and releases no ACK. Reliable transmission is admitted only after
the same owned frame completes outgoing I/O; acknowledgement requires a complete
validated response. [Input history](input-command-history.md) now separately retains
checked inputs and leases transmission admission through complete I/O. Backup
scheduling, prediction production and authoritative reconciliation remain host work.

Seven core cases cover both profiles, one/two/thirty-two inputs, ASCII/high-byte
packet keys, reliable prefixes and shared-decoder reconstruction of every field,
raw prediction trailers, exact/older snapshot selection and source lifetimes,
invalid counts/last-record fields/profiles/future references, header error order,
private Debug, atomic output and reliable-prefix capacity. A real Huffman-expansion
case defers the final reliable record while retaining and decoding all thirty-two
movement commands and EOF. One private hash case checks nine compiler-derived
[fixtures](../crates/client-core/tests/fixtures/movement_key_vectors.txt).

A real protocol-21 UDP case first admits a server-command key containing 0x92,
then sends a reliable command and three movement inputs through the original
acknowledged socket. The independent responder checks both native keys, all header
ACKs, each input, prediction trailer and EOF, then acknowledges the transmitted
reliable command. The client releases that pending record only after complete
response validation. This is synthetic transport evidence, not a native playable
session or an auth-required-server implementation.

The newly pinned public `src/common.c` joins the existing `cl_main.c`, `msg.c`,
`qcommon.h` and build flags. [Key evidence](movement-key-evidence.json) records its
normalized hash and the compiler evaluation method. No generated executable,
library, disassembly or game data is distributed. Source-only fixtures, profile
vectors and hosted source verification remain distinct from ordinary peer captures.
