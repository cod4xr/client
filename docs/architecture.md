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

Typed gamestate constituents live under client-core's `gamestate` module.
Shared entity wire code lives under `entities`, independently of its gamestate
consumer; compatibility re-exports preserve the original baseline/delta imports.
Both decoders use `EntityDecodeContext`, with explicit checked legacy map context.
Explicit configstring blocks use independent wire namespace and host work/storage budgets.
Validation precedes publication into caller-owned text and metadata; plans borrow
only cooked output. Initial entity baselines now have a separate owned semantic-word
representation and shared ordered-index state. Protocol 21 uses raw coordinates;
protocol 6 requires checked explicit map-center context. Complete
gamestate constituents use one private two-pass walker, validating all records,
trailers and capacities before publishing any final output. The complete auxiliary
gamestate wrapper validates its prefix, reserved tail and byte exhaustion before
that same publication pass. Other message admission and engine application remain
later boundaries, including complete legacy gamestates,
session/content application and complete snapshots.

The network owner can prepare a whole typed gamestate over its original ready
frame. An exclusive handle prevents concurrent frame replacement while the host
inspects cooked output; drop cancels release, and commit releases exactly that frame.
Decode work/failures and explicit commits have separate saturating counters. The
returned plan retains no socket/input-frame borrow or engine-update authority.

The new `cod4xr-content` crate owns deterministic file-data validation, beginning
with checked PE32 byte views and hash-reviewed native configstring defaults. It
has no filesystem/process/engine ownership and uses pinned SHA-256, Deflate and CRC libraries.
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
payload decoding and virtual mount policy remain separate layers.
Payload decoding owns reusable inline Deflate state but no allocator. Caller scratch
holds a guarded complete output; exact stream/length/CRC validation precedes final
copying. Checked results borrow only final storage. Host verification reuses entry
buffers, performs fallible reservations and reports only complete aggregate results.
Strict virtual file keys normalize native ASCII case/slashes into caller-owned bytes,
with independent byte/component budgets and full validation before publication.
They are lookup data, without physical path or extraction authority. Native request
sanitation, archive ownership and mount/duplicate policy remain separate boundaries.
Copied entry metadata contains numeric spans/declarations without borrowed ZIP bytes.
The payload decoder accepts a separately supplied exact byte range through the same
stream/length/CRC checks. Future virtual indexes can retain copied keys/metadata;
the host must own the original archive handles, epochs and bounded I/O.
Virtual mount indexes sort caller-owned source positions over immutable canonical
candidates. Last input wins each duplicate group; complete retention/storage
preflight precedes final key/metadata copying. The result borrows no source ZIP,
candidate or scratch data. Binary lookup returns copied routing/declarations;
archive IDs never imply priority, ownership or allocation by identifier magnitude.

The separate `cod4xr-client-content` host crate owns original file handles and
process-unique epochs. It preflights resources/ranges, checks observable file stamps
and feeds exact source bytes into deterministic decoding. No path is retained in
diagnostics. Stale/foreign epochs reject; copied lookup and final payloads retain
no source image or registry borrow. The inspector uses this actual owner and frees
the complete catalog image before per-entry reads. Search-order/pure/engine policy
remains separate from this filesystem ownership boundary.

IWI6 header geometry and direct bitmap/BC layout remain in the deterministic content
crate. Byte/mip/texel budgets precede payload views; active picmip extents and exact
exhaustion validate before complete layouts are exposed. Mip views borrow cooked
image bytes and assign no GPU authority. Header recognition includes wavelets while
direct interpretation rejects them. The inspector CRC-checks selected source entries
through owned I/O before separate header/direct-layout analysis. A separate pure
wavelet codec checks topology, budgets and caller capacities before reconstructing
in private scratch; complete pixels publish only after every mip/face succeeds.
Numerical codebooks reproduce from immutable public sources with retained licensing.
The inspector host owns reusable pixel/work buffers and aggregate pixel hashes.

