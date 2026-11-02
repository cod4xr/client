# Reliable pacing and ACK scheduling

`cod4xr_reliable::schedule` selects one data/retransmission or ACK-only datagram
over explicit monotonic host time. It owns no OS clock, sleep or automatic retry
loop. Defaults use the native four-packets-per-second data interval and 350 ms
ACK interval, with an additional host policy minimum of 20 ms between any two
successful datagram admissions. Intervals are configurable, positive and checked
for consistency; peer flags and window hints cannot change them.

`ReliableScheduler::poll` returns a future `WaitUntil` point or one exclusive
`ScheduledDatagram`. Data rotates through original active queue IDs, skipping
selectively acknowledged fragments. Successful data carries current receive ACKs
and refreshes the ACK timer. If all queued data is selectively acknowledged, idle
ACKs still progress. A long pause creates no accumulated send credits: deadlines
are calculated from the supplied current tick, so repeated polls at that same
tick cannot produce a burst after successful admission.

The lease holds both the scheduler and the selected original fragment through
one `send_with` callback. Header, sequence and future timers are preflighted before
offering I/O. Only a callback reporting complete success marks data transmitted
and commits timer/cursor changes; both commits are then infallible. Dropping the
lease or returning an error changes neither owner. `ClientSender::send_scheduled_reliable`
uses the existing original-socket/shared-envelope send primitive as that callback.
Its existing original-profile gate applies before I/O.

Timing is anchored to the supplied host attempt tick. Hosts must supply fresh
monotonic observations, enforce their own receive/work quotas and decide how to
handle I/O failures. A wait observation records elapsed host time; a cancelled
send plan does not admit its time or rate budget. Regressing time and overflowing
future timers fail before mutation. Construct a fresh scheduler with fresh windows
for each connection.

Call `observe_validated_packet` only after whole incoming receive/ACK validation
and commit. A validated force-ACK flag makes ACK work eligible earlier, while the
global interval still applies. Repeated same-tick requests cannot bypass it. An
eligible data packet can carry that ACK without an extra ACK-only send. Unparsed
or rejected packet flags must never reach this notification.

Receive ACK metadata is derived directly from the retained receive window. The
scheduler rotates past the first three selective ranges after successful sends,
and restarts pagination when the consumed base changes. Failed sends advance
neither pagination nor the outgoing fragment cursor. Debug output contains timing,
sequence and bounded ACK metadata, never payloads, qports or connection cookies.

Eight exact-clock tests cover timing/configuration boundaries, round-robin retry
identity, cancelled/failed callbacks, idle ACK progress, whole-span suppression,
no catch-up burst, forced-ACK rate limits, clock errors and selective pagination.
A real UDP test adds expired scheduled I/O, successful original retransmissions,
validated peer consumption and a subsequent idle ACK under scripted host ticks.
Unsupported-profile tests also reject scheduled sends without either admission.
[Unified endpoints](reliable-endpoint.md) now combine both windows, framing and
schedule ownership, including preflighted packet observations. [Connection owners](reliable-connection.md)
now provide the socket and actual elapsed clock. An ongoing game loop, typed
gamestate/application dispatch and live ordinary-peer interoperability remain open.
