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
