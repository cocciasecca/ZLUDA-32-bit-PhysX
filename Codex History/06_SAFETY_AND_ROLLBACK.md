# Safety, preservation and rollback

## Preserve these assets

- `~/.local/share/Steam/compatibilitytools.d/ZludaProton` — original custom tool, with COMGR library added
- Steam prefix `~/.local/share/Steam/steamapps/compatdata/35140/` — includes PhysX runtime and game settings/saves; **do not delete or reset**
- Steam game installation `~/.local/share/Steam/steamapps/common/Batman Arkham Asylum GOTY/`
- Any user save files or Steam Cloud data
- Existing launch options, especially `LD_LIBRARY_PATH` to `zluda-libs`

## Experimental build rules

- Work in separate repository checkout and build directory.
- Install test binaries as a **separately named** Steam compatibility tool, not over the working version.
- Keep a manifest of added/changed files and checksums.
- Ensure installer and uninstall scripts are reversible.
- Back up affected configuration/prefix files before changing them.
- Do not update host glibc, host ROCm, SteamOS, GPU drivers or the kernel.
- Do not install NVIDIA graphics drivers.
- Do not use `HSA_OVERRIDE_GFX_VERSION` or memory-lifetime shortcuts without specific justification and rollback.
- Avoid blanket `rm -rf`, particularly on game paths and compatibility prefixes.
- Remove kernel probes and temporary trace state reliably even after interrupted tests.

## Approval boundaries

Ask the user before `sudo` writes, patch installations, package installs, replacing DLLs, modifying launch options, terminating/restarting Steam or Batman, or profiling requiring elevated privileges. Read-only discovery can proceed autonomously under the user's SSH permission.

## Distinguish results

Report clearly what was observed, what was inferred, what failed, and what remains untested. The GPU-memory-unmap issue is currently a **high-priority hypothesis**, not a proven bug in ZLUDA and not a confirmed patch opportunity.
