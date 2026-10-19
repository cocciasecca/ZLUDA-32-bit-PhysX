# Progress

## 2026-10-18 — source reconnaissance and diagnostic prototype

Read all seven original handoff Markdown files before changes; originals preserved.
Initial checkout: clean tracked files on master, HEAD 86ec0a93cc83ba5b52e645892523e339dfa884f7; Codex History initially untracked. No AGENTS.md found under the repository/parent search.
Created separate worktree `/private/tmp/zluda-physx-diagnostics`, branch `experiment/physx-diagnostics` (source changes only there). No push, device writes, game launches, prefix changes, or tracing configuration.

Confirmed source already suballocates from a lazy 512 MiB GPU arena. Bridge cuMemFree only releases a range; it does not call HIP. The direct per-CUDA-free -> hipFree hypothesis does not fit this revision. Pageable transfer handling and HIP-internal staging/registration now deserve priority, but neither is proven to cause KFD unmaps.

Verified public upstream master and v7-preview.15 both resolve to this HEAD. Exact Proton-ZLUDA release convert.sh pins v7-preview.15 / zluda-windows-86ec0a9.zip. Installed binary identity remains unchecked.

Implemented first opt-in server request service-time aggregation (feature + environment switch), excluding the wait for the next request. This is partial Phase A, not a performance optimization or complete HIP/lifetime instrumentation.

SSH read-only connectivity failed: steamdeck DNS resolution, including outside sandbox. Requested device IP. Mac has no installed Rust/Cargo or Windows build environment; installed Rust 1.90.0 compiler and standard library only under /private/tmp/zluda-rust-toolchain for portable diagnostic tests (no system installation). Full Windows component build, CUDA tests, runtime attribution and benchmarks remain pending.

Validation completed: standalone diagnostics.rs compiled with Rust 1.90.0 on aarch64 macOS; both unit tests passed (success/error/latency aggregation and independent request rows/window reset). git diff --check passed. These tests do not validate Windows integration or CUDA semantics. No installed target or runtime was modified. Source and notebook are preserved on the experimental branch; original checkout remains on master with notebook files only.

## 2026-10-18 — SSH access established

User installed dedicated public key; SSH to deck@192.168.10.67 now works with explicit identity ~/.ssh/zluda_legion_ed25519 (private key remains on Mac, never in repository). Read-only checks confirmed kernel from historical notes, tool version `1791146054 cachyos-11.0-20261005-slr`, and embedded server commit `86ec0a93cc83ba5b52e645892523e339dfa884f7`. No matching Batman/server process found in the process-name check; no game launched. COMGR ldd resolves dependencies. perf present; command lookup found no bpftrace, rocprofv3, rustc, cargo, clang or MinGW compiler. Tool occupies 2.0 GiB; /home has approximately 73 GiB available. No target writes by agent.

SHA256 under ZludaProton:
- zluda/zluda64_server.exe: 40303768cb583431cc0b7221709e6c2cfa89490cfede801e58b7351524401fdf
- zluda/nvcuda.dll: 3d9b46c4b43bfefebf41271a2cd68aee32ef6d870fcadf4a431fb103cecbe1a9
- zluda/32/nvcuda.dll: 072bac0639c776d8c5b8b6ef3a3c0e8aaebcd96e0a19b874057cfead131f6867
- zluda-libs/libamd_comgr.so.3: e765f0913bc94de361b8e471e22c5085b2e6d967510cb80c2084b031e3f4781c

These are installed-file fingerprints, not a comparison against release archive hashes. Loaded runtime paths remain to inspect during an approved game session. Build and benchmark blockers remain; network/authentication blocker resolved.

## 2026-10-18 — manual GitHub Windows build prepared

User explicitly authorized pushing the experimental branch. Adapted existing .github/workflows/push_master.yml: manual dispatch on experiment/physx-diagnostics selects a Windows-only server job; existing full builds, GPU tests and release publication are skipped on that branch. Generates control and feature-enabled diagnostic servers, runs portable tests, and uploads README/provenance/SHA256 artifact for 14 days. No new workflow or default-branch change required. Branch push does not trigger this workflow (push trigger remains master only); user will dispatch it through Actions. actionlint 1.7.12 passed with existing custom gpu_small runner label configured locally; shellcheck/pyflakes disabled. git diff --check passed. Actual Windows build and GPU validation still pending.

