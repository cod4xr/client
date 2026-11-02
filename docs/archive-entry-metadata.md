# Copied archive-entry metadata

`Zip32Entry::metadata()` returns a compact `Zip32EntryMetadata` containing checked
compressed offset/count, declared output length/CRC and compression method. Its
fields are private; construction requires an entry from a successfully validated
catalog. It has no borrowed names, descriptors or ZIP bytes, and remains usable
after all catalog/input storage is released. Debug excludes names and checksums.

This prepares for a virtual index that copies canonical names and entry metadata
instead of retaining every complete IWD in RAM. Offsets are classic-profile host
spans; declared decoded sizes remain `u64`. A future host must bind this metadata
to the correct original archive handle/epoch and perform bounded file reads.
The metadata itself opens no file and gives no path or integrity authority.

`ZipPayloadDecoder::decode_bytes(metadata, input, budget, scratch, output)` checks
separately supplied compressed bytes. The catalog's expected input count and actual
input both must fit the host budget, then the supplied length must match exactly.
All existing complete-stream, guarded-output, exact-length and CRC rules run before
copying to final output. Every error preserves final bytes. Existing `decode(entry,
...)` delegates to the same implementation, preserving the borrowed-entry API.

Three tests prove correct span routing, copied metadata after original storage
release, metadata-only Debug, all short/extra source lengths, budget precedence and
same-sized corrupted replacement bytes. Existing extraction tests continue to
cover all codecs, stream prefixes/trailers, storage bounds and publication rollback.

This is a data/I/O ownership boundary, not a filesystem implementation. Archive
handle retention, virtual mount/duplicate policy, actual seek/read ownership and
native pure checksums remain separate work. The [copied index](mount-index.md) now
retains native last-entry precedence and caller-specified archive priority without
borrowing original ZIP buffers.
