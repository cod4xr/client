# Compatibility evidence

The product target is ordinary CoD4/CoD4X servers permitting no-auth clients,
including servers that never implement CoD4XR extensions. Auth-required servers
are unsupported by user choice. Compatibility requires named peers and observed
sessions; a source header, successful query or compatible-looking version number
does not prove a playable connection.

| Area | Evidence now | Remaining gate |
| --- | --- | --- |
| Discovery request | Exact OOB/getinfo byte vector; shared packet codec | Named ordinary server captures |
| Discovery response | Complete info validation, correlation, unknown bytes, bounds | Version-specific real peer matrix |
| UDP runtime | Real loopback responder/process tests; noise, endpoint, size, timeout/budget checks | Ordinary CoD4X server trials |
| No-auth connect | Challenge probe, owned requests, reply decoding and pending-state correlation | Persistent socket/channel handoff and ordinary peer traces |
| Channel/snapshots | Shared primitives available, not integrated | Reliable transport, gamestate, snapshots and input replay |
| Content/render/simulation | Not implemented | Stock/custom content and complete playable sessions |
| Optional extensions | No wire behavior invented | Negotiated features and ordinary-server fallback |

The pinned public client reference is `CoD4x_Client_pub` revision
`17182c7bf04b859dea5c0db4acdb0b83c28d6dd2`. Its README explicitly limits support
to servers with client authentication disabled; it builds a `cod4x_021.dll`
modification rather than a standalone Rust engine. It is research material only;
its runtime, assembly, assets and license terms have not been imported.

Observed source paths:

- Client `src/cl_main.c`: getinfo requests, infoResponse dispatch and
  CL_ServerInfoPacket read/discovery filtering.
- Client `src/q_shared.h`: legacy protocol 6, update version 21.8 and join-protocol
  macro. These declarations do not certify a live server matrix.
- Client `src/net_chan.c`: NET_OutOfBandData sends exactly the marker and supplied
  bytes; its Huff_Compress call is commented out in this pinned profile.
- Server `src/sv_main.c`: SVC_Info echoes the challenge, permits at most 128 bytes,
  advertises protocol 6 and sends `infoResponse` followed by normal info data.
- Server `src/netchan.c`: NET_OutOfBandPrint sends the four-byte marker plus text,
  excluding its C terminator.

`upstream.lock.json` records normalized source hashes and exact revisions. Run:

```powershell
python tools/verify_upstream.py C:\Users\Ethan\source\cod4x-client-reference C:\Users\Ethan\source\cod4x-reference
```

Current tests execute the Rust implementation and independent loopback responder
services. They do not execute the native client, renderer, authentication service
or original gameplay handshake. Parser choices such as rejecting ambiguous
challenge keys and trailing garbage are explicit malformed-input handling.