## Windows CI validation — run 39159028893

GitHub diagnostic branch build at a6379f0000c5abffb731261f8de42cfda7d9150d succeeded. Windows portable tests and both server builds passed; full/GPU jobs skipped as intended. Job duration approximately 3m40s. Downloaded artifact into /private/tmp/zluda-physx-build-39159028893 and verified both executable SHA256 values against bundled manifest. No target installation or game testing. Next: prepare separate experimental tool integration for user approval; no FPS improvement established.

## Integration correction: matching proxy required

Before installation, inspected zluda32/src/ipc.rs::read_git_version: it rejects a server whose embedded Git SHA differs. Previous server-only artifact cannot be paired with original installed 32-bit DLL. No incompatible binary installed. Workflow corrected to build/package zluda32 for i686-pc-windows-msvc from the same commit as both servers; checksum manifest includes DLL. Preserve existing handshake, protocol and original 64-bit ZLUDA library. User authorized preparing/installing separate experimental tool; no game launch or Steam setting change authorized by this action.

## Integration payload ready, device unreachable

Run 39160259995 succeeded: portable Windows tests, control/diagnostic x64 servers, matching x86 proxy all built. Downloaded to /private/tmp/zluda-physx-build-39160259995; all three manifest SHA256 hashes, PE machine types and embedded 866951a53be80c5a8d4f605b9ec72db8f6839817 revision verified. DEPLOYMENT.md records safe staging/rollback and existing-prefix DLL loading caveat. SSH 192.168.10.67 timed out twice; requested device connectivity/IP. Installation NOT performed. No target files, Steam settings or game prefix changed. Resume with read-only settings inspection then authorized separate tool installation when reachable.

## Separate experimental tool installed

User confirmed Legion awake; SSH restored. Installed /home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton-Experiment via staged independent copy and atomic rename. Source manifest before/after matched across all regular files and symlink targets; original tool unchanged. COMGR hash retained and experimental ldd resolves dependencies. Installed diagnostic server and matching proxy from run 39160259995; checked embedded SHA and file hashes. Updated only experimental default_pfx syswow64/nvcuda.dll, tool identity and user_settings.py; added unique per-invocation CSV path under experimental diagnostics/. Control server retained under diagnostic-control/.

Copied existing prefix nvcuda DLLs and user.reg/system.reg/userdef.reg to experimental pre-experiment-prefix-backup/ with hashes; originals verified unchanged. No game/Steam launched, no Steam setting edited. Inspected Proton code: normal non-NVIDIA path copies default_pfx nvcuda into prefix; actual first-launch delivery still requires hash/runtime verification. Launch options should reference experimental zluda-libs, not original path.

Installer preserved at tools/physx/install_experiment.py (expects verified payload sibling to deployed script); deployment cache /home/deck/.cache/zluda-physx-39160259995. Local syntax verification passed using compile(); initial py_compile could not write macOS bytecode cache and was replaced by cache-free check. Tool is installed, NOT runtime/graphics validated. Next user step: restart Steam when convenient, select ZludaProton-Experiment for Batman and change LD_LIBRARY_PATH to its zluda-libs. User to signal before/after game launch so loaded DLL/server and CSV can be checked read-only. No benchmark/performance claim.

## First live scene: police tape and electrical particles

User reports approximately 28 FPS versus earlier approximately 21 FPS in police-tape scene, with electrical spark/particle effects also visible (present in earlier run but previously unreported). User attributes these to PhysX; exact effect ownership not independently verified. Not a matched A/B or measured FPS trace; no optimization conclusion.

Confirmed running experimental server (Linux PID 4308), matching prefix DLL SHA256 0901df0c0a0c1a3bd995f93b681abc20071b82295fbfa44a8cd83fa6ad949c82, and experimental hipwine/HIP/HSA/COMGR paths in /proc/4308/maps. GPU/KFD handles present; this alone does not validate individual effects.

