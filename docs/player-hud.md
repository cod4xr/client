# Native player-HUD sections

`player_hud::SnapshotHud::parse` decodes the complete native HUD section with
explicit `EntityDecodeContext`, message-time bits and an optional immutable prior.
It returns privately decoded owned words and commits the reader only after both
archival and current arrays succeed. A prior from another profile rejects before
reading. Same-profile legacy map-center replacement remains explicit and supported;
the current context is recorded even when the section's changed bit is zero.

The outer changed bit gates both arrays, archival first. Each array has a five-bit
count, followed by that many element records. Each record's six-bit value is an
inclusive last-field index: zero means one field, not an empty prefix. Indices above
39 reject. Unmentioned words retain their full prior value. The forty-field catalog
uses semantic keys, not C memory offsets, pointer casts or engine resource handles.

Trailing slots follow the reviewed native rule exactly: clearing begins after the
transmitted prefix, zeros consecutive nonfree elements, and stops at the first
already-free element. That free slot's other words and all later inert slots remain
available as references for subsequent deltas. Clearing every tail slot would change
valid reader behavior. All 31 slots in each array remain owned, including inactive
words; a serialized prefix is not proof of active presentation elements.

Existing shared value rules handle integer masks, ordinary floats, wrapped relative
times, modern absolute origins and explicit-context legacy coordinates. Three more
reviewed native value rules are shared through the same scalar reader:

- Color values preserve/replace RGB and quantize alpha with five bits shifted by
  three. The alpha shortcut maps zero to 255 and every nonzero prior alpha to zero.
- Font scales divide the five-bit value by binary64 ten, add the exact promoted
  binary32 `1.4` constant, then round once to binary32. All 32 coefficient-derived
  goldens are tested; ordinary float intermediates differ for five encoded values.
- Compact previous-position floats use a 12-bit XOR window against the full prior
  biased integer. High prior bits and signed wrapping remain significant. Compact
  forms check finite signed-32 prior conversion, rather than restricting the prior
  to the small window. Raw forms preserve arbitrary bits, including NaNs and -0.

The checked float-to-signed-integer conversion is shared with legacy coordinate
decoding. Existing entity/baseline behavior and narrower ordinary-float rules remain
unchanged. Debug/errors contain only structural categories and bounded counts.
No raw colors, text indices, material identifiers or coordinates are printed.

`tools/import_hud_table.py` reproduces all forty fields from hash-locked public
`msg.c`/`q_shared.h`; hosted source verification runs it. A read-only installed-image
audit compared every native offset, width and hint and checked framing, array order,
tail clearing and numeric branches. The public HUD writer body is insufficient to
establish prefix framing, so the actual reader controls that rule.
[Audit metadata](player-hud-reader-evidence.json) records exact image identity and
addresses without executable bytes, disassembly or game values.

Twenty-six integration tests cover independent framing vectors, all forty fields,
both profiles, all scale values and alpha forms, compact high-bit/wrap boundaries,
raw/signed-zero floats, absolute modern origins, legacy full/short coordinates,
explicit context replacement, wrapped times, all 31 slots, prefix/tail retention,
owned lifetimes, private Debug output, every tested truncated full-section prefix,
invalid last-field indices, late failures across both arrays and 128 independent
modeled array traces. Complete player scalar/prediction decoding, reference history,
whole snapshot/message admission and renderer/resource integration remain open.
Native HUD slot/field widths are wire constraints, not wider engine/UI limits.
