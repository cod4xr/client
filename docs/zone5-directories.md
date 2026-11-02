# PC zone root directories

`zone5` catalogs script strings and asset records from a `CheckedFastfile5`.
Container framing, zlib integrity and decoded length must already have passed.
The directory layer validates only its root/prefix schema. Child asset bodies,
offset targets, insertion tables, engine registries and GPU resources remain open.

The physical serialized body begins with sixteen bytes: signed string count,
32-bit string-array link, signed asset count and 32-bit asset-array link. Root
links support null with zero count or the following-data marker. Negative counts,
inconsistent roots and other root links fail explicitly. A counts-only inspection
checks host count budgets and the minimum arrays before callers allocate metadata;
its success does not validate strings or asset records.

The pointer array for all script strings precedes their literal bytes. Null entries
consume no literal data and remain distinct from actual empty NUL-terminated strings.
Following links consume exact raw bytes through the terminator, without UTF-8
coercion, percent normalization or name interpretation. Offset/insert script-string
aliases need the future relocation layer and currently fail explicitly.

All eight-byte asset records follow those literal strings physically, before child
payloads. Each record contains a reviewed type code and an inert link word. All
thirty-three native labels are explicit; unknown codes fail. Serialized order and
duplicates are preserved. Link descriptors distinguish null/following/insertion and
biased packed offsets, but no target is followed or granted storage authority.
Subtracting one precedes four-bit block / twenty-eight-bit offset interpretation;
raw block codes and offsets remain unvalidated metadata until per-type dispatch.
Debug output shows link classes, without words or offsets.

The reviewed loader skips sound-driver globals. Two installed records of this type
carry offset-like words outside the declared block extent. Unconditionally resolving
every asset field would reject valid installed content. This catalog preserves those
words privately and exposes inert descriptors only when requested. A typed loader
must apply type-specific rules before any target/alias interpretation.

Physical stream position and logical virtual-block position are separate. Pointer
arrays and literal strings consume both; aligning the virtual block before an asset
array consumes no physical padding. Even a zero-count nonnull asset allocation aligns
logically, while a null asset array does not. The complete accounted virtual prefix
must fit its checked block declaration. These declarations remain inert memory
metadata, not actual owned resource spans.

`PreparedZone5Directory` has three stages:

1. `counts` returns bounded root metadata for exact private slot allocation.
2. `parse` checks every prefix field and stages metadata in caller-owned scratch.
   It returns exact cooked-byte requirements after string/record/virtual validation.
3. `publish` checks every final capacity before copying any cell or byte. Failure
   preserves all final storage and tails; success borrows final storage only.

Callers may retry publication with larger storage. Published strings can outlive the
source container, work arrays and plan, while copied asset records can outlive the
catalog. Scratch is unspecified after preparation failure. The pure crate allocates
nothing and owns no files or global string/asset arena. Counts, string bytes, physical
prefix bytes and actual caller capacities have independent policies. A synthetic
70,000-null-string catalog exercises wider host storage; typed legacy reference
widths remain a separate future asset-loader boundary.

`--inspect-zone-directory PATH_TO_FF` adds directory inspection after complete
fastfile validation. The host allocates exact checked metadata/string capacities,
publishes a real cooked catalog and reports counts/type histograms/link classes and
a SHA-256 over the cooked strings including terminators. Source/compression scratch
are freed before directory allocations. No strings, file names, asset names, pointer
words or offsets enter operational logs or reports. Root-directory interpretation,
unverified child assets and unverified gameplay are reported independently. Existing
`--inspect-fastfile` still validates only its container layer.

Thirteen library tests cover null/empty/raw strings, all kind labels, inert skipped
words, packed descriptors, wider catalogs, physical/logical alignment, empty-root
allocation behavior, every truncation, independent budgets/capacities, unsupported
aliases, unknown types, virtual overflow, publication rollback/retry/tails, cooked
lifetimes and deterministic mutations. Two process tests add an independent string
hash, private human/JSON reporting and zlib-valid malformed roots with no partial
report. Debug/release comparisons matched all 76 installed root catalogs: 32,169
string slots (32 null), 486,555 cooked bytes and 139,101 asset records. Every published
string SHA-256, type/link distribution, physical/virtual extent and full-container
hash matched independent Python. The largest consumed body prefix was 103,095 bytes.
[Aggregate evidence](installed-zone5-evidence.json) includes no strings or names.
These checks do not imply typed assets or playable rendering.

Primary immutable references are the
[root loader](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Game/IW3/ContentLoaderIW3.cpp),
[string/link handling](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Loading/ContentLoaderBase.cpp),
[stream accounting](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Zone/Stream/ZoneInputStream.cpp)
and [type identifiers](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/Common/Game/IW3/IW3.h).
Fourteen container/zone hashes are recorded in [source evidence](fastfile-format-sources.json)
and verified locally/in CI. This is fresh Rust with caller-owned transactional
storage; no upstream runtime or proprietary content is included.