Unsigned PC fastfile version-5 handling is another pure content boundary. A bounded
prefix inspection yields size requirements, independently from whole zlib/Adler-32
validation and exact-length publication. Serialized contents stay untyped; declared
block/external sizes grant neither allocation nor pointer authority. The host owns
file reads, fallible work/output buffers and diagnostic decoded hashes. This leaves
asset graphs, serialized relocation and engine resource admission to later layers.

The separate `zone5` layer accepts only a checked fastfile view. Counts inspection,
private metadata staging and final publication keep allocation planning distinct
from data admission. Cooked strings and copied asset records outlive source/work;
physical consumption and virtual-block alignment are accounted separately. Link
words remain inert metadata until per-type dispatch, including skipped native
records whose offset-like fields must not be dereferenced. The inspector allocates
exact admitted counts and validates its actual published catalog before reporting.

Client identity records have one owned representation shared by gamestate patches
and ordinary sequenced updates. Their small arrays are explicit native field
capacities, without imposing a player-table allocation on the engine. Parsing
retains raw normalized text and sequence bits. A separate caller-owned identity
state stages complete gamestate replacements and exact-next update batches before
atomic commit. A combined network-frame/identity lease commits both owners together
or cancels both, with local baseline generations and no source-buffer retention.
Whole mixed ordinary-message plans now feed the same identity staging rules and
server-command history. Sibling session/content admission remains separate.
Neither logs nor debug formatting contains that text.

Individual entity deltas reuse the generated baseline field catalogs
and one context-aware value reader. Index preparation exclusively retains both
original decoder/reader owners, allowing caller reference lookup by checked number
before complete record decode. Failure/drop rolls back both; success returns owned
inert state/removal and commits both cursors. No fixed entity table, snapshot
framing, host snapshot mutation or engine authority is part of this layer. Protocol
6 requires explicit checked map-center context, and its shared coordinate reader
distinguishes short prior-float addition from full prior-integer biased XOR. Context
stays on the exclusive index owner through preparation and commit.

Complete entity substreams now validate sparse source order/namespaces and every
record through the native terminator before publishing merged caller cells. A
private two-pass walker preserves both input cursors and all final cells on failure,
including late retention failures in the unchanged prior tail. Cooked source views
reuse final cells without repacking or retaining prior inputs. Host budgets replace
the native global parse-entity ring; full snapshot/epoch/message admission remains
separate from these inert substream results.

Snapshot client-state records share position-independent field value rules with
entities, but use their own semantic word catalog and six-bit index namespace.
Preparation leases the leading more bit, index and both reader cursors. Explicit
prior lookup returns owned state/removal only after complete field decoding; a
zero more bit commits stream termination independently of client index 63. Packed
names remain raw bytes outside Debug output. No global client ring, implicit
identity reconciliation or engine resource application is introduced.

Complete client-state substreams use the same closed source validation and sparse
merge cursor as entity substreams. Their own walker preserves the separate end bit,
zero-reference policy and client field schema. Private two-pass validation/replay
checks late retention and final tails before committing any caller cells or reader
cursors. Cooked source views retain only final storage; connection/generation and
snapshot-history admission remain host responsibilities.

Player-objective sections reuse the same position-independent scalar value rules
and a reproduced six-field semantic catalog. A privately owned sixteen-slot
candidate and cloned reader make complete-section publication atomic without
peer-selected allocation or borrowed network storage. Raw statuses and resource
words remain inert; reference profiles are checked before reading, while epoch,
sequence and full player-state admission remain separate host preflight.

Player stats, ammunition and weapon-model tables use independent owned section
types with explicit reference-profile checks and private candidate readers. Values
remain raw/inert; signed short arrays are absolute, and model tables are complete
byte replacements. The 64-entry stored-ammo wire projection is distinct from the
128-entry native struct and any wider engine inventory; applying it cannot imply
clearing out-of-projection entries. No source borrow or global native array is kept.

