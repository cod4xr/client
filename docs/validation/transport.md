# Transport validation

These records describe historical checkpoints. Remaining-gate statements apply to
that checkpoint; see [current validation](../validation.md) for the latest status.

Eight new independent loopback tests exercise the reusable network crate: one
source socket across challenge/connect/retries/handoff, exact cookie/protocol/qport
fields, inert owned refusal data with socket closure, oversized-datagram rejection,
other-peer filtering, last-slot receive quotas, expiration before a delayed `run`,
silent-peer expiry with retained work counters, and pre-I/O budget/endpoint rejection.
No ordinary gameplay server or complete game channel was used for this checkpoint.

Ten subsequent real UDP tests establish ordinary-channel handoff on the original
socket, qport bits/direction, initial sequences, empty messages, paced fragments
and exact-multiple terminators, interleaved receives, cancellation/I/O-failure
reservation, malformed/stale/reordered traffic, reassembly capacity and explicit
discard, auxiliary marker/qport checks, and expired waits without charged work.
One compile-fail documentation test establishes that receive data cannot escape
the scoped callback. Shared transport logic is consumed at the existing immutable
revision, without source changes to the server repository. Two more client research
files were hash-verified locally and in that checkpoint's hosted source job.

Eight command-history tests add native byte-cleaning/ownership/privacy checks,
valid/invalid caller storage, exact command-length and pending-window boundaries,
the full 128-entry baseline, check-before-commit acknowledgement, unsent/future/
regressing numbers, repeated reuse across every 1–128-slot capacity, and exhaustion
at the positive signed sequence bound. Three additional client source files are
hash-verified locally and in that checkpoint's hosted source job. No
server source files or server worktrees were edited for these client checkpoints.

Eight control-frame tests add independent idle/keyed wire vectors for both header
profiles, source ownership, every insufficient output prefix, a complete 128-command
window, retained-key/header boundaries, whole-prefix deferral and compression
expansion without truncation. A further UDP integration test rejects wrong protocol/
cookie contexts before sequence reservation and verifies a fragmented complete
control frame on the original socket. Marking the included command prefix sent is
separate from construction and from validated peer acknowledgement.

Eight payload tests add explicit profile mapping, independent initial/keyed server
vectors, native padding, key selection without commit, decoded-byte opacity,
unsent/future/regressing numbers, every short scratch/output prefix, incomplete/
empty/oversized input, sequence bounds and reserved compression symbols. Two UDP
tests establish original context use and rejection of unknown original protocols;
one compile-fail example prevents history release during decoded-view inspection.
All caller decoded output and history remain unchanged on decoding errors; private
scratch may be modified and is never consumed as a result on failure.

Eight auxiliary-body tests add independent native-shaped ACK/data vectors, exact
maximum size, every short output capacity, payload privacy, all flag/window-hint
bits, range counts/domains, truncation and exact suffix validation. A UDP test
checks qport/family admission before parsing without automatic ACKs, channel
reservation or window changes. The neutral crate has no dependencies. Thirteen
pinned client-reference hashes pass locally, including both reliable source files;
the historical server-reference entries were preserved without accessing server
worktrees. Temporal ACK validation and complete stream transport are not yet covered.

Ten receive-window tests cover transactional construction/preparation, caller
budgets, dirty storage/privacy, independent streams under reverse arrival and
repeated modulo reuse, full binary fragments, conflicting duplicates, gaps,
incremental reads, empty-fragment work limits, selective ACK pagination, maximum
signed-short geometry and signed sequence exhaustion without wrapping. A further
UDP test parses exact bodies before retaining reordered data, rejects malformed
lengths, ignores conflicting retransmissions and drains the original byte stream.
Its outgoing queue is empty and admits only ACK zero with no selective ranges;
it does not exercise a general outgoing ACK validator or retransmission runtime.

