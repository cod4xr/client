# Validation evidence

Latest local checkpoint: 614 tests pass in both debug and release on the pinned
Windows/Rust host. All-target Clippy and rustdoc pass with warnings denied;
formatting and repository policy pass (601 source files, 6 crates; all checked source
files below 250 lines). Discovery, challenge state/runtime, owned connect requests and bounded
reply decoding, bounded pending-connect state and owned UDP phase transitions are
covered, along with ordinary-channel integration, checked client-command history,
owned control encoding, payload/service validation, configurable server-command
retention, auxiliary reliable bodies, receive/send windows, whole ACK admission,
application framing, paced auxiliary UDP sends and socket/clock/endpoint ownership.
Complete protocol-21 gamestate constituents now have transactional coverage.
Reliable gamestate bodies now have whole-frame admission coverage. Other message
admission, the ongoing game loop, typed command delivery,
engine/content application and ordinary gameplay remain open.

The 590-test player-HUD checkpoint passed hosted Windows/Linux CI in
[run 41504237250](https://github.com/cod4xr/client/actions/runs/41504237250), including
locked sources and table reproduction. Hosted live player-scalar validation will
follow its commit. These changes edit only the client repository and consume the
published shared revision.

Twenty-four live scalar-prefix tests add independent count/time/angle/signed-short
vectors, both profiles, all 141 typed keys, full prefixes, all forced and
unconditional fields, omitted references, float forms, signed masking, flags,
events and wrapping times. All invalid counts and tested truncated full prefixes
preserve prior words and both cursors; late compact-reference failures, profile
binding, owned lifetimes and private Debug formatting are covered. One hundred
twenty-eight modeled traces per profile compare all words, including forced-zero
and unconditional behavior. All 141 native names/widths/hints match the locked
schema; [audit metadata](player-scalar-reader-evidence.json) also verifies the live
caller and prediction-request boundary. Whole player-state/prediction/snapshot
admission remain open; see [live player scalars](player-scalars.md).

Twenty-six player-HUD tests add independent framing vectors, all forty field rules,
both profiles, all scale indices and alpha forms, compact full-prior/wrap behavior,
raw floats, modern origins, explicit legacy centers and wrapping message times.
They cover both 31-slot arrays, exact first-free tail retention, inactive words used
by later deltas, owned lifetimes, private Debug output, every tested truncated full
section, all invalid last-field indices and failures in the last current element
after valid archival changes. One hundred twenty-eight modeled array traces compare
prefix updates and native clearing independently. The reproduced catalog matches
all forty native offsets/widths/hints; [audit metadata](player-hud-reader-evidence.json)
also binds the precise scale constants and numeric branches. Scalar/prediction and
complete snapshot admission remain open; see [player HUD](player-hud.md).

The core integration harness now loads all 31 component suites in one executable
and shares four fixture modules once. Before/after listing comparisons preserved
the exact qualified identities of 289 integration and nine unit cases; the
[inventory evidence](core-test-inventory.json) records their canonical hash. Full
workspace coverage remains unchanged. Missing, duplicate, unknown and conditional
harness declarations fail repository policy. Production runtime code, release
settings and the existing local Cargo target remain unchanged; see
[test organization](testing.md).

Twenty-one player-array tests add native stats/ammunition physical vectors, both
profiles, signed/unsigned boundaries, absolute updates versus XOR, high mask bits,
first/last wire slots, all groups, explicit zero clearing, empty masks, omitted
retention and raw model-table replacement. All tested truncations preserve cached
reader bits and prior ownership, including the last clip flag and last model byte.
Profile mismatches reject before reading; cooked values outlive inputs and Debug
output excludes inventory/model contents. One hundred twenty-eight modeled traces
per section/profile compare every retained value.
[Native audit metadata](player-array-reader-evidence.json) binds group counts, signed extension and model
bytes to the reviewed reader. Scalar/prediction decoding remains open;
see [player arrays](player-arrays.md).

Fourteen player-objective section tests add both explicit profiles, independent
physical status vectors, raw reserved statuses, whole-set/entry/word retention,
all six fields, prior XOR and float forms, zero clearing, cached/following fields,
every truncated full-section prefix, late compact-reference rollback, profile
binding and source-independent ownership. Debug output hides all objective words.
One hundred twenty-eight modeled traces per profile compare every slot/status/word
against independent arrays. The reproduced catalog matches all six native offsets,
widths and hints; [audit metadata](player-objective-reader-evidence.json) records
the parent sixteen-slot loop and leaf reader. Remaining player-state sections and
whole snapshots remain open; see [player objectives](player-objectives.md).

Thirteen complete client-state substream tests add native end-bit framing, client
63, cached/following fields, all sixty-four clients with exact header budgets,
prior/zero references, sparse merge/removal/tail rules, full source validation,
independent budgets and late dirty-output rollback. Cooked-only lifetimes, reusable
source views, 128 ordered-map traces per profile and forty results retaining 2,560
states exercise owned storage beyond the native global ring. A compile-fail example
protects borrowed cooked cells. Entity/client substreams share their closed ordered
validator and merge cursor; existing entity-stream tests continue to pass.
[Client-state substreams](snapshot-client-streams.md) remain separate from complete
snapshots, player-state sections, host reference epochs and message admission.

Fifteen client-state record tests add all twenty-four fields, physical byte vectors,
both baseline profiles, prior XOR and omitted-word retention, raw packed names,
signed-zero/raw/compact floats, all sixty-four client indices, separate end framing,
owned lifetimes, cancellation, invalid references/counts/order, every truncated
full-record prefix with cached bits and 128 modeled traces per profile. Two
compile-fail examples protect leased reader/index owners. The reproduced catalog
also matches all twenty-four offsets, widths and hints in the reviewed native
image; [audit metadata](snapshot-client-state-reader-evidence.json) records that
comparison. Complete client-state merging now has separate coverage; player-state
decoding and full snapshot admission remain open; see
[client-state records](snapshot-client-states.md).

Fifteen complete entity-substream tests add native terminator/framing bytes, maximum
index termination, valid existing cached bits and following fields, full sparse
merge rules, previous/baseline/zero precedence, removals, legacy context, all tested
truncations, late malformed records, input source/order failures, independent host
budgets/capacities and unchanged final tails. Cooked-only lifetimes/source reuse,
128 ordered-map traces, 512 retained entities and six simultaneous results with
3,072 entities exercise owned storage beyond the native global ring. A compile-fail
example prevents overwriting borrowed cooked input cells. Shared codecs moved to
`entities`, preserving compatibility imports and source-table reproduction.
[Audit metadata](snapshot-entity-reader-evidence.json) binds stream rules to the
reviewed native reader. Complete snapshots, player-state sections and native peers
remain open; see [entity substreams](snapshot-entity-streams.md).

Eleven nonzero legacy-delta tests add explicit coordinate-profile ownership, all
short values against prior state, fractional short/full distinctions, all eighteen
target layouts, native signed-wrap/binary32-rounding boundaries, context replacement
and reset, no-delta/removal handling, owned lifetimes, every truncated prefix with
cached bits, cancellation and late invalid-reference rollback. Nonfinite changed
origins reject; full forms additionally check signed-32 prior conversion, while
finite short forms keep native float addition. Existing baseline fixtures now share
one support module. Complete snapshots and native peer traces remain open; see
[legacy entity records](legacy-entity-deltas.md).

Nine legacy-baseline integration tests and two physical field tests add explicit
protocol-6 map-center context, all 128 short values on each axis, sixteen-bit full
coordinate windows, all eighteen layouts, native half-bias/truncation precision,
signed conversion bounds, nonfinite rejection, context replacement without index
reset, context-preserving reset, source ownership, all record truncations and late
cached-bit rollback. A native x87 audit verified precise addition before integer
conversion, avoiding an incorrect intermediate binary32 rounding. Existing modern
baseline/delta vectors remain covered. [Metadata evidence](legacy-coordinate-reader-evidence.json)
contains no binary bytes or disassembly. Complete legacy gamestates, nonzero legacy
records and native peer traces remain separate; see
[legacy baseline coordinates](legacy-baseline-coordinates.md).

Ten entity-delta integration tests and three field tests add nonzero references,
all eighteen target layouts, type changes, signed/XOR/absolute field distinctions,
message-relative time, owned states/removals, wrong-reference rejection, cancellation,
all record truncations, cached-bit rollback, compact-float reference bounds and
stream ordering/reset. A 1,024-record trace checks repeated no/full/partial changes.
Two compile-fail examples protect both reader and index owners. A read-only audit
of the reviewed installed native reader identified the narrow integer reference
mask and signed 16-bit angle conversion; the latter also corrects initial baseline
decoding and has new high-angle vectors. The sixteenth client source hash binds the
native entry point. [Evidence](entity-delta-reader-evidence.json) contains metadata
only. Complete snapshots and native peer sessions remain open; see
[individual delta records](entity-deltas.md).

Fifteen typed pre-game tests add new-command-only disconnect/map/restart notices,
native raw-prefix/token/copy-boundary rules, normalization and raw bytes, all
control-byte categories, truncated quote/comment forms, unknown-key retention,
slot collisions, source/history lifetimes, independent byte/event budgets and
late cancellation. A compile-fail example protects exclusive history ownership.
One actual UDP test rejects a third notice on budget, cancels/retries preparation,
then returns all notices after history/ACK commit despite one retained slot. Its
peer decodes the next frame with message ACK 7, command ACK 3 and key `m`. The
fifteenth client-reference source hash now pins the tokenizer. No console, script,
UI, filesystem or session effects execute. See [typed controls](pre-game-controls.md).

Twelve mixed-service core tests add complete protocol-21 command/identity/NOP/EOF
streams, independent identity counts beyond the command window, source/workspace
release, strict legacy/profile entry points, every truncation, late unsupported or
malformed identities, independent budgets/capacities, unchanged final tails, logical
padding isolation and joint command/identity preflight cancellation. One actual UDP
test rejects a late bad service without releasing a pending client command, then
admits a valid mixed stream. Its peer decodes the next client datagram with message
ACK 8, command ACK 3, config-data ACK 11 and the new retained key. Ordinary wire
limits stay unchanged; typed execution, other services and native gameplay peers
remain open. See [mixed controls](mixed-service-controls.md).

Ten configstring-block tests add independent legacy packed-index and protocol-21
32-bit vectors; normalized/raw/empty strings; cooked-only plan lifetimes; both full
wire namespaces; a complete block above 128 KiB; native maximum string lengths;
invalid counts/order/IDs; every input truncation and short caller capacity; host
budgets; dirty metadata and preservation of original reader/cache state on late
failure. This validates explicit patches, without native constant defaults, entity
baseline deltas, whole gamestate application or a real gameplay peer.

Six client-data tests add owned protocol-21 gamestate and ordinary sequenced
identity vectors, maximum/empty names and tags, raw bytes and percent normalization,
source-independent ownership, large reader offsets, unsupported profiles, every
truncation, strict field/identifier bounds, sequence high-bit preservation, cached
bit rollback and private debug output. The newer identity-state and mixed control
tests cover sequence admission and complete supported carriers separately; engine
updates and remaining services stay open.

Ten entity-baseline tests add independent physical field/header vectors, all eighteen
full protocol-21 layouts plus unknown-type fallback, unconditional-field positions,
integer/float/angle/flag/time/ground forms, signed union values, large/raw coordinate
bits, owned results, large input offsets, following byte/bit fields, every tested
truncation, invalid counts/removal/order/namespaces, explicit reset and existing-cache
rollback. Generated field catalogs reproduce from the same locked client source
hashes locally; hosted CI now includes the same check. No native captures, legacy coordinate support,
nonzero-reference snapshots or engine application are claimed.

Nine gamestate tests add independent complete constituent framing, cooked-only plan
lifetimes, mixed/interleaved records, multiple configstring blocks with global text
offsets, repeated client IDs, exact following byte/bit fields, all 4884 configstrings
and 1024 baseline entities, 65 retained client records, a complete body above 128 KiB,
every fixture truncation, invalid nested/opcode/trailer records, unsupported profiles,
dirty-output and cached-bit rollback, short capacities and exact host-budget bounds.
Enclosing-message admission, default importing, engine application and native peer
captures remain outside this constituent-level validation.

Five reliable-wrapper tests add independent complete body bytes, opaque tail bits,
source-independent plans, native unused-bit padding, every fixture truncation,
invalid commands/opcodes, every appended byte value, whole-body/host/storage budgets
and late extra-byte rejection above 128 KiB. A real UDP test reorders complete frames,
rejects a malformed earlier body without changing gamestate output, explicitly
discards it, admits the next valid body and retains the cooked plan after network
frame release. Transport byte ACKs remain separate from application validity. An
additional immutable public writer file is hash-locked from read-only remote evidence;
local server source/worktrees were not read or edited for that verification.

Three owner-preparation UDP tests add no-frame/terminal rejection without decode
work, exclusive preparation, cancel/retry, exact-once typed commit/release, cooked
data after socket/source-buffer release, failed decode preservation and explicit
discard followed by valid admission. New saturating counters distinguish attempts,
decode failures and commits, without changing receipt time or byte ACK state. One
compile-fail documentation test prevents replacing a frame while its prepared
handle remains live. Session/engine updates remain separate from frame commit.

Twenty-one content/inspector tests add synthetic PE32 header/section/address
validation, every required prefix, virtual-only/overflow geometry, deterministic
header/address mutations, exact image identity, native table shape/order/terminators,
short budgets/storage with complete rollback, raw defaults beyond wire lengths,
source-independent plans, native/non-Unicode arguments, bounded regular-file reads
and JSON/human process failures without paths or values. Debug and release inspector
runs against the reviewed installed client both imported 832 records/18,813 value
bytes; [metadata evidence](client-image-defaults-evidence.json) records its identity.
No game binary or values are included in fixtures or Git. Session admission,
engine/content application, maps/rendering and playable sessions remain open.

One block-scope regression verifies native default restoration across multiple
blocks without erasing earlier nondefault IDs. Existing standalone/mixed/minimal
tests also verify original start markers and exact block counts, with no source
frame retained or additional wire field. Existing failure tests continue to cover
whole-output rollback, including this metadata.

Eight sparse-config tests add native per-block restoration, nondefault persistence,
empty/latest overrides, no-block behavior, wide sparse IDs, large/raw values,
source-independent output, every short budget/scratch/final capacity, dirty workspace
reuse, invalid order/markers/NULs and private debug output. An independent map model
covers 256 traces; a decoded-wire integration releases parent plans and source frames
after merging. Imported default views also support source-lived byte references
from temporary copied records. No engine or session state is automatically applied.

Nine system-info tests add signed/unsigned identifier bit preservation, complete
decimal validation, missing/duplicate critical keys, unknown/raw inert values,
whole-string shared syntax checks, all forbidden control bytes, exact byte/field
budgets and big-info wire bounds. Debug/errors expose no remote text. A gamestate
wire-to-snapshot integration releases original sources before interpreting system
metadata. This performs no session admission, local CVar changes or archive mounts.

Seventeen classic ZIP catalog tests add independent directory/local-header and
streaming-descriptor vectors, original/physical order, duplicate names, raw/UTF-8
names, comments, unsupported features/extras, overlap and wide declared totals.
Every fixture prefix, appended byte values, exact host/storage bounds and record
mutations cover final-output rollback and bounded views. One argument and three
process tests cover explicit archive inspection, private metadata-only reports,
human/JSON events and oversized-file rejection before payload reads. Debug/release
inspector cataloged all 21 installed main IWDs / 24,243 entries; counts and size totals
matched Python's ZIP reader. Aggregate [evidence](installed-iwd-catalog-evidence.json)
contains no game values or file names. Payload integrity, extraction, virtual mounts,
pure checksums and asset loading remain open. The additional native filesystem
reference is hash-locked from the read-only client checkout; no local server source
was accessed or changed.

Eleven payload tests add independent stored/fixed/dynamic Deflate vectors, empty
streams, outputs beyond 32 KiB, all short prefixes and appended byte values, false
lengths/CRCs, input mutations, exact host/caller bounds and dirty state reuse.
Checked output survives original archive/metadata/codec/scratch release, with final
storage unchanged on errors. Two additional process tests cover all-entry aggregate
verification, human/JSON logs, false metadata and oversized declarations before
payload allocations. Pinned Deflate/CRC libraries have default allocation helpers
disabled. The debug verifier also checked all seven entries / 844,144 decoded bytes
in the smallest installed main IWD. Release verification checked all 21 installed
main IWDs: 24,243 entries / 3,999,040,612 decoded bytes passed exact lengths and CRCs.
[Payload evidence](installed-iwd-payload-evidence.json) records only aggregate
metadata. Virtual mount policy, native pure checksums and gameplay remain separate
unimplemented gates.

Five virtual-key tests add native ASCII case/slash equivalence, canonical hash
contracts, preserved raw/Unicode bytes, private Debug/errors, every forbidden
control byte, invalid relative names and complete final-output rollback. They cover
every short output, byte/component budgets, a 65,536-byte host-funded key,
source-independent ownership and 2,816 deterministic single-byte mutations. This
does not implement native request sanitation, OS paths, mount/duplicate policy or
physical asset resolution.

Three copied-metadata tests add exact compressed-span routing, declarations after
original archive/catalog release, metadata-only Debug, all short/extra source
lengths, catalog-budget precedence and same-sized corrupted replacement bytes.
The existing borrowed-entry decoder now delegates through the same separate-source
implementation, retaining all stream/length/CRC and final-output rollback coverage.
No file provider, handle/epoch owner or virtual mount index is claimed yet.

Seven mount-index tests add native ordered duplicates, canonical lookup and missing
keys, source-independent final storage/results, sparse archive IDs, empty inputs,
dirty scratch reuse, unused tails and private Debug output. Every short key/slot
capacity and exact input/retention budget cover whole final-output rollback. An
independent ordered map checks 256 traces; a catalog-to-payload integration releases
all original ZIP/key/candidate buffers before looking up and CRC-checking the last
duplicate. Actual file-provider ownership, archive search order, pure policy and
engine asset loading remain open.

Eleven owned-host tests add process epoch exhaustion without reuse, exact resource
budgets, safe missing/nonregular errors, global uniqueness, stale/foreign IDs,
every short image/payload capacity, work limits and out-of-file spans. Real temporary
files cover in-place changes and original handles after pathname replacement.
CRC failures preserve final output and admitted counters. A copied-index-to-owned-file
integration releases all original catalog/key/index storage, reads through its
original epoch and retains checked output after handle closure. Existing inspector
process tests now exercise this six-crate filesystem/codec ownership boundary.
Release verification through the new owner checked all 21 installed main IWDs /
24,243 entries / 3,999,040,612 decoded bytes. The 6,058,270,418 admitted source bytes
match the catalog images plus compressed spans exactly; [owned-I/O evidence](owned-iwd-payload-evidence.json)
records only aggregate metadata. The debug owner separately checked the smallest
archive's seven entries / 844,144 decoded bytes.

Thirteen IWI6 tests add all direct bitmap/BC formats, dense channel layouts,
reverse mips, NPOT blocks, inactive dimensions, cube/volume geometry, sixteen levels,
known wavelet metadata and explicit direct rejection. Every prefix, appended byte,
used/inactive picmip size, host budget and header mutation covers bounded views.
Owned headers and borrowed mip views retain only their documented source lifetimes.
Two process tests add selected-image CRC scope, private human/JSON reporting and
wrong-version failure without partial output. Debug/release inspection checked 6,565 native
headers / 6,540 direct layouts / 25 explicitly undecoded wavelet headers across all
21 installed main IWDs. [Image evidence](installed-iwi6-evidence.json) contains only
aggregate metadata. Seven public format-source hashes now verify locally and in CI;
no game pixels, binaries or upstream runtime implementation are distributed.

Fifteen wavelet tests add all five formats, all codeword suffixes and escape values,
bounded raw/bit cursors, signed coefficient/parity/wrap vectors, correlated color,
private parent deltas, multi-level continuity, rectangular raw prefixes, cube faces,
source-independent output views, every stream truncation and short caller capacity,
independent budgets, shape rejection and entropy mutations. The process inspector
now reconstructs wavelet pixels and rejects CRC-valid header-only images without a
partial report. Three numerical catalogs reproduce from locked public source;
their GPL license and attribution are retained alongside the fresh Rust runtime.
Debug and release inspection reconstructed all 25 installed wavelet images / 9,349,793 pixel
bytes. A separate string-bit Python decoder matched every per-archive SHA-256 digest,
26,876,561 consumed bits and 3,359,580 encoded payload bytes. [Wavelet evidence](installed-wavelet-evidence.json)
contains aggregate metadata only. This proves the inspected data path, not rendering,
whole engine content admission, GPU resources or playable joining.

Twelve fastfile tests add independently encoded stored/dynamic-Huffman streams,
explicit unsigned PC version-5 selection, all nine block IDs, empty containers,
exact metadata/body lengths, checksum/truncation/tail/dictionary failures, every
short caller capacity, independent host budgets, wide inert block totals, state
reuse, source-independent output lifetimes and deterministic mutations. Two process
tests separate container integrity from asset interpretation and verify private
human/JSON reporting and failure atomicity. Nine immutable public format-source
hashes verify through a shared bounded audit helper. Debug and release inspection
matched all 76 installed fastfiles against independent Python zlib/size/SHA-256
checks: 2,642,982,057 encoded bytes / 4,764,264,952 decoded bytes, with no compressed
tails. [Container evidence](installed-fastfile5-evidence.json) records aggregate
metadata only; no asset graph or renderer is claimed.

Thirteen zone-root tests add null/empty/raw strings, all native type codes, inert
skipped records, biased packed descriptors, a 70,000-null-string catalog, independent
physical/virtual alignment, zero-count allocation behavior, every prefix truncation,
host/scratch/final capacities, alias/type/count failures, virtual overflow, output
rollback/retry/tails, cooked lifetimes and mutations. Two process tests inspect a
real published catalog with an independent string hash and prove that zlib-valid
invalid roots emit no report or completion event. Fourteen container/zone source
hashes verify. Debug/release comparisons matched all 76 installed root catalogs:
32,169 string slots (32 null), 486,555 cooked bytes and 139,101 asset records, including
the two inert non-inline sound-driver fields. Every cooked-string SHA-256, type/link
distribution, prefix extent and full-container hash matched independent Python.
[Directory evidence](installed-zone5-evidence.json) contains aggregate metadata only. Child
assets, serialized targets, registry resources and playable rendering remain open.

Seventeen identity-state checks add fourteen integration tests, local generation
exhaustion, exclusive-owner compile-fail coverage and a real UDP joint-commit test.
They cover explicit baseline initialization, last duplicate wins, exact wrapping
successors, gaps/duplicates, full sequence bits, independent budgets/capacities,
late rollback, cancellation, replacement epochs, dirty workspace reuse, unchanged
tails and source-free retained records. A 256-trace ordered-map model checks
thousands of batches. Reordered malformed/valid UDP frames prove that wire failure,
staging failure and joint cancellation preserve the correct frame/identity owners;
joint commit retains data after all source/work buffers are cleared. Fourteen
locked client-reference hashes verify without reading a server checkout. Whole
ordinary-message identity dispatch and remaining session/engine resources are open.

Eight new independent loopback tests exercise the reusable network crate: one
source socket across challenge/connect/retries/handoff, exact cookie/protocol/qport
fields, inert owned refusal data with socket closure, oversized-datagram rejection,
other-peer filtering, last-slot receive quotas, expiration before a delayed `run`,
silent-peer expiry with retained work counters, and pre-I/O budget/endpoint rejection.
No ordinary gameplay server or complete game channel was used for this checkpoint.

Ten subsequent real UDP tests establish ordinary-channel handoff on the original
socket, qport bits/direction, initial sequences, empty messages, paced fragments
and exact-multiple terminators, interleaved receives, cancellation/I/O-failure
reservation, malformed/stale/reordered traffic, reassembly capacity and explicit
discard, auxiliary marker/qport checks, and expired waits without charged work.
One compile-fail documentation test establishes that receive data cannot escape
the scoped callback. Shared transport logic is consumed at the existing immutable
revision, without source changes to the server repository. Two more client research
files were hash-verified locally and in that checkpoint's hosted source job.

Eight command-history tests add native byte-cleaning/ownership/privacy checks,
valid/invalid caller storage, exact command-length and pending-window boundaries,
the full 128-entry baseline, check-before-commit acknowledgement, unsent/future/
regressing numbers, repeated reuse across every 1–128-slot capacity, and exhaustion
at the positive signed sequence bound. Three additional client source files are
hash-verified locally and in that checkpoint's hosted source job. No
server source files or server worktrees were edited for these client checkpoints.

Eight control-frame tests add independent idle/keyed wire vectors for both header
profiles, source ownership, every insufficient output prefix, a complete 128-command
window, retained-key/header boundaries, whole-prefix deferral and compression
expansion without truncation. A further UDP integration test rejects wrong protocol/
cookie contexts before sequence reservation and verifies a fragmented complete
control frame on the original socket. Marking the included command prefix sent is
separate from construction and from validated peer acknowledgement.

Eight payload tests add explicit profile mapping, independent initial/keyed server
vectors, native padding, key selection without commit, decoded-byte opacity,
unsent/future/regressing numbers, every short scratch/output prefix, incomplete/
empty/oversized input, sequence bounds and reserved compression symbols. Two UDP
tests establish original context use and rejection of unknown original protocols;
one compile-fail example prevents history release during decoded-view inspection.
All caller decoded output and history remain unchanged on decoding errors; private
scratch may be modified and is never consumed as a result on failure.

Nine service-control tests cover logical EOF/padding, complete normalized records,
duplicates/order/empty values, maximum big strings with shorter retained keys, plan
ownership without automatic commit, a complete 128-command window, every record
truncation, scratch/output capacities, quotas, invalid sequences and oversized strings.
A UDP round trip sends a real control frame, rejects a malformed acknowledged service
stream without freeing client history or exposing partial command output, then
validates the next complete message and explicitly commits its acknowledgement.
Transport receipt and host-validated message sequence remain distinct in that test.

Eight retained-history tests cover exact/dirty storage geometry, cancellation,
non-power-of-two slot reuse, retention more than 128 entries back, full big strings
with shorter keys, duplicate/older records, gaps, profile/message order, outgoing
acknowledgement failure cancellation and NOP-only message advancement. A further
UDP round trip stores a normalized server command and uses its exact key/sequence
plus the validated message acknowledgement to produce the next decodable client
control frame. Core preparation does not mutate state; its commit is infallible
under the exclusive preflighted borrows.

Eight auxiliary-body tests add independent native-shaped ACK/data vectors, exact
maximum size, every short output capacity, payload privacy, all flag/window-hint
bits, range counts/domains, truncation and exact suffix validation. A UDP test
checks qport/family admission before parsing without automatic ACKs, channel
reservation or window changes. The neutral crate has no dependencies. Thirteen
pinned client-reference hashes pass locally, including both reliable source files;
the historical server-reference entries were preserved without accessing server
worktrees. Temporal ACK validation and complete stream transport are not yet covered.

Ten receive-window tests cover transactional construction/preparation, caller
budgets, dirty storage/privacy, independent streams under reverse arrival and
repeated modulo reuse, full binary fragments, conflicting duplicates, gaps,
incremental reads, empty-fragment work limits, selective ACK pagination, maximum
signed-short geometry and signed sequence exhaustion without wrapping. A further
UDP test parses exact bodies before retaining reordered data, rejects malformed
lengths, ignores conflicting retransmissions and drains the original byte stream.
Its outgoing queue is empty and admits only ACK zero with no selective ranges;
it does not exercise a general outgoing ACK validator or retransmission runtime.

Eight application-framing tests add independent exact length/body vectors, output
atomicity, every three-chunk partition including partial length headers, retained
frame borrows, concatenation with one-frame consumption, zero-body framing,
dirty-storage reuse/privacy and terminal rejection of negative/over-budget lengths.
A native-size 1 MiB message drains incrementally through three reordered slots,
without requiring the receive window to hold the whole message. A compile-fail
example prevents body-buffer reuse during complete-frame inspection. Another real
UDP test assembles a complete application body after out-of-order one-to-three-byte
length-header pieces. Neither framing nor this fixture executes an application opcode.
The process-test responder now waits for explicit child completion rather than a
fixed sleep. Its unsupported-profile probe uses a practical one-second deadline;
deterministic core tests continue to check exact retry/expiry boundaries.

Twelve outgoing-window tests cover transactional storage and enqueue geometry,
exact binary fragment sizes without empty terminators, separate queue/flight spans,
actual-send admission, cancellation, stable retransmissions, unsent ACK holes,
late-invalid selective ranges, overlap/duplicate counting, retained selective bytes,
paired receive/ACK cancellation, maximum signed fields and sequence exhaustion.
An independent 50,000-byte loss/reorder/duplicate model repeatedly reuses seven
outgoing slots and must reconstruct the exact stream. This establishes model
state behavior; the following tests extend the evidence to actual UDP I/O.

Four auxiliary UDP-send tests add exact independent data/ACK vectors on the
original socket/qport, successful-I/O-only admission, expired sends without sent
authority, stable retries followed by validated peer consumption, partial-fragment
selective ACKs, host-controlled window hints, and profile/metadata rejection before
I/O. Original protocols 6 and 22 cannot emit this reviewed auxiliary profile merely
because a challenge advertises protocol 18. Saturating send totals/subset counters
are checked; no ordinary sequence is reserved. That checkpoint covers explicit
individual sends; the following tests extend it to paced retry/ACK scheduling.

Eight scheduling tests add exact-clock configuration and timer bounds, paced
round-robin retry identity, combined fragment/timer cancellation on drop or callback
failure, periodic ACK progress under wholly selective suppression, no long-pause
catch-up burst, bounded validated force-ACK requests, monotonic/overflow checks
and ACK pagination/reset. A real UDP test couples scheduled I/O failure/success
to both leases, preserves original retransmission bytes, consumes a validated peer
ACK and sends an idle ACK. Scheduled profile rejection also preserves both owners.
Host ticks in the UDP fixture are scripted; a continuous gameplay session and live
ordinary-peer timing interoperability have not been tested.

Six endpoint tests add atomic construction across all buffers/timers, combined
receive/ACK/force-clock admission, cancelled invalid force requests, bounded
failure-work records, terminal framing failure, partial prefixes, empty-fragment
quotas and exact boundaries between concatenated messages. A two-endpoint model
exchanges a 5,000-byte body and a reply under loss, duplicate delivery and receive
backpressure. Two framed-enqueue tests cover binary prefix/body boundaries and
capacity failure before any partial prefix write. One UDP fixture checks combined
invalid/valid packet admission, complete body delivery and consumed ACK emission.

The local workspace contains a pre-existing edit to the challenge process test;
it is excluded from this checkpoint. Hosted CI checks the committed version.

Six connection-owner UDP tests add original socket/profile/endpoint coupling,
deadline failure without data admission, exact framed exchange, wrong routing,
body/ACK rejection, scoped ordinary/connectionless callbacks with hidden debug
results, independent ordinary sending, actual receipt time after delayed I/O,
expired waits without work or timestamp refresh, terminal framing failure before
more socket operations, and profile/storage/timer failures preserving supplied
buffers. Counters include work before failure and count terminal transitions once.
The ordinary sender wrapper exposes no external auxiliary windows or schedulers.

Seven body-reader tests add independent cached-bit vectors around 128 KiB and
beyond 1 MiB, a differential mixed primitive trace against shared readers, large
byte copies, shared big-string normalization, exact input/output failure atomicity,
invalid bit widths, missing terminators, concrete shared entity/client index forms
and joint cache/index rollback. The adapter allocates nothing and changes storage
views only; pinned shared code still owns every field's wire decoding.

## Initial foundation

Date: 2026-11-01. Local host: Windows x86_64-pc-windows-msvc, Rust 1.95.0
(`59807616e`, LLVM 22.1.2). The existing server Cargo target directory was reused;
no client-specific target root or alternate build cache was created.

| Check | Local result |
| --- | --- |
| Rustfmt | Pass |
| Workspace/all-target Clippy, warnings denied | Pass |
| Debug tests | 16 pass: 8 wire/parser, 7 real UDP/process, 1 OS argument-encoding |
| Release tests | Same 16 pass |
| Rustdoc, warnings denied | Pass |
| Source/crate/dependency policy | 18 source files, 2 crates; largest source 220 lines |
| Pinned reference hashes | 7 files across 2 snapshots pass |
| Local cross-repository query | Pass; 1 accepted reply, 0 rejected packets |

Wire tests cover exact requests, every insufficient output prefix, required-field
truncation, malformed input, duplicate/stale correlation, byte encodings, numeric
overflow and normal-info capacity. Process tests cover wrong endpoints, stale and
oversized datagrams, deadline/receive-budget exhaustion, separate report/log output,
valid JSON-only failure logging and non-Unicode OS argument rejection.

The cross-repository check used the locally available query-only server executable,
identified by SHA-256 in [the evidence record](local-discovery-evidence.json).
It does not certify a server source checkout or ordinary CoD4X interoperability.
No native gameplay server, game renderer or complete join was tested. Auth-required
servers remain outside the selected no-auth compatibility baseline.

Windows/Linux hosted CI and source-verification jobs are configured but have not
run for this initial local commit. Linux behavior is not claimed from Windows tests.
