# Client work

Each gate needs observable behavior and recorded evidence. Source research,
scaffolding and a successful query are not substitutes for a playable client.

| Phase | Scope | Acceptance |
| --- | --- | --- |
| Foundation | Separate repo, shared immutable core, discovery, logs and standards | Exact bytes, hostile-input tests, real local UDP/process checks; implemented |
| No-auth connection | Version profiles, challenge/connect/retry/disconnect ownership | Ordinary server reaches an accepted connection with bounded state |
| Transport | Reliable commands, fragments, gamestate and snapshots | Captured peer traces, loss/reorder and malformed-message workloads |
| Content | Installed stock assets, archive mount order, maps and downloads | Stock/custom content loads with checked imports and pure validation |
| Presentation | Window, rendering, input, audio and UI | Correct frame/input behavior on target hardware |
| Simulation | Prediction, collision, movement, weapons and snapshot reconciliation | Recorded deterministic replays and live reference comparisons |
| Scripting/mods | Client-side presentation and expanded engine features | Named stock modes and existing mods run correctly |
| Release | Full sessions, version matrix, soak/performance/security | Supported ordinary no-auth servers play through complete sessions |

Connection libraries now own challenge/connect/ack transitions and ordinary
fragment transport. Auxiliary receive windows and application framing are covered
by deterministic and loopback tests. Outgoing queues and whole ACK admission now
have model tests; explicit auxiliary UDP sends now couple OS I/O to transmission
admission. Scheduling now paces original retries and ACKs with combined I/O admission.
Unified endpoints now own windows, framing, whole-packet admission and scheduling.
Connection owners now bind the original socket, actual host clock and endpoint,
with scoped ordinary callbacks and bounded packet operations. Complete-body reader
views now preserve shared field codecs beyond 128 KiB. Typed configstring blocks
now validate both baseline formats with caller-funded storage and whole-block
rollback. Protocol-21 initial entity baselines now decode all eighteen native
layouts, with shared index handling and owned semantic words. Complete protocol-21
gamestate constituents now validate all records/trailers before output publication.
Complete reliable gamestate bodies now validate their wrapper/tail before output,
with real UDP coverage across reordered frames and explicit application rejection.
Prepared owner handles now couple typed validation to one retained frame, with
exclusive inspection, cancellation and explicit commit/release counters.
Reviewed installed-client defaults now import through a separate deterministic
content crate and bounded metadata-only inspection command. Wire plans preserve
original block starts for correct per-block native default restoration. Effective
sparse snapshots now merge defaults/wire values with bounded work and transactional
publication. Read-only system-info views now validate full-width native server IDs
without applying arbitrary remote engine settings. Next are session/engine
application, other message admission, complete legacy gamestates, application dispatch and
the engine's ongoing game loop. Snapshot deltas remain separate.
Owned protocol-21 client identity records now cover gamestate and ordinary sequenced
forms. Identity state now stages gamestate replacements and exact-next update batches
with atomic sequence/table publication. A joint lease commits the retained network
frame and identity baseline together. Whole mixed ordinary messages now stage both
commands and identity batches, with next-frame UDP acknowledgement coverage. Sibling
session resources, typed execution and engine application remain open.
Typed pre-game preparation now selects disconnect/map-change/fast-restart notices
from new server commands with cancel/commit semantics and actual next-frame UDP key
coverage. Applying session, content, UI or restart effects remains host work.
Individual protocol-21 entity delta records now accept explicit nonzero references,
return owned states/removals and preserve both reader/index owners on failure.
Complete snapshots, remaining player-state sections and native peer traces remain
transport gates before simulation or gameplay admission.
Initial protocol-6 baseline coordinates now use checked explicit map-center context
with native precision/bias rules and the same atomic reader/index admission. Legacy
gamestate context integration remains a separate gate. Individual nonzero legacy
records now use explicit owned context, native short/full origin behavior and the
same prepared reader/index transaction as modern records.
Complete entity substreams now merge prior states, baselines, updates and removals
through the native terminator before final publication. Shared wire code lives in
the dedicated `entities` module, with compatibility imports retained. Caller-owned
results can retain more total entity states than the native ring; full snapshots,
player-state sections and epoch-bound history remain separate work.
Individual snapshot client-state records now decode all twenty-four native fields
for either baseline profile, with raw packed names, explicit prior selection and
joint framing/index/field rollback. The independent more-records bit preserves
client 63 as valid. Complete client-state substreams now merge prior/zero references
and removals before atomic final publication, sharing the entity sparse cursor and
supporting cooked-source reuse. Player-state decoding remains a gate before
whole-snapshot admission and epoch-bound history.
Complete native player-objective sections now return owned sixteen-slot results
with explicit profile binding, raw status preservation and whole-section reader
rollback. Stats, ammunition groups and raw weapon-model tables now have separate
owned transactional codecs with native wire projections. Scalar/prediction and HUD
sections are still required before complete player-state and snapshot admission.
Do not use discovery's protocol-6 field alone to choose a gameplay dialect.
Capture compatible ordinary-peer traces before claiming broad interoperability.

Classic stored/Deflate IWD catalogs now have checked metadata, caller-funded
descriptors and an inspector mode. Installed archive counts match an independent
ZIP reader. Stored/raw-Deflate payloads now have exact stream/length/CRC validation
and an explicit in-memory verification command. Next content gates are virtual
mount policy and native pure checksums before resource loading. Strict canonical
virtual file keys now cover native ASCII case/slash equivalence without OS path
authority; native request sanitation and physical resolution remain separate.
Copied numeric entry metadata and separate-source decoding now allow later payload
reads without retaining whole ZIP buffers. Host handle/epoch ownership remains open.
Copied virtual indexes now preserve caller-specified archive priority and native
last-entry duplicate precedence, with model and catalog-to-payload integration.
Actual archive search-order discovery and pure policy remain open.
An owned archive registry now retains original file handles with process-unique
epochs and bounded checked reads. Inspector verification releases whole ZIP input
before reading individual payloads through that owner. Full engine search order,
pure checksums, asset formats and playable rendering/simulation remain open.
PC IWI6 metadata and complete direct bitmap/BC layouts now have independent vectors
and installed-corpus inspection. All five wavelet formats have a separate bounded
pixel decoder and independent installed-pixel comparisons. GPU resources/uploads,
backend color policy and renderer integration remain open.
Unsigned PC version-5 fastfile envelopes now have bounded metadata inspection and
complete zlib/size validation before publication. Serialized relocation, typed
asset loading, resource ownership and map/model rendering remain open.
Root script-string and asset-directory catalogs now stage privately and publish
into exact caller-owned storage. Child schemas, string aliases, target relocation
and typed resources remain separate implementation gates.

Features that remove asset/model/image/entity/configstring limits need both
internal storage/resource design and a negotiated extended wire/content profile.
Ordinary servers continue to use their baseline limits. Internal storage should
not inherit 32-bit engine globals merely because the legacy wire has a bound.
Do not silently reinterpret legacy identifiers or promise lifted limits to peers
that have not negotiated them.
