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
