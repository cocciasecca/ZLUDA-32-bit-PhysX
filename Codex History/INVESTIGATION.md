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

## First successful in-scene request profile — police tape

User confirms identical position/scene/settings, reports approximately 25 FPS (user observation, not captured frame-time mean). Server Linux PID 5156; active CSV Windows PID 592, revision e8541ca. Launcher PID 416 CSV remains header-only; game CSV now grows, confirming per-process filename fix. Capture: 30.029 s of read-only /proc/sysfs samples; five COMPLETE 5 s CSV windows strictly after the capture's initial last-completed-window boundary, totaling 25.0018556 s. Excluded partly elapsed first window. Files: measurements/police-tape-2026-10-18/{selected-windows.csv,summary.json,samples.json}. Full raw CSV/maps retained locally at /private/tmp/zluda-scene-capture/police-tape-30s.json.

- cuTexRefSetAddress_v2: 64,151 calls; 2,565.85/s; 18.4036043 s service time; 286.88 us mean; 1.7586 ms maximum; **94.42% of total measured request service time**.
- cuMemcpyDtoHAsync_v2 (includes internal stream synchronization): 0.6888136 s, 3.53%.
- cuMemcpyHtoDAsync_v2: 0.1838876 s, 0.94%.
- cuLaunchKernel: 18,083 calls; 0.1360875 s host service time, NOT GPU kernel duration.
- All recorded request statuses zero in selected windows. No cuMemAlloc/cuMemFree requests in those windows.
- Total measured service time 19.4912619 s over 25.0018556 s; excludes IPC wait and diagnostic writes, includes decoding/serialization and nested calls.
- Over 30.029 s sample: server CPU 81.12% of one core; main-thread context switches 7,005.94/s (not all-thread aggregate); RSS 231,420 -> 231,320 kB. GPU busy arithmetic mean of 31 approximately 1 Hz aggregate samples 69.55% (not dedicated compute utilization).

Source trace: zluda64_server cu_tex_ref_set_address_v2 -> cuTexRefSetAddress_v2 -> zluda/src/impl/tex.rs::ref_set_address_v2 -> hipUnbindTexture for null pointer, hipTexRefSetAddress otherwise. Current counter groups bind AND unbind; cannot distinguish their rates or time. Flags/format setters already refresh only when changed. Need separate null/non-null, binding identity and bytes/redundancy counters or HIP tracing before optimization. At roughly 3 address calls per format/flags call, a bind/unbind pattern is plausible but NOT established.

This supersedes pageable transfer overhead as the first investigation target for this scene. Similarity between 2,566 address calls/s and historic approximately 2,417 KFD unmaps/s is a lead, not a causal match: captures are not simultaneous. No KFD measurement or userspace unmap stacks taken in this run. No frame-time improvement demonstrated. Next: attribute bind/unbind costs and KFD stacks; consider caching only with texture state, null-unbind semantics, kernel ordering and resource lifetime preserved. Do not skip synchronization or blindly cache texture addresses.

## Texture lifecycle follow-up

User authorized autonomous technical direction; no authorization inferred for interrupting live game or privileged tracing. Target strings in libamdhip64.so.7 identify build source paths /usr/src/debug/hip-runtime/rocm-systems-rocm-7.2.4/... . This refines earlier generic HIP 7.2 notes; COMGR remains separately supplied 7.2.3. Build paths are provenance evidence, not proof that binary exactly matches unpatched upstream source.

Official ROCm tag rocm-7.2.4 source inspected:
- https://github.com/ROCm/rocm-systems/blob/rocm-7.2.4/projects/clr/hipamd/src/hip_texture.cpp : hipTexRefSetAddress destroys prior texture object, creates another, then copies reference to device. ihipCreateTextureObject allocates descriptor via ihipMalloc(sizeof(__hip_texture), CL_MEM_SVM_FINE_GRAIN_BUFFER). ihipDestroyTextureObject releases image/sampler and calls ihipFree. hipUnbindTexture also destroys texture object and nulls handle.
- https://github.com/ROCm/rocm-systems/blob/rocm-7.2.4/projects/clr/hipamd/src/hip_memory.cpp : ihipFree calls SyncAllStreams and memory release/SvmBuffer::free if allocation is not handled by memory pools.

