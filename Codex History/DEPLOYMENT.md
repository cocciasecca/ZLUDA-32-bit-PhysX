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
