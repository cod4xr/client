# PC IWI6 wavelet pixels

`Iwi6WaveletImage` reconstructs the five PC IWI6 wavelet encodings into explicit
dense bitmap channels. RGBA becomes BGRA8; RGB becomes BGRA8 with opaque alpha;
luminance/alpha, luminance and alpha retain their dense channel order. This is a
pure codec, without allocation, filesystem access, GPU work or color conversion.

`requirements` validates header/shape/budgets and returns exact decoded/work byte
capacities. `decode` reconstructs every mip and face privately, publishing pixels
only after the complete entropy stream required for those levels succeeds. Any
failure preserves the entire output buffer, including its unused tail. Scratch
contents are unspecified on failure. Returned mip views borrow only published
output; encoded input, scratch and the image plan may be dropped independently.

Supported geometry is power-of-two 2D/cube images. One-dimensional raw levels are
supported; larger two-dimensional images require their smaller source mips.
Wavelet volumes and non-power-of-two dimensions fail explicitly. These are codec
profile boundaries, not engine-wide asset limits. Geometry/encoded-byte budgets,
decoded-byte budgets, work-byte budgets and actual caller capacity are independent.
The decoder uses fixed metadata for at most sixteen levels because this legacy
file profile stores dimensions in sixteen bits; image counts have no global arena.

The LSB-first entropy reader distinguishes raw prefix bytes from coded bits.
Zero-filled lookahead cannot authorize missing consumed bits. Three immutable
numerical codebooks cover every twelve-bit prefix; escape symbols decode their
explicit nine/ten-bit values. Reconstruction preserves native arithmetic shifts,
parity and modulo-256 output. Color uses blue-relative green/red coefficients;
source mip deltas operate on a private copy, preserving the published smaller mip.
Cube faces and reverse-resolution mip order share one continuous entropy cursor.

The reviewed decoder ignores trailing encoded data and picmip size-table fields
for wavelet bodies. This implementation exposes encoded payload length and exact
consumed whole bytes/bits without claiming complete input exhaustion or validating
those size fields. ZIP CRC/length verification remains a separate preceding gate.
Renderer format support, cube orientation, gamma and sampling policy stay outside
the codec.

`--inspect-images` CRC-checks selected ZIP entries, validates direct layouts and
reconstructs wavelet pixels. The host owns reusable work/output buffers. JSON
reports direct layouts, wavelet headers, successfully decoded wavelet images,
decoded bytes, consumption and an optional aggregate SHA-256 byte array. The hash
covers wavelet images in selected central-record order, then smallest mip to
largest with faces contiguous. It is diagnostic evidence, not asset authorization,
an archive checksum or pure-server validation. Operational logs contain counts
and fixed failures, without names, paths or pixels. A failed image emits no report.

Fifteen tests cover every codeword suffix and escape, all formats, exact hand
vectors, signed/wrapped reconstruction, correlated color, private deltas, multiple
levels, rectangular raw prefixes, cube faces, output lifetimes, truncation,
capacities, budgets, unsupported geometry and mutations. The inspector also rejects
CRC-valid headers lacking pixels and checks a synthetic independent SHA-256 vector.

The installed corpus contains 25 wavelet images across 21 main IWDs. Debug/release decoding
produced 9,349,793 pixel bytes; the independent Python string-bit/matrix decoder
matched every per-archive pixel hash and consumed extent. No installed pixel data
is stored in Git. [Evidence](installed-wavelet-evidence.json) contains only aggregate
counts and digests. Repeat the independent check against an operator-owned install:

```powershell
python tools/check_installed_wavelet_pixels.py 'D:\Game\main' docs/installed-wavelet-evidence.json
```

Primary algorithm/format research uses the immutable OpenAssetTools
[decoder](https://github.com/Laupetin/OpenAssetTools/blob/63e6d57b9c1866ba0ac8b3e7b134d0652c2be44c/src/ObjImage/Image/IwiWaveletDecoder.cpp).
Seven source hashes are recorded in [source evidence](iwi-format-sources.json).
`tools/import_iwi_wavelet_tables.py` reproduces only 368 numerical codewords,
rejecting gaps/overlaps/range failures before generation. The generated catalogs
retain GPL-3.0-only with [license text](../LICENSES/GPL-3.0-only.txt) and
[attribution](../NOTICE); the surrounding Rust runtime is independently written.
CI verifies the immutable hashes and exact generated files. No upstream C++ runtime
or proprietary asset is included.
