# Connect reply data

The core now decodes the selected connectionless connectResponse, error and print
shapes over the shared envelope/command primitives. Parsing alone grants no peer,
pending-state, time, identity or gameplay authority. The pending-connect controller
must check those facts before advancing a connection.

An acknowledgement carries an optional mod label. It remains byte data, including
relative-looking labels and non-UTF-8 bytes; nothing resolves or opens it as a
filesystem path. Extra fields and hidden data after an acknowledgement delimiter
reject. Asset mounts require their own validated virtual-name and root policy.

Refusals and informational messages are distinct. Their diagnostic bytes can
contain line breaks, native OOBErrorParser markup and private values. They are
preserved for a future display layer, never interpreted as commands or automatic
authorization/version-fallback requests. Debug and errors contain fixed categories
and lengths instead of payloads. One terminal C NUL is allowed; embedded terminators
with hidden trailing bytes reject.

The pinned public client reads refusal text into MAX_INFO_STRING storage and print
text into an 8192-byte buffer. The selected payload limits are therefore 1023 and
8191 bytes excluding a C terminator. The entire print-profile datagram is bounded
before parsing. These are named legacy diagnostic profiles, not limits on internal
content, image catalogs or future negotiated features.

Five tests cover bare/quoted/non-UTF-8 acknowledgement labels, opaque refusal and
notice text, hidden-data and extra-field rejection, exact destination capacities
and unrelated/malformed packets. All 44 client workspace tests pass locally in
debug and release with denied-warning Clippy/docs, formatting and source policies.
The earlier 39-test request-planning checkpoint also passed hosted Windows/Linux
CI. No ordinary-server gameplay join or path/command authority is claimed here.
