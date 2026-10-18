# Experimental tool deployment plan

Status: user authorized preparation/installation of a separate tool. No target installation yet. SSH timed out at 192.168.10.67 during preparation; reconnect before any writes.

## Payload

Build run 39160259995 at 866951a53be80c5a8d4f605b9ec72db8f6839817 adds matching 32/nvcuda.dll to control/ and diagnostic/ servers. Verify its outcome and all manifest hashes before staging. Earlier server-only artifact is NOT suitable with the installed proxy: zluda32 checks exact embedded Git revision at startup. Keep that safety check intact.

## Installation operations when reachable

1. Read source user_settings.py, compatibilitytool.vdf, toolmanifest.vdf and Proton prefix-update code. Inspect symlinks to ensure destination edits cannot affect original files. Verify no existing destination, disk space, and no running game/server before any deployment.
2. Source: /home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton. Destination: same parent /ZludaProton-Experiment. Stage to a fresh unique sibling directory and copy with independent files (reflinks acceptable, hardlinks prohibited). Never overwrite/delete an existing destination.
3. Transfer verified artifact to staging. Replace only staging zluda/zluda64_server.exe and zluda/32/nvcuda.dll; preserve 64-bit nvcuda.dll, HIP/hipwine/ROCm and original COMGR. Update staging default_pfx 32-bit nvcuda source to matching proxy if present. Keep control server separately for comparison.
4. Set unique compatibility-tool identity. Preserve user_settings options while resolving ZLUDA64_PATH to final experimental path, ensuring experimental library paths, and using a fresh per-launch Wine-visible CSV path in a dedicated diagnostics directory (outside game/prefix). File must not already exist because create_new prevents overwrites.
5. Record original and experimental checksums, exact changed files, dependency checks and rollback instructions. Verify original source hashes unchanged. Publish staging directory by rename only after validating contents.

## Separate launch boundary

Do not start Steam/game, change launch options or write to existing game prefix as part of installation. Existing prefix contains original 32-bit nvcuda.dll: installing a matching DLL in the TOOL alone does not prove that the game will load it. Inspect Proton update behavior and plan backup/restoration before the first approved launch; verify prefix DLL hash and running server path afterward. If explicit prefix replacement is required, explain and obtain approval first. A launched process is not visual PhysX validation.

Rollback before any launch: select the original tool if selection was changed, retaining the original launch options. Once a launch has updated prefix files, restore only the affected backed-up configuration/DLL files and verify hashes; never reset/delete the prefix or saves. The experimental directory can remain inert; do not blindly remove paths.

## Separate experimental tool installed

User confirmed Legion awake; SSH restored. Installed /home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton-Experiment via staged independent copy and atomic rename. Source manifest before/after matched across all regular files and symlink targets; original tool unchanged. COMGR hash retained and experimental ldd resolves dependencies. Installed diagnostic server and matching proxy from run 39160259995; checked embedded SHA and file hashes. Updated only experimental default_pfx syswow64/nvcuda.dll, tool identity and user_settings.py; added unique per-invocation CSV path under experimental diagnostics/. Control server retained under diagnostic-control/.

Copied existing prefix nvcuda DLLs and user.reg/system.reg/userdef.reg to experimental pre-experiment-prefix-backup/ with hashes; originals verified unchanged. No game/Steam launched, no Steam setting edited. Inspected Proton code: normal non-NVIDIA path copies default_pfx nvcuda into prefix; actual first-launch delivery still requires hash/runtime verification. Launch options should reference experimental zluda-libs, not original path.

Installer preserved at tools/physx/install_experiment.py (expects verified payload sibling to deployed script); deployment cache /home/deck/.cache/zluda-physx-39160259995. Local syntax verification passed using compile(); initial py_compile could not write macOS bytecode cache and was replaced by cache-free check. Tool is installed, NOT runtime/graphics validated. Next user step: restart Steam when convenient, select ZludaProton-Experiment for Batman and change LD_LIBRARY_PATH to its zluda-libs. User to signal before/after game launch so loaded DLL/server and CSV can be checked read-only. No benchmark/performance claim.
