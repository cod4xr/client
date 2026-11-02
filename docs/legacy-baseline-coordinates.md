# Legacy baseline coordinates

`EntityBaselineDecoder::with_legacy_map_center` selects protocol-6 initial entity
baselines using explicit `LegacyMapCenter` context. The original `new(Cod4x21)`
keeps absolute binary32 coordinates; `new(Legacy6)` still rejects missing context.
Discovery advertisements, arbitrary remote variables and a silent zero-center
fallback never choose this dialect. The host owns the current effective map center.

`LegacyMapCenter::new` checks all three supplied native binary32 values and computes
their half-unit bias followed by signed integer truncation. The reviewed native x87
reader keeps the addition precise until conversion. Binary64 reproduces its result
throughout the admitted signed-32 range, including negative integers, values just
below a half and large binary32 values. Rounding the addition to binary32 first
would change some of those biases. Nonfinite or unrepresentable results reject
without returning a partial context. Debug output hides the values.

Native origin encodings `-92`, `-91` and `-90` select X, Y and Z respectively. The
short form reads seven bits and subtracts 64 from a zero reference. The full form
reads sixteen bits, XORs the complete prior map-biased integer, restores the bias
and converts the signed result to binary32. The prior bias is not masked to sixteen
bits. Explicit wrapping arithmetic reproduces the reviewed x86 operations without
C signed-overflow assumptions. With a zero reference, only the low sixteen prior
bits change; the resulting difference fits seventeen signed bits and converts to
binary32 exactly. This is the legacy coordinate representation, not a larger-map
wire extension. Protocol 21 retains its independent raw-coordinate behavior.

Both profiles share the same field catalogs and outer change/value rules. Native
hint-2 fields still omit their outer flag; a normal float field in a type-specific
layout does not become a legacy origin field merely because it stores a coordinate.
All eighteen layouts and unknown fallback retain their existing table selection.
Returned baseline words remain owned inert data requiring engine/resource validation.

`profile()` reports the explicit decoder context. `set_legacy_map_center` replaces
the legacy center between records without resetting ordered indices, supporting
future per-block gamestate context application. It rejects protocol-21 owners
without changing them. `reset` resets ordering and retains the explicit current
center. Each complete parse still clones reader/index state and commits both only
after every field validates. Late coordinate failure preserves the original byte
cursor, cached bits and prior index. Parsing or changing context does not apply a
map, configure engine variables or acknowledge a complete server message.

Nine integration tests cover native context precision/truncation and signed bounds,
nonfinite rejection on every axis, explicit profiles, center replacement without
order reset, context-preserving stream reset, all 128 short values on all axes,
full positive/negative/large-center windows, all eighteen layouts, copied ownership,
following bytes/bits, every record truncation, late cached-bit rollback, signed
wrapping extremes and private debug output. Two private field tests use independent
physical byte vectors for short/full forms and every short input prefix. Existing
protocol-21 baselines and nonzero delta checks continue to pass.

The actual public reader is pinned in
[msg.c](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L885),
with its [writer](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L441).
The earlier commented 24-bit reader is not the active sixteen-bit profile.
[Native audit metadata](legacy-coordinate-reader-evidence.json) records the exact
previously reviewed installed image and coordinate-reader addresses, including its
precise half-addition and integer truncation. No executable or disassembly is stored.

Complete legacy gamestate framing, effective `CS12` context updates inside those
plans, nonzero-reference legacy deltas and native peer traces remain separate work.
The complete gamestate/reliable wrappers continue to require protocol 21. This
record codec does not establish a playable or broadly interoperable client.
