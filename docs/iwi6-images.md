# PC IWI6 image layouts

`iwi6::Iwi6Header` validates explicit PC version-6 metadata and interpreted geometry.
`Iwi6DirectImage` additionally validates complete dense bitmap/BC payload extents,
active picmip sizes and exact input exhaustion. Neither API allocates pixels, opens
files, performs GPU uploads or selects renderer capabilities. Header success alone
is not proof of complete pixel data.

The format has a 28-byte little-endian header. Bitmap color channels are explicitly
BGR/BGRA, with separate luminance, luminance/alpha and alpha encodings. Direct block
formats are BC1, BC2, BC3 and BC5. The five wavelet encodings are recognized in header
metadata and explicitly reject direct-mip interpretation; wavelet pixel decoding
remains unimplemented.

Effective depth is one for 2D/cube images and the stored depth for volumes. Cube
takes precedence over the volume bit as in the reviewed loader. Mip dimensions
halve with a minimum of one; cube faces are contiguous within each mip. File order
runs from smallest mip to largest, while public views use logical base level zero.
Dense bitmap rows have no extra implicit alignment; block sizes round width/height
up to 4-by-4 tiles. Used cumulative picmip sizes must match exact computed extents;
inactive size-table entries remain inert. The no-picmip flag does not disable mips.
Cube axes, color-space policy, normal-map interpretation and backend constraints
remain renderer responsibilities. Format flags are retained data, not executable policy.

Host byte, mip-level and total logical-texel budgets are independent. Texel counts
include every interpreted level, cube face and volume slice, using checked wide
arithmetic. One profile's mip metadata needs at most sixteen slots because stored
dimensions are `u16`; this is not an engine image-count limit or a global arena.
Inputs remain immutable. Mip views may outlive their parent plan while borrowing
only cooked image storage; owned headers can outlive the entire source buffer.
Debug/errors expose metadata and fixed categories, never pixels or names.

The inspector adds an explicit selected-image mode:

```powershell
cargo run --bin cod4xr-client-inspect -- --inspect-images 'D:\Game\main\iw_00.iwd'
```

It selects `.iwi` names without UTF-8 coercion, checks complete selected ZIP payload
lengths/CRCs through the original retained file handle, then interprets image headers
and direct layouts. The source ZIP image and catalog work arrays are released before
payload reads. JSON stdout separates image headers, direct layouts and wavelet
headers, explicitly marks wavelet decoding unfinished, and leaves gameplay unverified.
Unselected files are not advertised as CRC-checked. Human/JSON stderr logs retain
counts and fixed failures without file names, paths or pixel data.

Thirteen library tests cover all nine direct formats, reverse mip order, NPOT block
rounding, inactive depth, cube faces, volume slices, sixteen levels, wavelet header
recognition, explicit versions, zero dimensions, source lifetimes and metadata privacy.
They also cover every fixture prefix, all appended byte values, active/inactive
picmip entries, exact host budgets and deterministic header mutations. Two process
tests cover selected-image scope, independently computed CRC vectors, direct versus
wavelet reporting, wrong-version failures, native arguments and private output.

Debug and release inspection of all 21 installed main IWDs validated 6,565 IWI6 headers and
6,540 complete direct layouts. The other 25 were explicitly reported as wavelet
headers. The audit inspected 56,168 mip levels / 2,565,716,492 logical texels, with
maximum stored/effective dimension 2048. Selected image payloads totaled
2,018,472,572 CRC-checked decoded bytes. [Aggregate evidence](installed-iwi6-evidence.json)
contains no asset names, pixels or binaries. This is data-format evidence, not
rendering or gameplay evidence.

Format study uses immutable public OpenAssetTools references, with five hashes in
[source evidence](iwi-format-sources.json) and bounded local/CI verification through
`tools/verify_iwi_sources.py`. The Rust reader is a fresh checked implementation;
no upstream C++ runtime or game payload is included. Primary format/geometry sources:
[types](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/IwiTypes.h),
[reader](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/IwiLoader.cpp),
[writer](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/IwiWriter6.cpp),
[texture geometry](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/Texture.cpp)
and [format sizes](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/ImageFormat.cpp).