Native HUD storage owns both complete 31-slot wire arrays, retaining inert free-slot
words needed by future deltas. Private candidate state/reader publication makes
late errors atomic across archival and current arrays. The reproduced field table
shares ordinary value rules and checked integer-reference conversion, while color,
scale and 12-bit float windows retain reviewed native behavior. Typed presentation,
wider engine UI collections and resource lookup are independent of these wire words.

Live player-scalar prefixes own 141 inert words behind grouped closed field keys.
The reproduced catalog preserves conditional, unconditional and live forced-zero
reference rules without C offsets. Candidate publication is whole-prefix atomic.
Mode provenance records whether later command-time prediction lookup is required;
the prefix alone does not reconcile predicted words or admit following arrays,
complete player state, snapshot history or message acknowledgments.

Complete live player-state preparation composes all native sections behind one
exclusive reader lease. All candidates and prior prediction words are owned;
reference lifetimes end after preparation. Commitment selects transmitted values,
an exact profile/command-time sample or explicit native prior fallback, replacing
only eleven reviewed scalar words. Failed decoding/selection or dropped preparation
preserves the original reader; no unreconciled state escapes. Host-scoped prediction
queues, reference epochs, whole snapshots and message/ACK admission remain separate.
See [live player-state transactions](live-player-state.md).

Snapshot headers preserve the native six-byte time/distance/flag framing. Their
byte reads preserve preexisting cached bits for player-state decoding. The owned
raw full/delta selector performs no implicit sequence math, reference substitution
or history admission; all nonzero wire distances survive independently of old
engine ring sizes. See [snapshot framing](snapshot-headers.md).

Complete snapshot preparation composes these readers with one exclusive reader/
output lease. A closed cooked source binds all three state projections to one
message/world/profile; exact native message distance selects the prior. Every
source is checked before body reads. Player state remains privately pending and
both sparse walkers validate against no-op sinks before prediction resolution or
any final output write. Identical closed replay publishes both buffers and then
the original reader; failure/drop preserves all outputs and cursors. No native
history ring is reproduced. Enclosing message/ACK admission and host session/
history ownership remain separate; see [snapshot transactions](snapshot-bodies.md).

The specialized snapshot service carrier scans NOP/opcode framing and logical EOF
around the same private snapshot candidate. No final output write or prediction
commit precedes EOF. The enclosing lease advances through service bytes while
keeping the cooked snapshot's native body consumption distinct. Unsupported mixed
services are rejected, never skipped; wider routing/ACK ownership remains separate.
See [snapshot services](snapshot-services.md).

The snapshot payload adapter checks the actual decoded payload profile/sequence
against caller context before service work. Explicit prediction selection precedes
all output publication; cooked results retain only final storage and checked ACK
metadata. Temporary payload/key-history borrows end after parsing, while actual
ACK/history mutation remains with the original connection owner. Shared ordinary
profile widths are unchanged; see [snapshot payloads](snapshot-payloads.md).

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

Service-control plans validate NOP/command/EOF streams and opt-in protocol-21
identity updates before populating final normalized command bytes and owned client
cells. Caller-selected update budgets and private/final cell storage are independent
of the command window. They preserve full big-string records while deriving
the shorter native obfuscation key separately. Unsupported services reject before
partial output/application. Transport delivery sequence and host-validated message
sequence are distinct; only a complete validated message can advance the latter.
Typed execution, gamestate and session policy remain host work.

Typed pre-game preparation wraps a selected server-command update, inspecting only
new records and borrowing cooked plan bytes. Explicit byte/event budgets precede
history commit. Returned disconnect/map/restart notices retain full normalized
tokens and separate native copy-prefix views; they do not invoke console, scripts,
filesystem, UI or session effects. Unknown commands remain retained, including their
outgoing key. Hosts preflight sibling resources and commit the checked client ACK
before the prepared history; cancellation leaves both unchanged.

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
