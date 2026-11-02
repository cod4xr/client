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
| No-auth connect | Owned challenge/connect/acknowledged socket transitions, retry stability, inert refusals and bounded state | Clean disconnect and ordinary peer traces |
| Channel/snapshots | Ordinary transport, control/payload/service validation, configurable command histories and socket/clock/endpoint ownership with paced UDP I/O | Game loop, typed execution, gamestate, snapshots and input replay |
| Gamestate configstring blocks | Independent vectors for both profiles, whole-block validation, caller-funded storage, complete namespaces and large bodies | Engine application and ordinary peer captures |
| Client identity records | Owned protocol-21 gamestate/update vectors, exact text capacities, sequence-bit preservation and complete-record rollback | Whole-message validation, session sequence admission and ordinary peer captures |
| Initial entity baselines | Protocol-21 field vectors, all eighteen source-reproduced layouts, owned words and atomic index/message admission | Native peer captures, legacy coordinates, engine interpretation and later snapshot deltas |
| Complete gamestate constituent | Protocol-21 header/records/trailer, whole-output rollback, full wire namespaces and configurable retention | Outer-message admission, native defaults/content, legacy coordinates, engine application and real peer captures |
| Reliable gamestate body | Immutable wrapper/tail evidence, complete-output rollback, reordered real UDP frames and source-independent cooked plans | Session admission, native defaults/content, application ownership and ordinary peer captures |
| Typed frame ownership | Original-profile preparation, exclusive frame identity, cancel/commit behavior, work counters and real UDP tests | Session epochs, engine/content preflight and ordinary peer captures |
| Native defaults | Exact installed-image identity, checked PE32 data views, caller-funded raw imports and local 832-record inspection | More reviewed images, engine/content application and ordinary peer sessions |
| Effective config state | Native restoration/persistence, sorted sparse snapshots, wide IDs, 256 model traces, work/storage budgets and wire-view integration | Session epochs, engine/resource interpretation and ordinary peer sessions |
| Content/render/simulation | Native default import only; map/model loading, rendering and simulation remain open | Stock/custom content and complete playable sessions |
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
- Client `src/net_chan.c` and `src/client.h`: ordinary sequence initialization,
  qport direction, 1300-byte fragments, zero-length terminators and 128 KiB receive storage.
- Client `src/cl_netchan.c`: profile-specific payload-header offsets and message
  obfuscation depend on the original challenge; the ordinary channel retains that
  private context for the upcoming message layer without decoding gameplay yet.
- Client `src/qcommon.h`, `src/msg.c` and `src/qshared.c`: 128-command history,
  1024-byte slots and native reliable-command cleaning (0x92 to apostrophe, percent
  to period). The rewrite preserves the acknowledged key separately from ring reuse.
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
