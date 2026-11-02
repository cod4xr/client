# Legacy entity delta records

`EntityDeltaDecoder::with_legacy_map_center` selects protocol-6 individual updates
against an explicit prior state and checked `LegacyMapCenter`. The original
`new(Legacy6)` rejects missing context; `new(Cod4x21)` keeps its raw-coordinate rules.
Neither discovery nor peer text changes the selected profile. `profile()` reports
that context, and legacy center replacement preserves index ordering. `reset`
restarts ordering while retaining the explicitly selected current center.

All field tables, type selection, changed-prefix rules, removals and prepared
reader/index ownership are shared with [modern delta records](entity-deltas.md).
The same private coordinate reader now accepts both zero and nonzero reference
words, so initial baselines do not have a separate legacy origin implementation.
Normalized semantic words remain independent of a native memory layout or a fixed
engine entity array.

A short origin form adds its seven-bit signed delta to the prior binary32 value,
retaining a prior fractional component. A full origin form truncates the prior
value to a signed integer, applies the sixteen-bit XOR using all prior map-biased
bits, restores the bias through explicit signed wrapping and converts the result
to binary32. A fractional prior can therefore behave differently in short/full
forms, as in the reviewed native reader. Final full-form rounding also matters
near signed integer and binary32 precision boundaries. Other field kinds retain
their own native rules; a normal float in a target layout does not become a legacy
origin merely because its semantic word stores a position.

Changed legacy origins reject nonfinite priors. Full forms additionally reject a
truncated prior outside signed-32 conversion, avoiding native conversion overflow.
Finite short forms perform float addition without that integer-conversion bound.
Unchanged fields, no-delta records and removals do not perform coordinate math;
unchanged raw words remain inert until later engine validation. These strict invalid
reference checks do not establish geometry validity or remove the legacy wire's
coordinate limitations.

`prepare` retains exclusive reader/index owners while the host selects a matching
prior entity. `decode` uses that owner's exact coordinate context. A late invalid
coordinate, truncation, wrong number or dropped preparation leaves both originals
unchanged, including cached bits; the borrowed prior state is never modified.
Successful results own their semantic words independently of source buffers, prior
state and decoder lifetime. Map-center reconfiguration cannot occur during the
existing exclusive preparation lease. No I/O, callback, host snapshot mutation,
message acknowledgement or engine effect runs inside these operations.

Eleven integration tests cover all 128 short values against nonzero state, fractional
short/full differences, every target layout and unknown fallback, unconditional
forms, message-relative time, signed-wrap/float-rounding boundaries, explicit
profiles and center changes, context-preserving reset, no-delta/removal semantics,
source/prior lifetimes, all truncated record prefixes, cancellation, late invalid
reference rollback and full-form conversion bounds. Shared baseline fixtures moved
into one test-support module; existing baseline vectors remain covered.

The primary rules remain the pinned public
[coordinate reader](https://github.com/callofduty4x/CoD4x_Client_pub/blob/17182c7bf04b859dea5c0db4acdb0b83c28d6dd2/src/msg.c#L885)
and the exact reviewed native routines recorded in
[audit metadata](legacy-coordinate-reader-evidence.json). The implementation uses
their reader semantics, including prior truncation, rather than inferring nonzero
behavior from the zero-reference case. No binary or disassembly is included in Git.
Complete legacy gamestates, per-block effective `CS12` integration, full snapshot
framing/terminators, player/client records, engine application and native peer
traces remain separate work. These are record codecs, not a playable client.
