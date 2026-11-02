# Classic IWD archive catalogs

`cod4xr_content::zip32::Zip32Directory` provides deterministic read-only classic
ZIP metadata for IWD content. The host supplies immutable archive bytes, explicit
budgets, private descriptor scratch and separate final descriptors. No filesystem,
allocator, logger, archive mount, decompressor or engine state belongs to this layer.

The parser checks the terminal end record, central-directory extent/count, local
headers, matching names/methods/flags/sizes and compressed spans. It supports stored
and Deflate entries, both classic streaming-descriptor forms, explicit UTF-8 names,
raw legacy names, comments and framed inert unknown extra fields. Duplicate names
remain separate records in original central order; physical file order may differ.
Sorting private descriptors detects overlapping or aliased local ranges, then
restores central order before copying to final output. Every error preserves final
descriptors. Unused caller storage never participates in the result.

Budgets bound archive/directory bytes, entries, individual/combined name bytes and
individual/combined declared output. Name budgets apply before name scanning.
Counts use host widths and combined declared sizes use `u64`; a synthetic test
catalogs six billion declared output bytes without allocating output payloads.
The classic format itself still has 16/32-bit fields. ZIP64, multi-disk, encryption,
other compression methods, ambiguous terminal records and central signatures are
explicitly outside this reviewed profile. A future ZIP64 implementation must retain
wide engine sizes rather than silently reinterpret classic sentinels.

Copied entry views borrow only original archive bytes and carry private copied
metadata. They may outlive the catalog, scratch and descriptors. Their Debug output
contains counts and compression type, never names, comments, payloads or checksums.
Catalog names remain inert, including names that would be unsafe OS paths. This
layer neither normalizes them into virtual keys nor authorizes extraction. Archive
attributes and alternate Unicode extra fields acquire no mount or symlink authority.

Declared sizes/CRCs and a checked compressed span do not establish integrity.
[Checked payload decoding](zip-payloads.md) now supplies exact output length/CRC
validation in a separate layer. Virtual-path policy, duplicate precedence, mount
order, pure checksums and engine asset loading remain separate gates. The independent Deflate fixture was emitted by Python's standard
ZIP writer; deliberately invalid compressed fixtures verify that cataloging does
not accidentally claim decompression success.

The existing inspector now accepts an explicit archive mode:

```powershell
cargo run --bin cod4xr-client-inspect -- --archive 'D:\Game\main\iw_00.iwd'
```

JSON stdout reports counts only. Development logs remain human-readable by default;
`--log-format json` selects structured stderr. Inspection uses a 512 MiB per-file
host budget and bounded descriptor storage. This command policy does not limit
future engine content capacity. The existing image-inspection invocation still
works, and the workspace's default runnable binary remains the main client.

Seventeen catalog tests cover physical/directory order, duplicate names, empty
archives, raw/UTF-8 names, unsigned/signed descriptors and CRC/signature collisions,
malformed features/extras/headers, overlap, every fixture truncation, appended bytes,
exact budgets/storage, metadata privacy and deterministic record mutations. One
argument test and three process tests cover explicit archive selection, metadata
reports, human/JSON logs, malformed input and rejection of oversized files before
payload reading. Existing native-path argument tests also cover archive mode.

Debug and release inspectors successfully cataloged all 21 installed main IWDs: 24,243
entries, including 150 stored and 24,093 Deflate entries. Aggregate counts and sizes
matched an independent Python ZIP reader. This was metadata-only validation, not
payload integrity or gameplay evidence. The committed evidence contains aggregate
metadata only; no names, game payloads or binaries are included.

Format evidence: [PKWARE APPNOTE 6.3.10](https://pkware.cachefly.net/webdocs/casestudies/APPNOTE.TXT),
sections 4.3.7, 4.3.9, 4.3.12 and 4.3.16. The locked client reference's
[`FS_LoadZipFile`](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/files.c#L2877-L2980)
establishes native IWD catalog order and CRC-based pure-checksum inputs. Its hash is
recorded in `upstream.lock.json`; only the read-only client reference was inspected.
