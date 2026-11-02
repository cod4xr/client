# Checked archive payloads

`zip_payload::ZipPayloadDecoder` accepts an unforgeable catalog entry and explicit
compressed/output budgets. It verifies a complete stored or raw-Deflate payload's
exact declared length and ZIP CRC32 before copying any bytes into final caller
storage. A failure preserves final output; private scratch/codec state may change.
Success returns only the initialized final prefix, which can outlive the archive,
catalog, metadata, decoder and scratch.

[Copied entry metadata](archive-entry-metadata.md) also supports separately supplied
payload bytes through `decode_bytes`, with an exact source-length gate and the same
stream/length/CRC checks. Hosts can release original ZIP images before later bounded
reads, while retaining responsibility for archive-handle/epoch binding.

The content crate delegates Deflate and IEEE CRC32 to exact pinned `miniz_oxide`
0.9.1 and `crc32fast` 1.5.4 dependencies. Default features are disabled; the Deflate
allocator/vector helpers are excluded. The only new transitive package is `adler2`
2.0.1. No custom compression or checksum implementation was introduced.

The caller owns reusable inline decoder state and output/scratch buffers. Each
Deflate operation resets codec state, uses a non-wrapping caller buffer, and
requires a complete stream with every compressed byte consumed. Scratch holds the
declared output plus one guard byte, allowing excess output to reject without
accepting an attacker-controlled size declaration. Stored entries need no scratch.
Both input and declared output budgets validate before decoding/checksum work.
There is no allocation proportional to a declaration inside the content library.

The checked bytes remain data. CRC consistency does not establish publisher
identity, select archive precedence, admit a virtual path, authorize execution or
implement native pure-checksum policy. ZIP catalog declarations remain explicitly
unverified until this separate payload check succeeds.

The existing inspector adds explicit verification:

```powershell
cargo run --bin cod4xr-client-inspect -- --verify-archive 'D:\Game\main\iw_00.iwd'
```

The command checks all entries in memory and writes no extracted files. Its three
payload buffers are reused and grow only to the largest admitted entry. It uses
the [owned archive host](archive-host.md), frees the complete ZIP image after
cataloging, then reads exact compressed spans from the original retained handle. Command
policy permits up to 128 MiB compressed and 64 MiB decoded per entry; these are
host inspection budgets, not engine limits. File/descriptor/payload reservations
are fallible. Reports contain aggregate counts and `crc32_verified` only after
every entry succeeds. Failures emit fixed events without names or partial reports.
Catalog mode (`--archive`) still reports `unverified` payload integrity.

Eleven library tests cover stored/fixed/dynamic Deflate forms, empty streams,
independent Python/zlib vectors beyond the 32 KiB history window, every short
Deflate prefix, all appended byte values, false lengths/CRCs, byte mutations,
exact work/storage bounds, dirty codec/scratch reuse and source-independent
metadata-only output. Two process tests cover human/JSON verification success,
consistent-but-false CRC metadata, oversized declarations before payload allocation,
private values and absence of partial reports. Existing tests also cover native
non-Unicode paths and invalid verification arguments.

The debug verifier checked all seven entries / 844,144 decoded bytes in the
smallest installed main IWD. The release verifier checked all 21 installed main
IWDs: 24,243 entries and 3,999,040,612 decoded bytes passed exact-length/CRC checks.
[Aggregate evidence](installed-iwd-payload-evidence.json) records these distinct
local scopes without file names, payloads or binaries. This proves payload
consistency for those local archives; virtual mounting and gameplay remain open.

Primary codec contracts: [`miniz_oxide::inflate::core::decompress`](https://docs.rs/miniz_oxide/0.9.1/miniz_oxide/inflate/core/fn.decompress.html),
[`miniz_oxide` feature flags](https://docs.rs/crate/miniz_oxide/0.9.1/features)
and [`crc32fast`](https://docs.rs/crc32fast/1.5.4/crc32fast/).
