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