This is a concrete source-level candidate for synchronization/allocation churn hidden under texture APIs, consistent with measured hotspot. Runtime KFD attribution remains unconfirmed. Do not assume public hipFree uprobes catch internal ihipFree calls.

Implemented bounded, feature-gated TextureHistory classification: bind unknown, rebind same address+size, bind same after unbind, changed address+size, unbind bound/already-unbound/unknown. Tracks only successful requests, resets at memory free/module unload and at 4096-entry bound. Equality explicitly excludes format/flags and resource/content identity: it is diagnostic, not a safe-cache predicate. Labels replace generic cuTexRefSetAddress request name in same aggregate CSV; no nested/double-counted timing. Classification occurs before timer; successful history update included in handler service time. Default build omits tracking. All CUDA calls, synchronization and error results retained.

Three new portable tests pass (bind/unbind sequence/size changes, failed-call nonmutation/invalidation, capacity bound), plus three existing collector tests pass. Workflow tests new module on Windows too. actionlint passed with existing custom runner configuration, diff check passed, changed Rust files formatted. No device runtime replaced; game still active at last check. Next Windows build then matched proxy/server update when user closes game; targeted repeat capture will establish which texture operation dominates before proposing reuse.

## 2026-10-19 — texture lifecycle profile, police-tape scene

User returned to scene after update. Verified prefix x86 nvcuda SHA256 db825c4c0c6d4e3d428d5ea6d6b3bb1cd12f547561080a7d49ffe574e6b8802e, active server Linux PID 8396 and growing CSV Windows PID 1400 at fa5757a. Captured 30.0293 s; five fully contained aggregate windows total 25.0021308 s, excluding initial partly elapsed window. Raw capture /private/tmp/zluda-texture-capture/police-tape-30s.json; selected windows/summary/samples preserved in measurements/police-tape-textures-2026-10-19/.

- unbind_bound: 20,480 successful calls, 819.13/s, 16.0131859 s, mean 781.89 us, **82.20% of total measured request service time**.
- bind_same_after_unbind: 20,472 successful calls, 818.81/s, 2.4217113 s, mean 118.29 us, **12.43%**.
- unbind_already_unbound: 20,472 calls, 0.0093581 s, mean 0.457 us, only 0.048%. Optimizing redundant empty unbind alone would not address hotspot.
- DtoH async incl stream wait: 0.642118 s, 3.30%; HtoD async 0.1897017 s, 0.97%.
- No bind_changed_address_size or direct rebind_same_address_size rows in selected windows. All observed non-null binds reuse last successful address+size after unbind. This does NOT prove same format/flags, contents, memory generation, or a safe lifetime shortcut. 8-call difference can straddle window boundaries; do not infer exact per-texture sequence count.
- Total service 19.4808581 s. Server CPU 80.55% of one core; main-thread context switches 6,651.30/s; RSS 231,980 -> 231,944 kB; 31 approximately 1Hz aggregate GPU-busy samples average 69.52%. GPU kernel launch count differs from earlier capture (14,224 here vs 18,083 earlier); cannot assume workload equivalence solely from same scene. No measured frame-time trace or new FPS claim.

Evidence now narrows costly CUDA operation to null-address unbind -> HIP texture destruction/free, rather than repeated no-op unbind, plain device free or bulk copies. Source HIP 7.2.4 destruction releases image/sampler/descriptor, candidate for KFD churn. Runtime KFD call attribution and proportionality are STILL unmeasured. Historical ~2,417 unmaps/s is from a different session and must not be treated as matched with current ~819 texture destroys/s.

Next engineering step: investigate bounded texture resource reuse preserving logical unbound state, CUDA synchronization, flags/format/offsets, module and allocation lifetimes, error behavior and kernel visibility. Do not simply omit hipUnbindTexture; it currently includes implicit memory-free synchronization in HIP. Must retain required ordering and validate with focused lifecycle tests and same-scene frame-time A/B. No optimization installed. User can leave scene; no further live capture needed for this data point.
