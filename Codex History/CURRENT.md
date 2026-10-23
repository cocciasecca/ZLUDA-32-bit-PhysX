# Current work: complete native Linux transport candidate

The last user-validated game configuration remains Windows server + texture deferral
(29–30 FPS reported, later 26 FPS). No native installation has been performed.

The implementation now ports the full existing server dispatcher, not the earlier
smoke-only API subset. See tools/physx/native/README.md for architecture, build,
preflight and rollback. Native install is gated on same-revision successful Windows
and Linux artifacts plus an actual x86 DLL/PTX32 GPU test in an isolated test prefix.
Do not call the package validated until those gates have passed. User requests work
continue until DLLs and project are ready for Legion installation.

## Historical notes (superseded by the status above)

# Current configuration: restored texture deferral

The active server sources and Cargo.lock are restored exactly from `3e9ea16`,
the revision associated with the user's 29 FPS report. This restores deferred
texture destruction, transfer retention, kernel classification and its original
diagnostics. Range cache remains enabled. It does not establish a 29 FPS guarantee.

Removed from active source/build: texture-object pool, pool capture, texture-global
probe and split-unbind probe. Their history and measurements remain in Git/docs.
The user reported 19 FPS for the first pool and 22 FPS for its revised unbind path.

The single push/build/install script now builds `physx-texture-reuse`, installs
that server with its matching proxy, and enables REUSE=1, REUSE_TRANSFERS=1,
RANGE_CACHE=1 and the original diagnostics. Old pool/probe flags are reset to 0.
Package configuration is checked before upload to prevent installing a stale pool
artifact. Batman/server closed checks, backup and hash checks remain automatic.
45 portable tests passed; kernel-analysis tests and Windows compilation run in CI.
No build, push or installation was performed by the agent.

## Historical notes (superseded)

# Current state — 2026-10-21

## Decision: close the texture-deferral/cache investigation

This is the furthest justified outcome of this specific investigation, not a claim that ZLUDA has reached its absolute performance limit. Keep the bounded allocation-range cache as a candidate baseline; disable texture deferral and diagnostic logging by default. No controlled FPS improvement has been established for the range cache.

The latest validated capture followed all 76,962 texture-address requests and 17,810 kernel launches. No modeled reuse across candidate kernels was observed; 17,100 candidate binding episodes ended at noncandidate kernels. Simply delaying those cleanups does not eliminate their cost in this capture. See [PATH_OUTCOME.md](PATH_OUTCOME.md) and measurements/shadow-coverage-2026-10-21/.

## Deployment state

Last verified installed on Legion: commit `3e9ea162d124d3e7e118465f8f920e007d9d162f`, run `39521873732`, with diagnostics and texture deferral ON. User reported 29 FPS after advancing and returning; not a controlled comparison. Today's repository cleanup does not change the running game.

The next installation through tools/physx/push_build.py --install selects the control server, keeps range cache ON, sets texture deferral OFF and removes the diagnostic environment variable. Backup/manifest have a distinct baseline suffix. Original Proton, HIP/COMGR and saves remain untouched.

## Workflow and retained evidence

Work on branch experiment/physx-diagnostics in /private/tmp/zluda-physx-diagnostics; original IDE checkout stays on master. tools/physx/PUSH_BUILD.md documents the current automation. Historical fixed-run installers are retained for provenance, not current deployment. Experiments remain available behind existing build features; no deletion of measurement evidence or rollback packages.

Next investigation should target a different mechanism and use this simpler baseline. Do not repeat the no-texture-kernel deferral experiment without new evidence.

Active branch cleanup: texture experiments moved to tag archive/physx-texture-deferral-2026-10-21. Three artifacts now: control, diagnostic, proxy. Optional pre-unbind synchronization probe prepared and locally tested; disabled by default, not deployed. See UNBIND_PROBE.md.

Installed unbind probe: run39524047539 commit19714c9cd429bed6ccc49ea1dc9bd58373e9bfda, diagnostic server and matched proxy verified. SPLIT_UNBIND=1, range cache ON, reuse OFF; unique unbind-probe UUID CSV per launch. Backup update-backup-39524047539-unbind-probe. No running game modified; prefix preserved during install. Await same-scene launch and capture. Probe timers are nested: exclude from service totals. Need probe-OFF comparison before causal conclusions.

Probe ON capture19714c9: user26–27FPS;46473 phase pairs reconcile with all unbind calls, zero errors. Pre-sync1.096s, subsequent unbind15.381s in25.001s. Dominant cost persists after explicit sync, not proof of pure deallocation. Probe-OFF comparison still missing. See measurements/unbind-probe-2026-10-21. No new deployment.

