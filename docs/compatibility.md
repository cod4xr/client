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
| Channel/snapshots | Ordinary transport, control/payload/service validation, configurable command histories and socket/clock/endpoint ownership with paced UDP I/O | Game loop, typed execution, gameplay snapshot admission, input replay and ordinary peer sessions |
| Typed pre-game controls | New-command-only disconnect/map/restart notices, native token/copy rules, budget cancellation and next-frame UDP key/ACK coverage | Session/content/UI effects, remaining typed handlers and ordinary peer captures |
| Entity delta records | Both coordinate profiles, explicit legacy context, nonzero references, all target layouts, owned results and joint cursor/index rollback | Player-state decoding, whole snapshot/message admission, host reference ownership and native peer traces |
| Snapshot entity substreams | Native terminator, complete sparse merge, prior/baseline/zero references, whole-output rollback, cooked-source reuse and retained data beyond the native ring | Player-state decoding, full snapshot/message admission, reference epochs and native peer traces |
| Snapshot client-state records | Both profiles, all 24 reproduced fields, explicit prior/zero references, packed raw names, native end-bit framing and whole-record reader/index rollback | Epoch-bound snapshot/message admission and native peer traces |
| Snapshot client-state substreams | Complete native sparse merge, all 64 wire indices, separate end bit, prior/zero references, whole-output rollback, independent budgets and cooked-source reuse beyond the native ring | Reference epochs, full snapshot/message admission and native peer traces |
| Player-objective sections | Both profiles, all 16 wire slots, separate status/word changes, reproduced six-field schema, native reader comparison, owned prior/zero references and whole-section rollback | Typed resource/status validation, reference epochs and full snapshot admission |
| Player stats/ammunition/model tables | Both profiles, native signed/unsigned widths, complete masked groups and raw tables, prior/zero retention, explicit wire projections and whole-section rollback | Prediction production, reference epochs, typed inventory/resources and full snapshot admission |
| Player-HUD sections | Both arrays/profiles, all 40 reproduced fields, explicit legacy context, native numeric/tail rules, owned references and whole-section rollback | Prediction production, typed rendering/resources, reference epochs and whole snapshot/message admission |
| Live player-scalar prefixes | Both profiles, 141 reproduced fields and closed typed keys, native hints/value rules, owned prior/zero references, prediction-request provenance and whole-prefix rollback | Prediction production, reference epochs and complete snapshot/message admission |
| Complete live player state | Both profiles, complete section composition, exclusive reader lease, exact prediction-sample matching or explicit prior fallback, owned results and whole-state rollback | Prediction production/session-bound queues, typed gameplay/resources, reference epochs and complete snapshot/history/message admission |
| Snapshot headers | Both profiles, independent six-byte vectors, all time/flag bits and delta distances, cached-bit preservation and immediate player-state handoff | Reference sequence/epoch/history resolution and complete snapshot/message admission |
| Complete snapshot wire body | Both profiles, all four constituents, coherent cooked prior sources, exact message/epoch binding, prediction resolution, whole-reader/two-output rollback and caller-funded history beyond the native ring | Joint enclosing-message/ACK admission, host session/history ownership, prediction production, typed simulation and ordinary peer captures |
| Gamestate configstring blocks | Independent vectors for both profiles, whole-block validation, caller-funded storage, complete namespaces and large bodies | Engine application and ordinary peer captures |
| Client identity records | Owned protocol-21 gamestate/update vectors, exact text capacities, sequence-bit preservation and whole mixed-message rollback | Legacy adapters, remaining services and ordinary peer captures |
| Client identity state | Atomic gamestate replacement, exact-next batches, local generations, ordered-map traces, joint UDP frame/identity commit and mixed-message staging | Sibling session resources, legacy configstring adapters and real peer captures |
| Initial entity baselines | Both coordinate profiles, explicit checked legacy map centers, all eighteen reproduced layouts, owned words and atomic index/message admission | Legacy gamestate context integration, native peer captures, engine interpretation and complete snapshots |
| Complete gamestate constituent | Protocol-21 header/records/trailer, whole-output rollback, full wire namespaces and configurable retention | Outer-message admission, native defaults/content, complete legacy carrier/context, engine application and real peer captures |
| Reliable gamestate body | Immutable wrapper/tail evidence, complete-output rollback, reordered UDP frames, cooked plans and joint identity-baseline commit | Remaining session resources, native defaults/content, application ownership and ordinary peer captures |
| Typed frame ownership | Original-profile preparation, exclusive frame identity, cancel/commit behavior, work counters and real UDP tests | Session epochs, engine/content preflight and ordinary peer captures |
| Native defaults | Exact installed-image identity, checked PE32 data views, caller-funded raw imports and local 832-record inspection | More reviewed images, engine/content application and ordinary peer sessions |
| Effective config state | Native restoration/persistence, sorted sparse snapshots, wide IDs, 256 model traces, work/storage budgets and wire-view integration | Session epochs, engine/resource interpretation and ordinary peer sessions |
| Native system info | Shared big-info validation, full-width decimal server IDs, host budgets, raw unknown fields and wire-to-snapshot integration | Session epochs, resource/CVar policy and real ordinary peer sessions |
| Classic IWD catalogs | Checked stored/Deflate metadata, local/central consistency, overlap rejection and independent installed-archive counts | Virtual mounts, pure checksums and asset loading |
| Checked archive payloads | Exact stored/raw-Deflate lengths, full stream consumption, CRC32, rollback and explicit verification command | Host mount ownership, native pure checksums and asset loading |
| Virtual file keys | ASCII case/slash equivalence, raw bytes, wide host-funded names, atomic output and mutation checks | Native request sanitation, host mount ownership and physical asset resolution |
| Copied mount index | Explicit priority, native duplicate precedence, sparse IDs, model traces and catalog-to-payload integration without source ZIP retention | Native search order, pure policy and actual asset loading |
| Owned archive host | Original retained handles, process-unique epochs, real file mutation/rename tests, bounded reads and copied-index-to-payload integration | Native search order, pure policy, asset-format interpretation and engine lifecycle |
| IWI6 images | Checked direct layouts, bounded five-format wavelet reconstruction, explicit channel order, synthetic cube/volume vectors, private image inspection and installed pixel hashes | Wavelet volume/non-power-of-two support, GPU upload/rendering and real gameplay presentation |
| Fastfile5 containers | Explicit unsigned PC profile, complete zlib/Adler-32 checks, exact decoded lengths, inert wide block metadata and private inspection | Serialized pointer/asset interpretation, other profiles, map/model resource loading and rendering |
| Zone5 root catalogs | Bounded root/string/asset records, cooked publication, all native type labels, inert serialized links and separate physical/virtual accounting | Script-string offset/insert aliases, child assets, target relocation, engine registries and gameplay rendering |
| Content/render/simulation | Checked defaults/IWD data and copied lookup; engine map/model loading, rendering and simulation remain open | Stock/custom asset loading and complete playable sessions |
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
