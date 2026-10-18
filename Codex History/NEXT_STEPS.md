# Next steps

1. SSH established at deck@192.168.10.67 with ~/.ssh/zluda_legion_ed25519. Static identity/hashes verified (PROGRESS.md). Inspect loaded HIP/HSA/hipwine/COMGR during an approved game session; no matching game/server currently found.
2. Validate diagnostic prototype on Windows builder; Rust/Cargo not preinstalled on this Mac. Portable Rust unit test status in PROGRESS. Preserve complete source on experiment/physx-diagnostics branch; worktree under /private/tmp may be cleaned by OS.
3. Complete HIP/API/lifetime instrumentation after inspecting target runtime tracing support. Obtain approval for a concrete bounded userspace/kernel attribution capture; do not modify tracing state yet. Need exact KFD-unmap user call stack, including possible worker threads, before choosing a patch.
4. Prepare separate tool payload and manifest, preserve COMGR, and present actual files/commands before requesting installation approval. No executable experiment is ready for deployment yet.
5. Capture baseline and overhead control, identify supported minimal optimization, then A/B frame times and user-assisted smoke/fog/fire correctness validation. Do not remove required DtoH synchronization or add speculative pooling.

Current blockers: Windows/HIP build environment unavailable; unmap trigger and causality unknown. No runtime performance conclusion. Original historical notes remain unchanged.

## Immediate build action

After authorized branch push, manually run existing ZLUDA workflow (push_master.yml) with branch experiment/physx-diagnostics. Download physx-servers-<SHA> only after successful build; inspect logs on failure. No workflow dispatched by agent yet. Then prepare separate integration and get device-installation approval before any writes.

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
