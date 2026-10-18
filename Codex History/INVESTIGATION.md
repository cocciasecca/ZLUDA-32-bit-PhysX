# Investigation

## Confirmed from source and public metadata

Fork HEAD and upstream master consulted on 2026-10-18: `86ec0a93cc83ba5b52e645892523e339dfa884f7`; no fork-specific tracked patch at start.
[Upstream tag API](https://api.github.com/repos/vosen/ZLUDA/git/ref/tags/v7-preview.15) resolves v7-preview.15 to that commit.
[Exact Proton release](https://github.com/UndeadBulwark/Proton-ZLUDA/releases/tag/tool-11.0-20261005-r2) describes v7-preview.15.
[Release convert.sh](https://github.com/UndeadBulwark/Proton-ZLUDA/blob/tool-11.0-20261005-r2/convert.sh) pins that tag and `zluda-windows-86ec0a9.zip`. This is provenance evidence, not installed-file verification. Cached/overridden payloads and local modifications remain possible.

Ordinary proxy cuMemFree cannot directly produce one hipFree per call at this revision: it only updates the range allocator. This narrows H1; it does not exclude internal HIP allocations, texture/module cleanup or transfer staging.

DtoH uses a fresh pageable Vec and an explicit stream wait. Proxy host allocations use calloc. Candidate: HIP handles pageable transfers through transient pinning/staging. Need HIP/HSA user stacks at KFD unmap entry, correlated with API counts and frame times. There is currently no evidence establishing that path on the target.

## Measurement design

Time server request work BEFORE SignalObjectAndWait; wrapping the entire dispatch would incorrectly include client idle time. Prototype includes decode, handler and response serialization; excludes notification/wait, full client IPC latency, and diagnostic writes. Aggregation writes still delay the response and can perturb timings; compare instrumented versus uninstrumented runs.

Next attribution capture: inspect loaded HIP/HSA library paths and build IDs from /proc/<pid>/maps; collect symbolized userspace stacks for unmap in a short approved run, including all server threads. Compare hipMemcpyHtoDAsync/DtoHAsync, hipFree/FreeHost, registration and module unload; distinguish direct synchronous ownership from worker-thread cleanup. Aggregate counts alone cannot establish per-event causality.

## Unknown / blocked

SSH name resolution fails; no current target observation. Installed binary SHA/version and actual runtime libraries unknown. No runtime causal attribution, supported optimization, graphics validation or frame-time improvement. Phase A lacks HIP event/lifetime/byte counters and client round-trip measurements; extend after obtaining target traces and validating this prototype.

## Target verification update

SSH access now works at 192.168.10.67. Installed server embeds the exact fork commit 86ec0a93cc83ba5b52e645892523e339dfa884f7. Installed-file hashes recorded in PROGRESS.md; release archive equality not yet checked. COMGR dependencies resolve. No game/server found by process-name check, so runtime-loaded library provenance and causal attribution remain pending.

## First live scene: police tape and electrical particles

User reports approximately 28 FPS versus earlier approximately 21 FPS in police-tape scene, with electrical spark/particle effects also visible (present in earlier run but previously unreported). User attributes these to PhysX; exact effect ownership not independently verified. Not a matched A/B or measured FPS trace; no optimization conclusion.

Confirmed running experimental server (Linux PID 4308), matching prefix DLL SHA256 0901df0c0a0c1a3bd995f93b681abc20071b82295fbfa44a8cd83fa6ad949c82, and experimental hipwine/HIP/HSA/COMGR paths in /proc/4308/maps. GPU/KFD handles present; this alone does not validate individual effects.

Diagnostic failure found: CSV only 150-byte headers, tagged Windows PID 416 (0x1a0). Proton log shows TWO server starts, Windows PIDs 0x1a0 and 0x594. Both inherit same per-Proton-invocation CSV path; create_new lets first server create file and prevents second from recording. Active server has no CSV fd. Existing portable tests did not cover launcher+game inheritance. Fix: append server PID to configured CSV base path, retain create_new/no overwrite. Three portable tests now pass including two-process-path collision protection and preservation of existing data. This run provides no CUDA timing breakdown. Need rebuilt matching proxy/server and user-approved restart/update before repeating. No current game interruption or runtime replacement.
