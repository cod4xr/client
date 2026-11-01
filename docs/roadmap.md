# Client work

Each gate needs observable behavior and recorded evidence. Source research,
scaffolding and a successful query are not substitutes for a playable client.

| Phase | Scope | Acceptance |
| --- | --- | --- |
| Foundation | Separate repo, shared immutable core, discovery, logs and standards | Exact bytes, hostile-input tests, real local UDP/process checks; implemented |
| No-auth connection | Version profiles, challenge/connect/retry/disconnect ownership | Ordinary server reaches an accepted connection with bounded state |
| Transport | Reliable commands, fragments, gamestate and snapshots | Captured peer traces, loss/reorder and malformed-message workloads |
| Content | Installed stock assets, archive mount order, maps and downloads | Stock/custom content loads with checked imports and pure validation |
| Presentation | Window, rendering, input, audio and UI | Correct frame/input behavior on target hardware |
| Simulation | Prediction, collision, movement, weapons and snapshot reconciliation | Recorded deterministic replays and live reference comparisons |
| Scripting/mods | Client-side presentation and expanded engine features | Named stock modes and existing mods run correctly |
| Release | Full sessions, version matrix, soak/performance/security | Supported ordinary no-auth servers play through complete sessions |

Next is the source-grounded no-auth connection lifecycle. Do not use discovery's
protocol-6 field alone to choose its gameplay dialect. Reuse message/channel
primitives while keeping challenge state and retry ownership in the client.
Capture the first compatible peer traces before claiming broad interoperability.

Features that remove asset/model/image/entity/configstring limits need both
internal storage/resource design and a negotiated extended wire/content profile.
Ordinary servers continue to use their baseline limits. Internal storage should
not inherit 32-bit engine globals merely because the legacy wire has a bound.
Do not silently reinterpret legacy identifiers or promise lifted limits to peers
that have not negotiated them.