Eight application-framing tests add independent exact length/body vectors, output
atomicity, every three-chunk partition including partial length headers, retained
frame borrows, concatenation with one-frame consumption, zero-body framing,
dirty-storage reuse/privacy and terminal rejection of negative/over-budget lengths.
A native-size 1 MiB message drains incrementally through three reordered slots,
without requiring the receive window to hold the whole message. A compile-fail
example prevents body-buffer reuse during complete-frame inspection. Another real
UDP test assembles a complete application body after out-of-order one-to-three-byte
length-header pieces. Neither framing nor this fixture executes an application opcode.
The process-test responder now waits for explicit child completion rather than a
fixed sleep. Its unsupported-profile probe uses a practical one-second deadline;
deterministic core tests continue to check exact retry/expiry boundaries.

Twelve outgoing-window tests cover transactional storage and enqueue geometry,
exact binary fragment sizes without empty terminators, separate queue/flight spans,
actual-send admission, cancellation, stable retransmissions, unsent ACK holes,
late-invalid selective ranges, overlap/duplicate counting, retained selective bytes,
paired receive/ACK cancellation, maximum signed fields and sequence exhaustion.
An independent 50,000-byte loss/reorder/duplicate model repeatedly reuses seven
outgoing slots and must reconstruct the exact stream. This establishes model
state behavior; the following tests extend the evidence to actual UDP I/O.

Four auxiliary UDP-send tests add exact independent data/ACK vectors on the
original socket/qport, successful-I/O-only admission, expired sends without sent
authority, stable retries followed by validated peer consumption, partial-fragment
selective ACKs, host-controlled window hints, and profile/metadata rejection before
I/O. Original protocols 6 and 22 cannot emit this reviewed auxiliary profile merely
because a challenge advertises protocol 18. Saturating send totals/subset counters
are checked; no ordinary sequence is reserved. That checkpoint covers explicit
individual sends; the following tests extend it to paced retry/ACK scheduling.

Eight scheduling tests add exact-clock configuration and timer bounds, paced
round-robin retry identity, combined fragment/timer cancellation on drop or callback
failure, periodic ACK progress under wholly selective suppression, no long-pause
catch-up burst, bounded validated force-ACK requests, monotonic/overflow checks
and ACK pagination/reset. A real UDP test couples scheduled I/O failure/success
to both leases, preserves original retransmission bytes, consumes a validated peer
ACK and sends an idle ACK. Scheduled profile rejection also preserves both owners.
Host ticks in the UDP fixture are scripted; a continuous gameplay session and live
ordinary-peer timing interoperability have not been tested.

Six endpoint tests add atomic construction across all buffers/timers, combined
receive/ACK/force-clock admission, cancelled invalid force requests, bounded
failure-work records, terminal framing failure, partial prefixes, empty-fragment
quotas and exact boundaries between concatenated messages. A two-endpoint model
exchanges a 5,000-byte body and a reply under loss, duplicate delivery and receive
backpressure. Two framed-enqueue tests cover binary prefix/body boundaries and
capacity failure before any partial prefix write. One UDP fixture checks combined
invalid/valid packet admission, complete body delivery and consumed ACK emission.

Six connection-owner UDP tests add original socket/profile/endpoint coupling,
deadline failure without data admission, exact framed exchange, wrong routing,
body/ACK rejection, scoped ordinary/connectionless callbacks with hidden debug
results, independent ordinary sending, actual receipt time after delayed I/O,
expired waits without work or timestamp refresh, terminal framing failure before
more socket operations, and profile/storage/timer failures preserving supplied
buffers. Counters include work before failure and count terminal transitions once.
The ordinary sender wrapper exposes no external auxiliary windows or schedulers.

Seven body-reader tests add independent cached-bit vectors around 128 KiB and
beyond 1 MiB, a differential mixed primitive trace against shared readers, large
byte copies, shared big-string normalization, exact input/output failure atomicity,
invalid bit widths, missing terminators, concrete shared entity/client index forms
and joint cache/index rollback. The adapter allocates nothing and changes storage
views only; pinned shared code still owns every field's wire decoding.
