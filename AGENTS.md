# CoD4XR client contract

CoD4XR is a general-purpose client and server rewrite. Product-specific mods
consume optional features; they do not own the engine or baseline protocol.
The independent server repository is https://github.com/cod4xr/server.

- Implement the no-auth baseline. Auth-required servers are unsupported; never
  forge credentials, bypass an auth service or claim all-server compatibility.
- Reuse shared protocol/content/simulation code from explicit immutable server
  revisions. Do not fork wire codecs or depend on server-runtime crates.
- Runtime/OS/network/render/input ownership stays outside deterministic core code.
- Keep files focused, aim for 250 lines, hard maximum 400. Inherit workspace
  lints and metadata, forbid unsafe, and run format/Clippy/tests/docs/policy gates.
- Keep raw remote bytes out of logs. Human development logs and JSON production
  logs share stable event names and bounded counters. Never execute peer strings.
- Current deliverable is discovery, not a playable client. Keep compatibility
  profiles and demonstrated peers separate from product goals and advertisements.
- No game executables/assets, credentials, private settings or operator configs
  in Git. Public upstream client source is research material, not a copied runtime.
- Never create extra local Cargo target directories to avoid locks. Reuse the
  operator's existing target via tools/use-existing-target.ps1. Stop only this
  client's exact development executable if locked; do not kill unrelated builds.
- Preserve in-progress server work; it is a separate repository.
- Do not use Graphify. Do not publish/push unless requested.
