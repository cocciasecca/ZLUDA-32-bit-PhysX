# Next steps

1. Obtain reachable Legion Go IP (asked asynchronously). Read-only SSH: verify uname, paths, binary hashes/version strings, loaded HIP/HSA/hipwine/COMGR and running process identity. No game launch required for static checks.
2. Validate diagnostic prototype on Windows builder; Rust/Cargo not preinstalled on this Mac. Portable Rust unit test status in PROGRESS. Preserve complete source on experiment/physx-diagnostics branch; worktree under /private/tmp may be cleaned by OS.
3. Complete HIP/API/lifetime instrumentation after inspecting target runtime tracing support. Obtain approval for a concrete bounded userspace/kernel attribution capture; do not modify tracing state yet. Need exact KFD-unmap user call stack, including possible worker threads, before choosing a patch.
4. Prepare separate tool payload and manifest, preserve COMGR, and present actual files/commands before requesting installation approval. No executable experiment is ready for deployment yet.
5. Capture baseline and overhead control, identify supported minimal optimization, then A/B frame times and user-assisted smoke/fog/fire correctness validation. Do not remove required DtoH synchronization or add speculative pooling.

Current blockers: target hostname unresolved; Windows/HIP build environment unavailable; unmap trigger and causality unknown. No runtime performance conclusion. Original historical notes remain unchanged.
