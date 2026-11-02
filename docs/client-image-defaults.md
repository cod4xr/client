# Installed client defaults

`cod4xr-content` imports the native default configstrings needed for compatible
gamestate application. The caller supplies the installed image bytes, work budgets,
packed output and opaque metadata slots. The library owns no filesystem, executable
loader, process, engine state or logging subscriber. Its returned plan borrows only
cooked storage, allowing the source image to be released.

The first reviewed profile identifies one locally audited `iw3mp.exe` by complete
SHA-256 and exact byte length. This is a compatibility/layout identity, not publisher
authentication or a claim about every retail/localized image. Unknown images reject
before table interpretation. RustCrypto `sha2` 0.11.1 is pinned, with its transitive
versions/checksums recorded in Cargo.lock; hashing uses its
[standard digest API](https://docs.rs/sha2/0.11.1/sha2/).

The data layout follows the already locked public client: `src/cl_main.c` locates
`constantConfigStrings` at preferred address `0x007209b0`, and `src/qcommon.h`
declares 16-byte rows containing index, string address and two reserved fields.
The reviewed image has base `0x00400000`, 832 defaults, 18,813 value bytes, a longest
value of 1506 bytes and last index 2273. Only this metadata/hash is recorded in
[local evidence](client-image-defaults-evidence.json); no image or default values
are embedded in the repository.

`Pe32Image` provides checked file-backed preferred-address views according to
[Microsoft's PE format](https://learn.microsoft.com/en-us/windows/win32/debug/pe-format).
It validates DOS/PE/i386/PE32 headers, optional-directory extents, header/section
budgets, ordered nonoverlapping virtual sections, raw extents and 32-bit image
geometry. Raw alignment padding can be read; virtual-only zero fill cannot become
file data. No process addresses, relocations or imports are applied. Image size is
metadata and never allocates a corresponding virtual image.

The importer binds the hash before interpreting the table. A no-output pass checks
every row, reserved field, identifier/order, file-backed string address, terminator,
host budget and complete reviewed table totals/sentinel. Only then does the same
immutable view copy raw values into final caller storage. All errors preserve final
text/metadata; unused tails stay unchanged. Debug output contains counts and lengths.
Native defaults preserve percent and non-UTF-8 bytes: they are copied data, not wire
strings, and inherit no wire-string or native global character-arena limit.

The runnable host-side inspector reads an explicitly selected regular file with an
8 MiB byte quota, including a bounded read if it grows after metadata inspection:

```powershell
cargo run -p cod4xr-client --bin cod4xr-client-inspect -- PATH_TO_IW3MP_EXE
cargo run -p cod4xr-client --bin cod4xr-client-inspect -- PATH_TO_IW3MP_EXE --log-format json
```

Native OS paths are preserved. JSON stdout reports image identity and counts;
shared human development/JSON production logs use stderr. Paths and game values
stay out of logs/reports. The primary client remains the package's default binary.
The local process-stop helper checks exact development paths for both binaries.

Tests use synthetic images/data only. They cover hostile PE geometry/address work,
every required prefix, large virtual extents without allocation, image identity,
table/terminator/order checks, every short budget/capacity, dirty-output rollback,
source-independent results, raw values beyond legacy wire lengths, native arguments,
bounded regular-file reads and metadata-only JSON/human process failures. A separate
local inspection successfully imported the reviewed installed image's 832 defaults.
Merging defaults with validated wire blocks, session/engine application, map/model
loading and playable ordinary-server sessions remain open.
