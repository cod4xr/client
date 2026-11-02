# Client architecture

CoD4XR has separate client and server repositories. Shared codec libraries are
consumed from `cod4xr/server` at a complete immutable Git revision recorded in
Cargo.toml and Cargo.lock. The client never depends on the server application,
query responder, world ownership or runtime orchestration. A dependency update
must explicitly review conformance and public Rust API changes.

This supersedes the original server ADR's single-workspace direction for the
client product. It retains one implementation of shared wire formats rather
than copying protocol code into a second repository. Extracting a dedicated
shared-core repository is a later ownership decision, not a prerequisite here.

The new auxiliary reliable body format lives in `cod4xr-reliable`, a neutral
dependency-free crate within this repository. It implements a previously absent
body codec, retaining the shared outer envelope rather than duplicating it. This
boundary permits later reuse without depending on client OS or game ownership;
no server source or published shared revision changes are required.

Complete reliable bodies use a client-core storage-view adapter over the pinned
shared primitive reader. It rebases bounded views and preserves the legacy cached
bit byte; integer, bit, string and index field decoding still has one shared
implementation. This permits native reliable bodies larger than the primitive
view cap without turning that cap into an engine memory or asset limit.

Typed gamestate constituents live under client-core's `gamestate` module. Explicit
configstring blocks use independent wire namespace and host work/storage budgets.
Validation precedes publication into caller-owned text and metadata; plans borrow
only cooked output. Protocol-21 zero-reference entity baselines now have a separate
owned semantic-word representation and shared ordered-index state. Complete
gamestate constituents use one private two-pass walker, validating all records,
trailers and capacities before publishing any final output. The complete auxiliary
gamestate wrapper validates its prefix, reserved tail and byte exhaustion before
that same publication pass. Other message admission and engine application remain
later boundaries, with native constant
defaults, legacy coordinate fields and nonzero-reference snapshot deltas still open.

The network owner can prepare a whole typed gamestate over its original ready
frame. An exclusive handle prevents concurrent frame replacement while the host
inspects cooked output; drop cancels release, and commit releases exactly that frame.
Decode work/failures and explicit commits have separate saturating counters. The
returned plan retains no socket/input-frame borrow or engine-update authority.

The new `cod4xr-content` crate owns deterministic file-data validation, beginning
with checked PE32 byte views and hash-reviewed native configstring defaults. It
has no filesystem/process/engine ownership and depends only on pinned SHA-256.
Host-side inspection opens an explicit user-selected file under a byte quota,
imports into prefunded buffers and reports metadata through the shared logger.
Game binaries and default values are not distributed. Native default importing
is separate from merging or applying a validated gamestate to an engine.

Validated configstring views preserve original block starts and a total block count.
Native constants are reapplied for each block; these boundaries keep later default
merging source-independent without silently changing omission/reset semantics.

Effective config snapshots merge immutable source descriptors with caller-owned
sorting scratch and packed final storage. Native default restoration follows original
block scope; wide sparse IDs never allocate an ID-sized array. Input work and final
retention have independent budgets. All capacities/selected values validate before
publication, and only final output is borrowed by the returned snapshot.

System-info interpretation borrows that effective snapshot and reuses the shared
big-info grammar. Explicit byte/field budgets bound inspection; the critical server
identifier is unique and decimal-checked with all 32 bits preserved. Unknown fields
remain inert bytes. This metadata layer owns no CVar changes, resource mounts,
session epoch, protocol selection or message/config acknowledgement authority.

Content archive catalogs retain original central-directory order over immutable
caller bytes. A private descriptor workspace validates local spans and rejects
overlap before publishing final metadata. Entry views carry no path or integrity
authority. Host file I/O and metadata reporting stay in the existing inspector;
decompression, CRC verification and virtual mount policy remain separate layers.

Client identity records have one owned representation shared by gamestate patches
and ordinary sequenced updates. Their small arrays are explicit native field
capacities, without imposing a player-table allocation on the engine. Parsing
retains raw normalized text and sequence bits; session admission and whole-message
application remain separate. Neither logs nor debug formatting contains that text.

