# Pending: cache repeated texture metadata setters in the 32-bit proxy

User steered back to the previously proposed redundant-IPC idea. Added a bounded
64-handle flags/format cache, successful responses only, guarded by GlobalState's
mutex and ZLUDA_PHYSX_TEXREF_CACHE=1. Uses the allocation-cache invalidation barriers:
errors, context changes, lifetimes and unknown operations clear both caches.
Known launches/transfers/address binds preserve metadata; these do not change
flags/format in the supported bridge. Binds and synchronization are never skipped.
Native setter errors still propagate on misses/changes; failed values are not cached.
This is specific to the host-side metadata setters of the supported HIP bridge.
New package marker windows-texref-cache-v1 prevents accidental old modified installs.
Reference stays pristine 86ec0a9. Diagnostic-work gating remains in this build too.
User builds/deploys with the usual script. No FPS gain claimed, no remote mutation.

## Previous work

# Next prototype: skip unused diagnostic work

A/B gameplay captures both reported 23 FPS; no reproducible mean-FPS benefit is
established. Server CPU was roughly 84% modified / 81% reference (one core=100%).
Current change gates shadow texture histories, request labels and diagnostic PTX
analysis on an active recorder, including after output failure. These paths ran
previously even with the diagnostic variable unset. Cleanup/reuse barriers and
GPU calls remain unchanged. Installer now disables logging for ordinary gameplay
as well as comparisons. Profiling remains available via ZLUDA_PHYSX_DIAGNOSTICS.
No performance gain claimed; user runs build/install. No remote mutation or PR.

## Previous work

# Pending comparison before upstream contribution

User deferred master integration/PR until an original-vs-modified comparison.
Only fetched remotes: origin/master already equals upstream/master at d623f19;
local master remains 86ec0a9. No push, merge, PR or remote install performed.
Workflow now prepares a pinned pristine 86ec0a9 Windows bridge alongside ours.
Use one run with --comparison reference / modified; both disable runtime logging.
Keep the 64-bit ZLUDA runtime, HIP, Proton and game configuration fixed. This
isolates bridge changes, not differences between complete ZLUDA releases.
See tools/physx/PUSH_BUILD.md for A/B/A commands. Default behavior is unchanged.

# Current default: Windows bridge with texture deferral

The default is the behavior of e8dac2e: Windows server, original owned transfers,
texture destruction/transfer deferral and range cache. Cleanup removes retired
native backends and fixed-run deployment tools without changing these settings.
Use tools/physx/README.md and PUSH_BUILD.md for maintained documentation.

Development remains on experiment/physx-diagnostics. User runs push/build/install.
No PR, master update or remote installation is authorized for this cleanup step.
Historical experiments and detailed measurements remain in Git and this directory.
