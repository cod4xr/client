# Unsigned PC version-5 fastfile containers

`fastfile5::Fastfile5Decoder` checks the PC `IWffu100` signature, little-endian
version five, one complete zlib stream and its decoded size declaration. It never
loads executable code, follows serialized pointers, allocates declared blocks,
registers assets or grants renderer authority. Complete container bytes remain
untyped serialized data for the next content layer.

The twelve-byte outer header precedes zlib data. The decoded section starts with
eleven little-endian 32-bit words: serialized body size, external size and nine
memory-block declarations. The body size excludes this forty-four-byte size header.
Block identifiers are temporary, runtime, large runtime, physical runtime, virtual,
large, physical, vertex and index. They describe the legacy loader profile; they
do not impose engine allocation limits or authorize memory access. Block totals
use wide arithmetic and are independent of serialized byte counts. External size
is inert metadata, never an inferred file extent or download request.

`inspect` inflates only the bounded size-header prefix and returns copied metadata,
exact decoded-byte requirements and work requirements. Its success does not prove
that the remaining compressed body exists or passes integrity checks. Decoded-byte,
encoded-byte, total-block and external-size budgets are independent host inputs.
There is no inherited native one-gigabyte memory cap or automatic arena allocation.
Caller capacity is checked separately from those policy budgets.

`decode` resets its inline inflater and processes the complete stream into private
caller-owned scratch with an extra overflow guard byte. Zlib header/Deflate/Adler-32
validation, exact declared length and complete encoded-input exhaustion must all
succeed before any final bytes are copied. Concatenated streams, preset dictionaries
and trailing bytes fail. Every failure preserves the entire output, including its
unused tail; scratch and codec state are unspecified. The next operation resets
the inflater independently, so a failed file cannot seed another file's history.
No decoded or scratch storage is allocated inside the pure crate.

`CheckedFastfile5` borrows only final output. Its metadata is copied, and `contents`
views only the serialized body after the checked size header. Encoded input, work
and the decoder may be dropped while those views remain usable. Debug and errors
contain fixed categories and counts, never serialized content, pointer values or
file/asset names. Signed/console/other-version formats fail explicitly; this file
profile choice is independent of server connection authentication.

The inspector adds:

```powershell
cargo run --bin cod4xr-client-inspect -- --inspect-fastfile 'D:\Game\zone\english\common_mp.ff'
```

The host owns file I/O, fallible buffers and SHA-256 calculation. Its explicit
inspection budgets are command policy rather than engine size limits. JSON stdout
reports byte counts, declared sizes and the whole decoded section's SHA-256 as a
byte array. Container integrity is reported independently from unverified asset
interpretation and gameplay. Human/JSON logs retain fixed events and bounded counts.
Any container failure produces no partial report. Nothing is extracted to disk.

Twelve library tests cover independently encoded stored/dynamic-Huffman vectors,
all nine block identifiers, zero-body containers, decoded-length/Adler mismatch,
concatenation/tails/dictionaries, every fixture truncation and short capacity,
independent exact budgets, wide declarations, explicit profiles/endianness,
mutations, source-independent lifetimes and state reuse after failure. Two process
tests cover private human/JSON reporting, malformed files and report atomicity.

`tools/check_installed_fastfiles.py` compares the built inspector with independent
Python zlib decoding, exact size checks and every decoded SHA-256, storing aggregate
metadata only when explicitly requested. It reads an operator-owned install and
never writes content or records names. Debug and release checks of all 76 installed
files matched 2,642,982,057 encoded bytes / 4,764,264,952 decoded bytes and every
decoded SHA-256, with no compressed tails. The largest decoded file was 181,060,522
bytes. [Aggregate evidence](installed-fastfile5-evidence.json) records counts and a
SHA-256 over the ordered per-file decoded hashes. These checks establish container
validation, without asset interpretation or playable map loading.

Primary format research is pinned to OpenAssetTools revision
`63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c`, with nine hashes in
[source evidence](fastfile-format-sources.json). The
[factory](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Game/IW3/ZoneLoaderFactoryIW3.cpp),
[profile constants](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneCommon/Game/IW3/ZoneConstantsIW3.h),
[block definitions](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/Common/Game/IW3/IW3_Assets.h),
[size reader](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Loading/Steps/StepLoadZoneSizes.cpp)
and [zlib processor](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ZoneLoading/Loading/Processor/ProcessorInflate.cpp)
establish the outer container boundaries. This is independently written Rust using
the already pinned miniz codec; no upstream C++ runtime or game data is included.
Local/CI hash checks share a bounded immutable-source verifier.
