# Initial entity baselines

`gamestate::entity_baselines::EntityBaselineDecoder` parses initial
`svc_baseline` records from a zero reference, including the preceding ordered
entity index. Protocol 21 uses explicit raw-coordinate context; protocol 6 now has
[checked map-center context](legacy-baseline-coordinates.md) through
`with_legacy_map_center`. The original protocol-6 constructor rejects missing context.
Discovery advertisements do not select a profile.

The locked public client's `src/msg.c` supplies eighteen entity field lists and
their writer rules. All lists begin with the same changed/8-bit entity-type field;
the decoded type selects subsequent field order, with unknown types using list 17.
List lengths are 58, 59 or 60, so the last-changed-field count uses six bits.
Hint 2 omits an outer changed flag. Other hints retain that flag. Zero-reference
decoding uses the same wire order as the native writer without copying its runtime.

The decoder reuses the pinned shared primitive and ordered-index codecs. Its new
baseline rules compose those primitives for integer residual bits followed by byte
fields, signed movement directions, positive/negative zero, compact integer floats,
full float words, angles, single-bit/full flags, relative/absolute time, ground
references, event bytes and protocol-21 raw coordinate words. It deliberately does
not treat later entity records as zero-reference baselines. The shared field reader
now accepts explicit prior-word/time context for [individual deltas](entity-deltas.md);
initial baseline decoding continues to supply zeros and preserves its exact vectors.
Angles convert a signed 16-bit turn to degrees, including negative values in the
high half of the wire word, as confirmed by the reviewed native reader audit.

Raw float payloads, including NaNs and large coordinates, retain their exact bits.
They are inert wire state, requiring separate engine geometry/resource validation.
No pointer casts, native struct packing, peer-directed allocations or fixed engine
entity table are involved. `EntityBaseline` owns sixty semantic words plus its
widened entity number. `BaselineWord` names common fields and the type-specific
union words, independently of a C ABI. Debug output contains only number/word count.

The host owns one decoder per gamestate and explicitly resets it for a new stream.
A parse clones both message and shared index state, validates the full record and
then commits both together. Truncation, invalid order/namespace, removal records or
invalid changed-field counts preserve both originals. Returned baselines own their
values and permit source-frame release. Nothing applies a baseline to the engine.

Generated tables are split into six focused files under
`crates/client-core/src/entities/tables`; compatibility import paths remain
available for existing gamestate callers. Reproduction verifies the
normalized hashes of both `src/msg.c` and `src/q_shared.h` from `upstream.lock.json`,
checks all list shapes, reviewed encodings and unique semantic aliases, then uses
the pinned formatter. The alias map was reviewed against the native entity and
union declarations. To check a read-only public client checkout:

```powershell
python tools/import_entity_baseline_tables.py C:\Users\Ethan\source\cod4x-client-reference
```

An explicit `--write` regenerates the client tables. Hosted source CI runs the
same reproduction check after verifying locked reference sources. No server
source files, shared revision, game executable or asset data changes are required.

Ten new tests cover independent physical field/header vectors, all eighteen full
layouts and unknown-type fallback, unconditional fields, compact/raw/negative-zero
floats, angles, flags, time/ground forms, signed values, large coordinates, raw NaN
bits, source-independent ownership, large input offsets, following byte/bit fields,
every fixture truncation, namespace exhaustion, index order/reset, malformed counts,
removal rejection, private debug output and rollback of existing cache/index state.
These are source-grounded conformance fixtures, not captures from a live native peer.

Complete protocol-21 gamestate constituents now have a separate transactional
plan. Reviewed native constants and effective default/wire merging have separate
content/snapshot APIs. Complete legacy gamestates, other enclosing-message admission,
complete snapshots, engine interpretation and ordinary-server
interoperability remain open. A baseline parse alone does not establish a playable connection.
