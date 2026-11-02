# Snapshot and entity validation

These records describe historical checkpoints. Remaining-gate statements apply to
that checkpoint; see [current validation](../validation.md) for the latest status.

Eight original-owner snapshot ACK cases cover standalone cancellation/key-copy/
slot-reuse semantics, repeated/zero/unsent/regressing prefixes, both profiles through
actual payload decoding, cooked-data host inspection, source/workspace/sample
lifetimes, explicit fallback and private Debug. Shared-header, decode-workspace,
late-service and prediction failures preserve both final buffers and original key
history. Two compile-fail examples protect original-owner leases. Existing ACK and
payload behavior remains covered after sharing the header/prepared-key code; see
[snapshot acknowledgements](../snapshot-acknowledgements.md).

Six decoded snapshot-payload cases exercise both profiles through actual shared
compression/obfuscation and payload decoding. They cover positive retained-key ACK
metadata, actual profile/message mismatch before service work, late unsupported
services, failed prediction, explicit sample/fallback, retry, owned lifetimes and
private Debug. Successful and failed parsing leave outgoing ACK/key state intact;
only a separate explicit owner commit advances it. Joint connection history/ACK
admission remains open; see [snapshot payloads](../snapshot-payloads.md).

Nine snapshot-only service cases add independent framing/EOF vectors, native
padding, cached-bit continuity, owned cooked references, all tested truncations,
unsupported prefix/tail services, exact record budgets, missing EOF/snapshot,
cancellation and failed/successful prediction selection. A compile-fail example
protects the enclosing reader lease. Combined service routing and payload/ACK/
history ownership remain open; see [snapshot services](../snapshot-services.md).

Twenty-two complete snapshot wire cases cover independent physical framing, both
profiles, full/delta sparse merges, explicit coherent message/epoch references,
following services/cached bits, baselines and unused-source inspection, every
tested truncation, late client errors, independent budgets/capacity, prediction
selection/fallback/retry, drop cancellation and private/owned results. Forty cooked
plans and distance 255 demonstrate caller-controlled retention beyond native rings.
Three compile-fail examples protect reader/output/reference leases. All existing
player-state/stream cases remain intact after reusing their private pending-player
and sparse-walk implementations. Host history and enclosing message/ACK admission
remain open; see [complete snapshot wire bodies](../snapshot-bodies.md).

Seven native snapshot-header cases cover physical time/distance/flag vectors,
every nonzero delta distance and flag pattern, all truncated headers, cached-bit
preservation, following bytes, owned/private metadata and immediate/straddling
player-state handoff. Reference history and full snapshot/message admission remain
open; see [snapshot headers](../snapshot-headers.md).

Fifteen complete live player-state cases cover native section composition, both
profiles, independent empty-state vectors, following byte/cached-bit operations,
exact/wrapping prediction keys, all eleven replacements and explicit prior fallback.
Every tested mixed-body truncation, late current-HUD failure, invalid commit choice
and mismatched sample preserves the original reader. Prior/source/sample lifetimes,
context changes and private formatting are covered. Two compile-fail examples
protect the exclusive reader lease and consuming commitment. Prediction production,
reference epochs and whole snapshot/message admission remain open; see
[live player-state transactions](../live-player-state.md).

Twenty-four live scalar-prefix tests add independent count/time/angle/signed-short
vectors, both profiles, all 141 typed keys, full prefixes, all forced and
unconditional fields, omitted references, float forms, signed masking, flags,
events and wrapping times. All invalid counts and tested truncated full prefixes
preserve prior words and both cursors; late compact-reference failures, profile
binding, owned lifetimes and private Debug formatting are covered. One hundred
twenty-eight modeled traces per profile compare all words, including forced-zero
and unconditional behavior. All 141 native names/widths/hints match the locked
schema; [audit metadata](../player-scalar-reader-evidence.json) also verifies the live
caller and prediction-request boundary. Whole player-state/prediction/snapshot
admission remain open; see [live player scalars](../player-scalars.md).

Twenty-six player-HUD tests add independent framing vectors, all forty field rules,
both profiles, all scale indices and alpha forms, compact full-prior/wrap behavior,
raw floats, modern origins, explicit legacy centers and wrapping message times.
They cover both 31-slot arrays, exact first-free tail retention, inactive words used
by later deltas, owned lifetimes, private Debug output, every tested truncated full
section, all invalid last-field indices and failures in the last current element
after valid archival changes. One hundred twenty-eight modeled array traces compare
prefix updates and native clearing independently. The reproduced catalog matches
all forty native offsets/widths/hints; [audit metadata](../player-hud-reader-evidence.json)
also binds the precise scale constants and numeric branches. Scalar/prediction and
complete snapshot admission remain open; see [player HUD](../player-hud.md).

Twenty-one player-array tests add native stats/ammunition physical vectors, both
profiles, signed/unsigned boundaries, absolute updates versus XOR, high mask bits,
first/last wire slots, all groups, explicit zero clearing, empty masks, omitted
retention and raw model-table replacement. All tested truncations preserve cached
reader bits and prior ownership, including the last clip flag and last model byte.
Profile mismatches reject before reading; cooked values outlive inputs and Debug
output excludes inventory/model contents. One hundred twenty-eight modeled traces
per section/profile compare every retained value.
[Native audit metadata](../player-array-reader-evidence.json) binds group counts, signed extension and model
bytes to the reviewed reader. Scalar/prediction decoding remains open;
see [player arrays](../player-arrays.md).