The deterministic core owns request encoding and checked borrowed metadata.
It has no sockets, clocks, logging subscriber, filesystem access, renderer or
process-global state. The network crate owns UDP resources, challenge entropy,
complete receive buffers and monotonic I/O deadlines. Core state enforces phase
quotas and elapsed time. The application owns endpoint/budget selection, discovery
entropy, output and the logging lifetime. Rendering/input/audio
and simulation boundaries will be added as implemented behavior warrants them;
there are no placeholder engine crates or fabricated game states.

Challenge, pending-connect and acknowledged transport have consuming ownership
transitions on one bound socket. Refusal and failure close it. The network API is
blocking and intended for an I/O worker; no library installs a runtime or log
subscriber. See [network ownership](owned-network-phases.md) for the public phase
contract. Acknowledgement can be consumed into an ordinary channel using caller-
owned reassembly storage. Disjoint borrowed send/receive halves permit host pacing
and receive work between fragments. Shared sequencing/fragmentation is reused
without copying it into client code. Message bodies and auxiliary transports stay
opaque until their separate checked layers are implemented; see
[ordinary channel](ordinary-channel.md). Gamestate initialization remains open.

The deterministic command history separates enqueue, host-admitted transmission,
read-only acknowledgement checking and post-validation commit. One dedicated key
slot survives pending-ring reuse. This named baseline has up to 128 pending
commands; it does not constrain future engine strings or negotiated custom command
profiles. Incoming server commands are not executed by this storage layer.

Owned control plans serialize a complete pending prefix under explicit 9/16-byte
header profiles, then reuse shared compression/obfuscation. Logical and wire budgets
are enforced independently; compression expansion rebuilds a shorter whole prefix.
The network sender checks original protocol/challenge before reservation. A plan
does not mark commands transmitted or commit acknowledgements; the host owns those
transitions after complete I/O and validated inbound messages, respectively.

Inbound ordinary payload decoding obtains its profile/cookie from original socket
context, checks the retained transmitted command key and uses shared transforms/
decompression into caller buffers. Decoded views retain a history borrow and expose
untrusted service bytes, including native Huffman padding. They neither execute
records nor commit acknowledgements before service validation through logical EOF.

Service-control plans validate NOP/command/EOF streams before populating final
normalized command output. They preserve full big-string records while deriving
the shorter native obfuscation key separately. Unsupported services reject before
partial output/application. Transport delivery sequence and host-validated message
sequence are distinct; only a complete validated message can advance the latter.
Typed execution, gamestate and session policy remain host work.

Server-command history retains full normalized records in caller-selected slots,
including capacities above 128. Exact sequence tags protect reused slots. Preparation
checks profile/message order and selects newer records without mutation; an exclusive
prepared update has an infallible commit. The host can commit its already checked
outgoing acknowledgement first, then incoming storage, or cancel the prepared update
on failure. Summary counters expose work without logging private command/key bytes.

Discovery uses the existing connectionless envelope and normal info-string profile.
Requests contain a 128-bit random correlation value as 32 ASCII hex bytes. A reply
must come from the connected endpoint and contain exactly one matching challenge.
Unknown fields are preserved; no unknown field grants authority or activates code.
The protocol advertisement is displayed without selecting a join dialect.

One receive buffer admits a complete UDP datagram so truncation cannot turn an
oversized packet into an accepted prefix. The parser independently enforces the
normal-info bound. Receive work and elapsed time are both bounded. The client
does not execute peer commands, automatically download content, launch an existing
game executable or install hooks in this phase.

The no-auth baseline is explicit. There is no credential flow or fallback that
claims to satisfy an auth-required server. Optional future engine capabilities
must be negotiated independently from baseline versions. Community/mod-specific
behavior belongs above reusable engine interfaces. No extension bits, identifiers
or handshake have been reserved without an implemented and reviewed requirement.

Logging reuses the server's isolated bounded runtime. Libraries install no global
subscriber; the application keeps workers alive through shutdown. Console logs
are readable in debug builds, JSON in release, with an explicit format override.
Raw remote metadata belongs only in escaped report output, not operational logs.
