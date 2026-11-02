# Content validation

These records describe historical checkpoints. Remaining-gate statements apply to
that checkpoint; see [current validation](../validation.md) for the latest status.

Seventeen classic ZIP catalog tests add independent directory/local-header and
streaming-descriptor vectors, original/physical order, duplicate names, raw/UTF-8
names, comments, unsupported features/extras, overlap and wide declared totals.
Every fixture prefix, appended byte values, exact host/storage bounds and record
mutations cover final-output rollback and bounded views. One argument and three
process tests cover explicit archive inspection, private metadata-only reports,
human/JSON events and oversized-file rejection before payload reads. Debug/release
inspector cataloged all 21 installed main IWDs / 24,243 entries; counts and size totals
matched Python's ZIP reader. Aggregate [evidence](../installed-iwd-catalog-evidence.json)
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
[Payload evidence](../installed-iwd-payload-evidence.json) records only aggregate
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
match the catalog images plus compressed spans exactly; [owned-I/O evidence](../owned-iwd-payload-evidence.json)
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
21 installed main IWDs. [Image evidence](../installed-iwi6-evidence.json) contains only
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
26,876,561 consumed bits and 3,359,580 encoded payload bytes. [Wavelet evidence](../installed-wavelet-evidence.json)
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
tails. [Container evidence](../installed-fastfile5-evidence.json) records aggregate
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
[Directory evidence](../installed-zone5-evidence.json) contains aggregate metadata only. Child
assets, serialized targets, registry resources and playable rendering remain open.