User now owns push/build/install. Local-only script update: baseline/diagnostic/unbind-probe profiles and safe repeat switches with distinct backups. Five settings profile transitions tested, fresh log IDs verified. Next: user installs run39524047539 profile diagnostic (probe OFF), no new compilation needed. Existing flags/format setters already avoid refresh when unchanged; no new texture optimization justified yet. No push or remote mutation in this step.

Probe OFF verified19714c9, profile diagnostic, user24–25FPS. Bound unbind659us vs ON692us, ~82% service remains unbind. Workload not equivalent:959 kernel launches/sec vs645, mostly changed bindings OFF vs identical ON. No causal FPS comparison. Evidence measurements/unbind-off-2026-10-21. Leave probe OFF; no remote writes.

Local preparation: bounded native texture global probe, profile texture-global. Two local layout tests pass; Windows integration test added to CI. No push/build/install performed. Await user script and runtime observation. See TEXTURE_GLOBAL_PROBE.md. LRU/sequence recorder not yet implemented; first gate is global feasibility.

Runtime076a3128 confirmed, run39574184476 profiletexture-global. User21FPS. All8 sampled native texture globals return size88 and matching nonzero host/device object field. Feasibility lookup gate passes for sampled symbols, not proof of safe writes/pooling. Sample budget exhausted; no further GPU probe reads for this process. FPS cause unestablished. Next: bounded LRU and sequence observation; no deployment performed. Evidence measurements/texture-global-2026-10-21.

Local combined pool capture ready, six local tests passed; user owns push/build/install. Profile pool-capture. IMPORTANT: when user says ci sono, arm via fresh diagnostics/pool-capture.start (timestamp newer than server startup); wait60sec, require complete marker. Captures LRU8/16/32/64/128, offsets/config/coverage, ring3000 and symbol names. No GPU behavior change, optimistic locality only. See POOL_CAPTURE.md.

Combined capture COMPLETE d8bf7d4: user21FPS;50029 binds,41distinctkeys,zero errors/unknown/nonzero offsets/resets. LRU64/12899.918% optimistic hits;32slots97.587%;8/16zero. Ring18texrefs, flags set whilebound (knownunchanged). Proceed toward opt-in pool prototype with native fallback; do not claim measured speedup or safe ownership yet. See measurements/pool-combined-2026-10-21.

Local prototype texture-pool implemented,64objects, native fallback, lifetime drains, preserved sync, host never owns pooled objects.10 portable ownership/control-flow tests passed; production Windows/GPU unverified. Profile texture-pool; user owns push/build/install. See TEXTURE_POOL_PROTOTYPE.md.

## 2026-10-23: Linux backend feasibility

User accepts restored deferral as the baseline (reported 29–30 FPS, initial particle
stutter remains). No runtime setting changed. Added tools/physx/native smoke harness:
explicit native library, separate process, context/allocation/copies/PTX kernel/two
verified results/cleanup. Four mocked tests are automated locally and in CI; actual
Linux/GPU execution and Proton integration remain outstanding. Python is only a
feasibility harness, not the proposed per-call transport. Recovered missing tracked
files in the existing temporary checkout from HEAD without overwriting present files.

### Native binary bridge (2026-10-23)

Added tools/physx/native/bridge.rs: standalone Rust Linux server loading ZLUDA
through dlopen and Windows-i686-compatible probe client. Versioned binary loopback
RPC, per-run token, bounded messages/allocations, opaque allocation IDs, context
cleanup on disconnect. Supports initialization, allocations, copies, one fixed PTX
kernel, free and quit. run_bridge.py automates startup/authentication/shutdown and
optionally runs the Windows probe through a specified Wine executable. Main CI
builds/tests Linux x86-64 and Windows x86 artifacts; deployment still selects the
established deferral backend. This is NOT the game's nvcuda.dll transport yet:
module/texture/event/stream RPC and integration with existing dispatch remain open.
Local transport tests pass; Linux-specific Rust code type-checked locally via
metadata emission, not linked/run on Linux. Actual Linux/HIP and Windows/Wine GPU
execution remain unverified. No game performance claims and no installation.

### DLL native transport bootstrap

`zluda32/src/native_backend.rs` now intercepts exported CUDA calls when explicitly
selected; routes initialization/context/memory copies/alloc/free to binary Linux
transport, rejects unsupported operations and prevents any Windows server fallback.
Protocol shared in tools/physx/native/wire.rs. Baseline installer sets native mode OFF.
Windows CI builds and exercises the actual DLL via native-dll-probe.exe and a fake
peer. Real Windows build and Linux/HIP test have not been run locally. This is NOT
ready for Batman: enumeration, module/kernel/texture/event/stream support remains.
