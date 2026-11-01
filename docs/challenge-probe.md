# Bounded challenge probe

The deterministic client core now owns a pending challenge probe over the shared
`ChallengeRequest`/`ChallengeResponse` codecs at revision
`c3bd151068465a40e11e3db9d616789fdffa146f`. The named response profile is
`0 xproto 18`; it is a compatibility marker, not a selected gameplay protocol.
Unmarked legacy responses and other profiles do not silently trigger fallback.

The host supplies a selected endpoint, fresh random signed nonce and monotonic
time. The core owns one absolute deadline, admitted-attempt count, receive quota,
next retry and terminal state. No sockets, wall clocks or allocations are owned.
Retries keep the same nonce and never reset the deadline or resource counters.
After the last request attempt, valid responses are still permitted until expiry.

Receiving charges every inspected packet, including wrong endpoints, unsolicited
responses, malformed commands and incorrect nonces. Acceptance requires a pending
request, exact IP and port, the named profile, the echoed nonce and time before
expiry. The last allowed received packet can succeed; the last rejection exhausts
the receive quota. Acceptance or failure is terminal. A receipt is returned once;
its cookie stays out of Debug, errors and statistics and grants no server identity
or gameplay admission.

Output-capacity failure preserves request bytes and probe state, including the
last observed host clock. A backwards host clock rejects without mutation.
Deadline overflow rejects construction; retry arithmetic clamps to the admitted
deadline instead of wrapping. Wide time and quota domains are separate from
legacy engine globals or game resource limits.

Eight deterministic tests exercise pending/terminal transitions, unsolicited and
wrong-peer traffic, nonce/profile rejection, retry timing and late replies,
absolute expiry, every request-capacity prefix, monotonic-clock recovery, every
small receive quota and wide arithmetic. These tests establish core behavior,
not an ordinary-server gameplay join. The CLI challenge I/O adapter follows next.

Local acceptance: all 24 client workspace tests pass in debug and release;
all-target Clippy, denied-warning rustdoc, formatting and repository policy pass.
The policy now checks main/feature branch naming and full hexadecimal dependency
revisions. No server source files were edited for this client checkpoint.
