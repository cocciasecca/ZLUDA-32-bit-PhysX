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