Fourteen player-objective section tests add both explicit profiles, independent
physical status vectors, raw reserved statuses, whole-set/entry/word retention,
all six fields, prior XOR and float forms, zero clearing, cached/following fields,
every truncated full-section prefix, late compact-reference rollback, profile
binding and source-independent ownership. Debug output hides all objective words.
One hundred twenty-eight modeled traces per profile compare every slot/status/word
against independent arrays. The reproduced catalog matches all six native offsets,
widths and hints; [audit metadata](../player-objective-reader-evidence.json) records
the parent sixteen-slot loop and leaf reader. Remaining player-state sections and
whole snapshots remain open; see [player objectives](../player-objectives.md).

Thirteen complete client-state substream tests add native end-bit framing, client
63, cached/following fields, all sixty-four clients with exact header budgets,
prior/zero references, sparse merge/removal/tail rules, full source validation,
independent budgets and late dirty-output rollback. Cooked-only lifetimes, reusable
source views, 128 ordered-map traces per profile and forty results retaining 2,560
states exercise owned storage beyond the native global ring. A compile-fail example
protects borrowed cooked cells. Entity/client substreams share their closed ordered
validator and merge cursor; existing entity-stream tests continue to pass.
[Client-state substreams](../snapshot-client-streams.md) remain separate from complete
snapshots, player-state sections, host reference epochs and message admission.

Fifteen client-state record tests add all twenty-four fields, physical byte vectors,
both baseline profiles, prior XOR and omitted-word retention, raw packed names,
signed-zero/raw/compact floats, all sixty-four client indices, separate end framing,
owned lifetimes, cancellation, invalid references/counts/order, every truncated
full-record prefix with cached bits and 128 modeled traces per profile. Two
compile-fail examples protect leased reader/index owners. The reproduced catalog
also matches all twenty-four offsets, widths and hints in the reviewed native
image; [audit metadata](../snapshot-client-state-reader-evidence.json) records that
comparison. Complete client-state merging now has separate coverage; player-state
decoding and full snapshot admission remain open; see
[client-state records](../snapshot-client-states.md).

Fifteen complete entity-substream tests add native terminator/framing bytes, maximum
index termination, valid existing cached bits and following fields, full sparse
merge rules, previous/baseline/zero precedence, removals, legacy context, all tested
truncations, late malformed records, input source/order failures, independent host
budgets/capacities and unchanged final tails. Cooked-only lifetimes/source reuse,
128 ordered-map traces, 512 retained entities and six simultaneous results with
3,072 entities exercise owned storage beyond the native global ring. A compile-fail
example prevents overwriting borrowed cooked input cells. Shared codecs moved to
`entities`, preserving compatibility imports and source-table reproduction.
[Audit metadata](../snapshot-entity-reader-evidence.json) binds stream rules to the
reviewed native reader. Complete snapshots, player-state sections and native peers
remain open; see [entity substreams](../snapshot-entity-streams.md).

Eleven nonzero legacy-delta tests add explicit coordinate-profile ownership, all
short values against prior state, fractional short/full distinctions, all eighteen
target layouts, native signed-wrap/binary32-rounding boundaries, context replacement
and reset, no-delta/removal handling, owned lifetimes, every truncated prefix with
cached bits, cancellation and late invalid-reference rollback. Nonfinite changed
origins reject; full forms additionally check signed-32 prior conversion, while
finite short forms keep native float addition. Existing baseline fixtures now share
one support module. Complete snapshots and native peer traces remain open; see
[legacy entity records](../legacy-entity-deltas.md).

Nine legacy-baseline integration tests and two physical field tests add explicit
protocol-6 map-center context, all 128 short values on each axis, sixteen-bit full
coordinate windows, all eighteen layouts, native half-bias/truncation precision,
signed conversion bounds, nonfinite rejection, context replacement without index
reset, context-preserving reset, source ownership, all record truncations and late
cached-bit rollback. A native x87 audit verified precise addition before integer
conversion, avoiding an incorrect intermediate binary32 rounding. Existing modern
baseline/delta vectors remain covered. [Metadata evidence](../legacy-coordinate-reader-evidence.json)
contains no binary bytes or disassembly. Complete legacy gamestates, nonzero legacy
records and native peer traces remain separate; see
[legacy baseline coordinates](../legacy-baseline-coordinates.md).

Ten entity-delta integration tests and three field tests add nonzero references,
all eighteen target layouts, type changes, signed/XOR/absolute field distinctions,
message-relative time, owned states/removals, wrong-reference rejection, cancellation,
all record truncations, cached-bit rollback, compact-float reference bounds and
stream ordering/reset. A 1,024-record trace checks repeated no/full/partial changes.
Two compile-fail examples protect both reader and index owners. A read-only audit
of the reviewed installed native reader identified the narrow integer reference
mask and signed 16-bit angle conversion; the latter also corrects initial baseline
decoding and has new high-angle vectors. The sixteenth client source hash binds the
native entry point. [Evidence](../entity-delta-reader-evidence.json) contains metadata
only. Complete snapshots and native peer sessions remain open; see
[individual delta records](../entity-deltas.md).

Ten entity-baseline tests add independent physical field/header vectors, all eighteen
full protocol-21 layouts plus unknown-type fallback, unconditional-field positions,
integer/float/angle/flag/time/ground forms, signed union values, large/raw coordinate
bits, owned results, large input offsets, following byte/bit fields, every tested
truncation, invalid counts/removal/order/namespaces, explicit reset and existing-cache
rollback. Generated field catalogs reproduce from the same locked client source
hashes locally; hosted CI now includes the same check. No native captures, legacy coordinate support,
nonzero-reference snapshots or engine application are claimed.
