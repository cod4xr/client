# Copied virtual mount index

`mount_index::MountIndex` builds canonical virtual file lookup from an immutable
slice of `MountCandidate` values. Each candidate pairs a validated `AssetKey`, a
caller-chosen `ArchiveId` and copied catalog metadata. The last candidate for each
canonical name wins. Callers supply explicit archive priority and original central
directory order within each archive; archive ID magnitude never determines priority.

This preserves the reviewed native loader's within-archive behavior: each central
entry is inserted at the head of its filename hash chain, so later duplicates take
precedence. Ordinary installed IWDs contain duplicate names with differing payload
metadata. Rejecting all duplicates would break their normal lookup behavior.

The builder inspects input candidate/key-byte budgets and writes only source
positions into caller-owned private scratch. It sorts those positions by canonical
bytes and original position, selects each group's last candidate, then preflights
all retained counts, key bytes and final capacities. Only after complete success
does it copy keys and numeric routing/declarations into final storage. All failures
preserve final bytes and descriptors; private scratch is unspecified. Sorting and
publication use immutable slices, without allocation or replayed external callbacks.

The result borrows only final key bytes and descriptors. Original ZIP buffers,
catalogs, candidate arrays, source keys and scratch may be released. Binary lookup
accepts a validated key and returns copied `MountedAsset` routing/metadata that may
also outlive the index. Record iteration is in canonical byte order. Unused caller
tails do not participate, and sparse archive IDs retain all `u64` bits without
allocating an ID-sized array. Debug/errors exclude names, payloads and checksums.

An `ArchiveId` is a host routing identifier, not an owned file handle. This layer
opens no files, binds no epochs, applies no pure policy and loads no engine assets.
The [archive host](archive-host.md) now retains original handles and issues bounded reads; the payload
decoder then checks source length, complete stream, decoded size and CRC. Native
request sanitation, actual archive search-order discovery and physical filesystem
resolution remain separate work. Candidate ordering is explicit, not inferred
from the directory enumerator.

Seven tests cover source-independent lookup, native duplicate precedence across
canonical case/slash variants, sparse IDs, missing binary-search keys, record/privacy
contracts, copied results after index release, empty input, dirty workspace reuse,
every short output/descriptor capacity and exact inspection/retention budgets.
An independent ordered-map model checks 256 override traces. A complete integration
catalogs valid duplicate ZIP entries, canonicalizes names, builds the copied index,
releases all original ZIP/key/candidate buffers, and decodes/checks the selected
payload from a separately retained bounded byte source.

Native source evidence:
[`FS_LoadZipFile` hash-chain insertion](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/files.c#L2957-L2964)
in the immutable, hash-locked read-only client reference. This index implements
data lookup rather than claiming an installed engine mount or playable session.
