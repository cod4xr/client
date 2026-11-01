# Auxiliary reliable send window

`cod4xr_reliable::send` owns an outgoing queue over exact caller-provided byte
and metadata buffers. It allocates nothing. Construction checks all geometry and
the explicit in-flight span before clearing metadata; dirty backing bytes remain
private. Queue capacity is independent of that span and can exceed the native
32-slot initial buffer. The native default span is four; larger explicit spans
must fit the signed sender-window hint and be supported by the selected peer.
Incoming hints never change either resource decision or activate an extension.

`enqueue` copies an entire input into consecutive fragments of at most 1200 bytes.
Capacity and sequence checks finish before any byte or counter changes. Empty
input is a no-op. Exact multiples do not add a zero-length terminator: this is a
byte stream, distinct from ordinary-channel fragmentation. Binary bytes retain
their exact original values, without command-string normalization. The half-open
enqueue interval and retained byte/fragment counts contain metadata only.

`prepare_transmission` exclusively borrows one retained active fragment. Its
body view uses the original sequence and bytes plus explicitly supplied receive
ACKs/flags and the host-selected sender-window hint. The existing shared outer
marker/qport codec remains separate. Dropping preparation does not mark a send.
Only after complete successful OS datagram I/O may the host call `record_sent`.
Retransmissions retain the same original sequence and bytes and report that the
send is no longer the first. Timers, pacing and retry budgets remain the host's job.

`prepare_acknowledgement` takes a completely parsed packet and preflights its
whole cumulative/selective ACK update. The cumulative base cannot regress,
exceed queued data or skip beyond the current in-flight span. Every claimed
fragment must have been transmitted; queued data and unsent holes are insufficient.
Selective ranges must fit both the relative span and actual retained data. A
late invalid range cannot release an earlier valid prefix or mark an earlier
valid range. Unsorted/overlapping valid ranges are admitted and newly marked
fragments are counted once. No incoming flag or hint is applied during this check.

Preparation changes nothing and can be cancelled. Its commit is infallible under
the exclusive storage borrow. Selective marks suppress retransmission but retain
the original bytes and queue capacity until a validated cumulative ACK consumes
them. Omitting a previous selective range does not erase its mark. Sequence-tagged
slot reuse prevents old IDs from exposing new data. Queuing stops before a data
sequence would need an unrepresentable consumed endpoint; reconnecting replaces
wrapping, with a shared signed-ACK boundary constant for send and receive state.

The host must also preflight the incoming receive update before committing either
side. Separate exclusive borrows permit both preparations to coexist. Failure on
either side drops the other preparation; committing both then cannot fail midway.
Neither side alone grants permission to execute payloads or send a force-ACK reply.

Tests cover exact binary fragment boundaries, cancellation, dirty storage, capacity,
the distinct queue/flight spans, unsent claims, regressions, whole-update atomicity,
overlapping ranges, retained selective bytes, sequence exhaustion and maximum signed
metadata geometry. An independent 50,000-byte model introduces first-attempt loss,
reorder and duplicates while repeatedly reusing seven outgoing slots; the reconstructed
byte stream must match exactly. This is deterministic model evidence. Real outgoing
auxiliary UDP I/O, pacing/retransmission orchestration and ordinary-peer traces are
still required before claiming a complete transport session.
