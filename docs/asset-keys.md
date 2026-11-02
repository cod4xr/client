# Canonical virtual file keys

`asset_key::AssetKey` validates strict relative file names into caller-owned cooked
bytes for future virtual asset lookup. ASCII case normalizes to lowercase and
backslashes normalize to forward slashes. Percent signs and high bytes remain
verbatim; no UTF-8 decoding or Unicode case folding changes legacy names.

The caller supplies byte/component budgets and final storage. Empty names, roots,
directory markers, repeated separators, dot components, native-forbidden double-dot
substrings, colons and ASCII controls reject before any output writes. Success
exposes only complete initialized bytes; original input may be released. Unused
output tails remain unchanged. Equality, ordering and hashing use canonical values;
Debug/errors expose only lengths/counts and fixed failure categories.

No native fixed path array limits this representation. A test admits a 65,536-byte
key with explicit host funding. Keys retain no allocator, source frame, OS handle,
filesystem resolver or logger.

This is a strict virtual file namespace, not a host-path sanitizer. It does not
implement Windows device-name/alias policy, grant extraction authority or resolve
loose physical files. Directory catalog records are not file keys. Native request
sanitation is also a separate adapter: the reviewed native function strips leading
and repeated separators and removes certain dots. This API rejects ambiguous forms
instead of treating them as aliases. The native comparator's inconsistent colon
handling is deliberately excluded. Normal relative names preserve native ASCII
case and slash equivalence; existing network/inspector behavior is unchanged.

Five tests cover canonical equivalence/hash contracts, raw/Unicode bytes, private
Debug output, every forbidden control byte, malformed paths, every short output,
exact work/component budgets, source-independent ownership and 2,816 deterministic
single-byte mutations. Mount order, duplicate precedence, archive ownership,
physical filesystem policy and actual asset resolution remain separate gates.

A separate Python metadata audit found 24,094 file records and 149 directory markers
in the installed main IWDs. All file names fit this strict form, with at most five
components. Ten exact-name duplicates occur within archives, each with different
payload metadata. This is preliminary namespace evidence, not a Rust mount test.
An upcoming index must preserve native last-entry precedence and explicit archive
order instead of rejecting these ordinary duplicate records. The index should copy
keys and entry metadata so archive byte buffers need not remain resident for lookup.

Source evidence is the hash-locked read-only client reference's
[`FS_SanitizeFilename`](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/files.c#L3003-L3063)
and [`FS_FilenameCompare`](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/files.c#L3159-L3189).
