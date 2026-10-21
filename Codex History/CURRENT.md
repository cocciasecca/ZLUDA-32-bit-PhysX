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
