# Effective sparse configstrings

`config_state::ConfigSnapshot` merges caller-selected default values and validated
block-scoped patches into a complete sorted, unique, caller-owned snapshot. It
applies no engine/session state and gives values no command or path authority.
The protocol core depends on no content importer or filesystem implementation.

The native rule matters: every configstring block reapplies all known defaults.
A default ID uses its explicit value only if the last block contains that ID;
otherwise its default is restored. IDs without a default keep their last explicit
value from any block. With no native blocks, no implicit constants are applied and
the effective snapshot is empty. Empty explicit values remain valid overrides.

Sources are immutable slices of `ConfigValue`/`ConfigPatch`, so validation and
publication never replay arbitrary external iterators or callbacks. Callers can
adapt the content importer's default views and the gamestate plan's patch views
into prefunded descriptor storage. View byte references retain their source-data
lifetime independently of temporary copied view objects.

The caller supplies separate input-record/input-byte and effective-record/text
budgets, private `ConfigMergeSlot` scratch, final text and final opaque metadata.
The builder checks source counts, byte work, default order, native block starts,
within-block order and embedded NULs. Used scratch descriptors contain only IDs,
source indices and block numbers. They are sorted in place by ID without allocation;
unused scratch tails never participate. No array is sized by ID magnitude.

Each group selects its effective source using original block scope. All selected
counts, text extents and final capacities validate before any final write. Only
then are immutable source bytes copied into packed output and sorted unique slots.
Every failure preserves final output; scratch is unspecified. The result borrows
only final cooked storage, permitting sources, parent plans and descriptors to be
released or reused. `value(id)` uses binary search; unused output is never exposed.

IDs retain all `u32` bits, including sparse `u32::MAX`. Counts/text remain host
budgets rather than baseline namespaces, native character arenas or per-field wire
limits. Raw percent/non-UTF-8 bytes are preserved without a second normalization.
Debug output exposes bounded work/resource counts, never values. The library owns
no allocator, logger, source-frame release or session/engine commit.

Eight tests cover restoration and nondefault persistence, latest empty overrides,
no-block behavior, source-independent results, raw values above legacy field sizes,
sparse wide IDs, every short budget/scratch/output capacity, dirty scratch reuse,
invalid order/markers/NULs and private debug output. An independent native-style
map model checks 256 traces. A wire-decoder integration test builds effective state
from validated gamestate views and releases all source/frame/descriptor storage.
Content importer view tests also retain byte references from temporary view copies.

This completes the data merge, not a playable engine transition. Session epochs,
server-info/resource interpretation, asset loading, legacy coordinates, snapshots,
rendering and real ordinary-server gameplay remain open.
