# Live player-scalar prefixes

`player_scalars::PlayerScalarPrefix::parse_live` decodes the complete native scalar
prefix of a live player state for an explicitly selected baseline profile. It takes
message-time bits and an optional immutable `PlayerScalars` prior. A wrong-profile
prior rejects before reading; `None` supplies zero words. Private candidate words
and a cloned reader make the entire prefix atomic, including cached bits.

The prefix starts with the native origin/velocity mode bit and an eight-bit field
count read through the bit cursor. Counts 0..=141 are valid; this is a prefix count,
unlike the HUD's inclusive last-field index. Every omitted word retains the full
prior value. All 141 fields use a reproduced catalog and closed semantic keys:

```rust
use cod4xr_client_core::{BaselineProtocol, player_scalars::{PlayerScalars, word}};
let scalars = PlayerScalars::zero(BaselineProtocol::Cod4x21);
let origin_x_bits = scalars.word(word::motion::ORIGIN_X);
```

Keys are grouped into motion, view, weapons, effects, events and session modules.
Their constructor is private; they are catalog identifiers, not native C offsets
or wider engine/player/resource IDs. `schemas/player-scalar-words.json` records
reviewed source-to-semantic names. `tools/import_player_scalar_table.py` reproduces
the six namespaces, key inventory and field table from locked public sources.
Every generated source stays below the project's file-size target.

Native change hints remain explicit. Six unconditional fields omit the outer
changed bit but still carry their normal value marker/payload. Eight live fields
(origin/velocity components, bob cycle and movement direction) use zero references
when the mode bit selects transmission. They still read the outer changed bit.
This forcing applies only inside the serialized prefix. Hint one keeps ordinary
conditional behavior. Zero-valued unconditional fields inside a prefix clear the
prior; fields outside the prefix retain it.

Shared scalar values now also cover absolute signed-short view angles without a
zero marker, raw binary32 XOR and signed-short integer XOR with masked references
and sign extension. Existing signed-byte and delta-angle forms share those same
reviewed helpers. Normal flags, times, ordinary compact floats and positive integer
widths retain their prior rules. Native player origins use raw XOR in both reviewed
profiles, separately from entity/HUD legacy map-center coordinates.

`origins_transmitted` and `needs_prediction` retain native mode provenance. A false
transmission mode causes the containing native live reader to request a
command-time prediction lookup. This prefix decoder does not perform that lookup,
invent predicted coordinates, apply fallback or admit a reconciled player state.
Following stats/ammunition/objective/HUD/model sections remain unconsumed. Full
player-state ownership must resolve prediction or choose explicit native fallback
before whole-state admission; snapshot history/message ACKs remain separate.

A read-only installed-image audit matched all 141 native field names, widths and
change hints against the locked public schema, with unique aligned offsets. It
also checked the live caller flag, forced-reference rule, unconditional fields,
signed integer masking and angle conversion.
[Audit metadata](player-scalar-reader-evidence.json) records image identity and addresses without
binary bytes, disassembly, positions or inventory data.

Twenty-four integration tests cover independent framing/time/angle/negative-short
vectors, both profiles, all keys, full zero prefixes, all forced/unconditional
fields, omitted retention, raw/compact/zero floats, signed masking, flags/events,
wrapping/absolute times, following/cached fields, all invalid counts, every tested
truncated full prefix, late reference failure, source-independent ownership,
private Debug output and 128 modeled prefix traces per profile.
