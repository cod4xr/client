# Native player-state array sections

`player_arrays` provides owned codecs for three native player-state sections:
`PlayerStats`, `PlayerAmmunition` and `WeaponModels`. Each parser takes an explicit
baseline profile and optional immutable prior value. `None` supplies zero state;
a prior from another profile rejects before any read. Each complete section is
decoded privately, then both reader cursors commit together. Failure preserves the
prior and original cached bits. Results outlive network input and prior ownership.

Stats have one changed bit followed, when set, by a five-bit mask. Slots 0..=2
carry absolute signed 16-bit values, slot 3 an unsigned six-bit value, and slot 4
an unsigned byte. A changed section with an empty mask is valid. Slot meanings and
gameplay range policy are intentionally outside the wire codec.

Ammunition uses absolute signed shorts, unlike XOR scalar deltas. Stored ammo has
one overall changed bit and four optional sixteen-entry groups. Clips have eight
group bits with no overall changed bit. A present group carries a sixteen-bit mask
and one signed short per set bit. Empty masks are valid. Omitted entries preserve
the full prior word; explicit zero values clear it. The private group helper checks
its static array size at compile time, so incomplete groups cannot be ignored.

The native C player struct has 128 stored-ammo entries, but this wire updates only
the first 64. `PlayerAmmunition` is explicitly that wire projection plus 128 clips:
applying it must preserve higher engine-owned stored entries. Out-of-projection
access returns `None`; the codec never narrows wider engine IDs or clears unseen
inventory. Counts and widths describe the baseline wire, not engine-wide limits.

Weapon models carry one changed bit and, when set, a complete 128-byte absolute
table. Zero and high bytes remain raw. An unchanged table retains every prior byte;
there is no per-entry mask, string framing or asset resolution. Debug/error output
for all three section types contains only structural metadata.

Stats and ammunition precede the objective and HUD sections in native player state;
weapon models follow HUD. These parsers do not skip those intervening sections or
admit a complete player state, snapshot or message ACK. Profile checks also do not
replace connection, map/gamestate generation and snapshot-sequence binding.

The locked public `msg.c` writer and `q_shared.h` array declarations are paired with
a read-only native reader audit. It verifies signed extension, mixed bit/byte
cursors, group counts and the complete model-table loop.
[Audit metadata](player-array-reader-evidence.json) records exact image identity and addresses
without binary bytes, disassembly or game values.

Twenty-one integration tests cover independent physical vectors, both profiles,
signed extremes, high mask bits, first/last entries, absolute updates, zero clearing,
empty masks, omitted retention, raw model bytes, cached/following fields, every
tested truncation including late clip/model bytes, profile checks before reading,
owned lifetimes, private Debug output and 128 modeled traces per section/profile.
Player scalar/prediction and HUD decoding, full reference history and complete
snapshot/message admission remain open.
