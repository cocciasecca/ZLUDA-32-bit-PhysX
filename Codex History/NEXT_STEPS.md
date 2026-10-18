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
