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
