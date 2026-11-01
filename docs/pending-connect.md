# Pending no-auth connect state

The core now owns a flat pending-connect controller over an immutable request plan.
It admits retries under one absolute phase deadline and cumulative send/receive
quotas. Complete packet bytes remain identical across retries, and a response
after the last attempt can still succeed before expiry.

Replies advance state only after at least one request was admitted and only from
the plan's exact peer IP and port. Unsolicited, wrong-peer, malformed and unrelated
packets consume inspection work without admitting a connection. Valid print
messages are informational; they neither admit/refuse nor extend the deadline.
An error response is a distinct terminal refusal, without evaluating native markup
or activating authentication/version-fallback behavior.

The last receive-quota slot may acknowledge or refuse. A nonterminal packet in
that slot exhausts the phase instead. Acknowledgement, refusal, expiry and quota
exhaustion prevent any later acknowledgement or retry from being published.
Capacity and backwards-clock failures preserve bytes and state. Wide deadline
arithmetic rejects overflow and clamps retry timing rather than wrapping.

Validated observations retain the original requested protocol and routing qport
for later channel initialization. These are not independently negotiated values.
Mod labels and diagnostic payloads remain borrowed untrusted bytes, with no path
or command authority. Debug contains only fixed kinds, timing and counters.

The runtime must retain the same bound UDP socket across challenge and connect
phases, preserving the return path under the peer's address/cookie policy. The
standalone challenge-probe command closes its socket when done; its one-shot I/O
wrapper is not the connection runtime. A subsequent channel handoff must own the
socket and relevant metadata together. An acknowledgement alone is not a playable
session: channel, stats/gamestate, content and simulation initialization remain.

Eight new tests cover pending/peer correlation, identical retries and last-attempt
replies, notice/refusal separation, absolute expiry under traffic, every output
prefix, complete receive quotas and clock/budget boundaries. This checkpoint passed
52 tests in debug/release plus denied-warning Clippy/docs, formatting and repository
policy. It also passed hosted Windows/Linux CI. Subsequent socket/channel work and
the latest test counts are recorded in [validation](validation.md); ordinary peer
joins remain a separate gate.
