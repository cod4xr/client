# Owned host archive I/O

`cod4xr-client-content` owns retained regular-file handles, process-local handle
epochs and bounded reads. It depends only on deterministic `cod4xr-content`, which
continues to own format, key/index and payload validation. The client now has six
real crates; filesystem operations remain outside deterministic content/protocol code.

`ArchiveRegistry::open_local_file` accepts an explicitly host-approved native path,
checks open-handle/file-size budgets, reserves handle storage fallibly, and retains
the original file. No path or OS error string is stored in registry diagnostics.
IDs come from a process-wide monotonic atomic counter, are never reused after
closure, and reject exhaustion rather than wrapping. They are runtime epochs,
not persistent cache identities or publisher authentication.

Whole-image reads fill caller private input for a deterministic parser. Catalogs
can publish copied declarations/index data, then release the source image. Later
`read_entry` or `read_asset` operations route to the exact original handle, preflight
budgets/capacities and source extents, seek/read an exact compressed span, and invoke
the existing complete stream/length/CRC decoder. Successful payloads borrow only
final caller storage and remain valid after the archive closes. All failures leave
final decoded bytes unchanged; private image/compressed/scratch buffers are unspecified.

The registry checks observable source length/modification stamps before and after
reads. These checks detect ordinary in-place changes; they do not claim a
cryptographically immutable filesystem snapshot. CRC checks still validate admitted
payload consistency. Replacing a pathname does not silently redirect an already
open handle. Reopen creates a new epoch; stale or foreign-registry IDs reject.
Handles are private, reads require an exclusive registry borrow and no cloned raw
file handle is exposed. Dropping the registry closes remaining files through RAII.

Resource policy is explicit rather than a native 1024-archive global. Handle lookup
uses retained records, without allocating by sparse ID magnitude. Usage distinguishes
attempts, completed reads and admitted source/decoded bytes. Failed or partial reads
never count as admitted bytes. Safe I/O errors retain operation, `ErrorKind` and an
optional numeric OS code, without OS-supplied text or path information.

The inspector is the first real consumer. Archive modes open one original handle,
read/catalog its image, copy only declarations and summary counts, release the whole
ZIP buffer, then verify entries through that same retained handle. Three private
payload buffers are reused. Reports remain metadata-only and gain an admitted-source
byte count; success still requires every selected entry's exact length and CRC.
The primary client CLI remains a discovery/challenge probe, without playable joins.

Eleven tests cover epoch exhaustion, resource budgets, safe missing/nonregular I/O,
global uniqueness, close/reopen and foreign IDs, every short image/payload storage,
work limits, out-of-file spans, source changes, original handles after rename/path
replacement, CRC rollback and exact admitted counters. A complete integration builds
a copied virtual index, releases all original image/key/index storage, reads the
selected real temporary file through its retained handle and retains the checked
payload after closing the registry. Existing inspector process tests exercise the
new consumer without exposing private fixture paths or values.

Platform contracts are provided by Rust's standard library:
[`OpenOptionsExt::share_mode`](https://doc.rust-lang.org/std/os/windows/fs/trait.OpenOptionsExt.html#tymethod.share_mode)
and [`fs::rename`](https://doc.rust-lang.org/std/fs/fn.rename.html). The rename/handle
behavior is tested directly on the pinned Windows host. Hosted cross-platform
validation will follow this checkpoint's commit.

The release inspector verified all 21 installed main IWDs through this owner:
24,243 entries and 3,999,040,612 decoded bytes passed exact length/CRC checks.
Admitted source reads totaled 6,058,270,418 bytes, exactly the combined catalog
images plus compressed payload spans. [Aggregate evidence](owned-iwd-payload-evidence.json)
contains no file names, paths, extracted payloads or game binaries. The debug owner
also verified the smallest archive's seven entries / 844,144 decoded bytes.