Diagnostic failure found: CSV only 150-byte headers, tagged Windows PID 416 (0x1a0). Proton log shows TWO server starts, Windows PIDs 0x1a0 and 0x594. Both inherit same per-Proton-invocation CSV path; create_new lets first server create file and prevents second from recording. Active server has no CSV fd. Existing portable tests did not cover launcher+game inheritance. Fix: append server PID to configured CSV base path, retain create_new/no overwrite. Three portable tests now pass including two-process-path collision protection and preservation of existing data. This run provides no CUDA timing breakdown. Need rebuilt matching proxy/server and user-approved restart/update before repeating. No current game interruption or runtime replacement.

Read-only live sample (PID 4308, 10.000927 s): server CPU 78.4927% of one core from process utime+stime ticks; main-thread context switches 8,327.23/s from /proc/PID/status (NOT all-thread aggregate); RSS 226,936 -> 226,824 kB; aggregate GPU busy 70% at both endpoints (NOT interval average). No KFD counter or frame-time samples collected. Scene held constant was requested, not independently verified. These data remain compatible with historical overhead but do not establish cause or regression/improvement. Corrected build triggered: https://github.com/cocciasecca/ZLUDA-32-bit-PhysX/actions/runs/39161675790 at e8541ca. Do not replace running binaries; require user-approved stop/update/restart for next capture.

## Corrected diagnostics installed after user closed game

User explicitly authorized update after closing Batman; no game/server processes found. Run 39161675790 succeeded. Downloaded payload verified SHA256, PE architectures and embedded e8541ca7621562abd09d5f54ef83526701baf6dc revision. Updated experimental server, matching proxy, experimental default_pfx proxy and control payload only. Previous files saved under ZludaProton-Experiment/update-backup-39161675790; update-manifest-39161675790.json records new hashes. Original tool checked against complete original manifest; COMGR unchanged. Existing prefix nvcuda hashes unchanged during update. Proton source confirms nvcuda copy is in per-launch tracked-files block, outside prefix-info-change condition; verify actual proxy delivery on next launch. No game launch or Steam settings change by agent. Existing CSV retained; new outputs append .pid-N.csv. Initial automatic approval review timed out before execution; allowed retry succeeded. Runtime CSV behavior still pending next user launch.

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

## 2026-10-19 — resumed after session interruption

Verified run 39163206056 succeeded at fa5757a2357726eda5aee9bb93063b28893fb6f3. Downloaded artifact /private/tmp/zluda-physx-build-39163206056; all three binary checksums, PE architectures and embedded revisions passed. Legion reachable; pgrep found neither Batman nor ZLUDA server. Continued previously authorized experimental installation while game closed. Updated only experimental diagnostic/control servers and matching proxy/default_pfx proxy; saved prior files in update-backup-39163206056. Source original tool fully verified against saved manifest, COMGR unchanged, existing prefix DLL hashes unchanged during update. Update manifest is in experimental tool directory. No Steam/game launch or settings change. New bind/unbind classification awaits a fresh user scene run. Next: user launches existing ZludaProton-Experiment with unchanged options, returns to same police-tape scene; verify new prefix DLL hash db825c4c0c6d4e3d428d5ea6d6b3bb1cd12f547561080a7d49ffe574e6b8802e and new texture_address/* CSV labels, then capture 30 seconds. No optimization or FPS claim yet.

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

## 2026-10-19 — opt-in texture reuse prototype implemented

User approved prototype. Added texture_reuse.rs pure state machine and server adapter/dispatch barriers, gated by feature AND runtime environment. Preserves explicit cuCtxSynchronize at deferred unbind and reused bind; drains before other API requests, invalidates on lifetime/config changes, bounds tracked handles to 64. No direct HIP struct access or removal of ordering barriers. Separate cleanup timings prevent moving cost outside diagnostics unnoticed. See TEXTURE_REUSE.md for limits, semantics and A/B plan. 11 new portable tests pass, including 4096-operation model comparison; collector tests pass. Workflow updated to test/build additional reuse variant with explicit runtime enablement metadata. No Legion runtime modified or game restarted. Pending Windows build, then controlled target validation; performance remains unknown.
