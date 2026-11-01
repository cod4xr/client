# Owned no-auth connect requests

The core now constructs immutable connect datagrams from a checked challenge
receipt and explicit caller-selected settings. It uses the shared InfoWriter,
InfoString and OOB packet codecs rather than copying wire implementations.
There is no socket activity, authorization call, invented GUID or player admission
in this component. Connect retries and acknowledgement handling remain separate.

The plan binds its destination to the checked receipt's exact peer. Protocol is
an explicit positive signed value, independent of discovery's protocol 6 and
the challenge's xproto 18 marker. Compatibility version bytes are also explicit.
The qport is encoded with the signed-short spelling used by the pinned public
client, retaining all 16 routing bits. This is syntactic profile construction,
not proof that a selected server supports the requested protocol or version.

Caller userinfo is validated through shared primitives. Protocol, challenge,
qport and xver keys cannot be overridden through case aliases. Duplicate unknown
keys reject. Unknown nonempty values retain their bytes; empty values follow the
shared legacy setter's deletion semantics and are omitted. Private names,
passwords, identity claims and return-path cookies never enter Debug/errors.
The builder performs no identity/password validation or authorization.

Complete wire admission includes the four-byte envelope and both connect quote
delimiters. C-string scratch space is independent and is never sent. An exact
1023-byte command fits; one more byte rejects before publication. The plan owns
all its bytes after caller inputs drop. Encoding preflights complete output
capacity and leaves all output unchanged on failure. The normal legacy command
budget is a wire-profile boundary, not a new internal game/content limit.

The pinned client `src/cl_main.c` constructs `connect "USERINFO"` and supplies
protocol/challenge/qport/xver through its info setter. Its `src/net_chan.c`
NET_OutOfBandData sends exactly the envelope plus supplied bytes; the Huffman
compression call is commented out. Both files are included in the client's
source lock. No original client code is included in the runtime.

Eight new tests cover input ownership and private-byte preservation, all signed
cookie/qport boundaries, explicit protocol independence, empty-value deletion,
every output-capacity prefix, full command admission, conflicting/duplicate keys
and version-delimiter injection. All 39 client workspace tests pass locally in
debug and release, with denied-warning Clippy/docs, formatting and policy checks.
The earlier 31-test CLI checkpoint passed hosted Windows/Linux CI. Ordinary peer
joins, current hosted validation and acknowledgement handling remain separate.
