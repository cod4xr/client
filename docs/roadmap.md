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
application, other message admission, legacy coordinates, application dispatch and
the engine's ongoing game loop. Snapshot deltas remain separate.
Owned protocol-21 client identity records now cover gamestate and ordinary sequenced
forms. Identity state now stages gamestate replacements and exact-next update batches
with atomic sequence/table publication. A joint lease commits the retained network
frame and identity baseline together. Ordinary-message identity dispatch, sibling
session resources and engine application remain open.
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
