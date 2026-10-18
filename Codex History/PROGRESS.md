# Progress

## 2026-10-18 — source reconnaissance and diagnostic prototype

Read all seven original handoff Markdown files before changes; originals preserved.
Initial checkout: clean tracked files on master, HEAD 86ec0a93cc83ba5b52e645892523e339dfa884f7; Codex History initially untracked. No AGENTS.md found under the repository/parent search.
Created separate worktree `/private/tmp/zluda-physx-diagnostics`, branch `experiment/physx-diagnostics` (source changes only there). No push, device writes, game launches, prefix changes, or tracing configuration.

Confirmed source already suballocates from a lazy 512 MiB GPU arena. Bridge cuMemFree only releases a range; it does not call HIP. The direct per-CUDA-free -> hipFree hypothesis does not fit this revision. Pageable transfer handling and HIP-internal staging/registration now deserve priority, but neither is proven to cause KFD unmaps.

Verified public upstream master and v7-preview.15 both resolve to this HEAD. Exact Proton-ZLUDA release convert.sh pins v7-preview.15 / zluda-windows-86ec0a9.zip. Installed binary identity remains unchecked.

Implemented first opt-in server request service-time aggregation (feature + environment switch), excluding the wait for the next request. This is partial Phase A, not a performance optimization or complete HIP/lifetime instrumentation.

SSH read-only connectivity failed: steamdeck DNS resolution, including outside sandbox. Requested device IP. Mac has no installed Rust/Cargo or Windows build environment; installed Rust 1.90.0 compiler and standard library only under /private/tmp/zluda-rust-toolchain for portable diagnostic tests (no system installation). Full Windows component build, CUDA tests, runtime attribution and benchmarks remain pending.

Validation completed: standalone diagnostics.rs compiled with Rust 1.90.0 on aarch64 macOS; both unit tests passed (success/error/latency aggregation and independent request rows/window reset). git diff --check passed. These tests do not validate Windows integration or CUDA semantics. No installed target or runtime was modified. Source and notebook are preserved on the experimental branch; original checkout remains on master with notebook files only.

## 2026-10-18 — SSH access established

User installed dedicated public key; SSH to deck@192.168.10.67 now works with explicit identity ~/.ssh/zluda_legion_ed25519 (private key remains on Mac, never in repository). Read-only checks confirmed kernel from historical notes, tool version `1791146054 cachyos-11.0-20261005-slr`, and embedded server commit `86ec0a93cc83ba5b52e645892523e339dfa884f7`. No matching Batman/server process found in the process-name check; no game launched. COMGR ldd resolves dependencies. perf present; command lookup found no bpftrace, rocprofv3, rustc, cargo, clang or MinGW compiler. Tool occupies 2.0 GiB; /home has approximately 73 GiB available. No target writes by agent.

SHA256 under ZludaProton:
- zluda/zluda64_server.exe: 40303768cb583431cc0b7221709e6c2cfa89490cfede801e58b7351524401fdf
- zluda/nvcuda.dll: 3d9b46c4b43bfefebf41271a2cd68aee32ef6d870fcadf4a431fb103cecbe1a9
- zluda/32/nvcuda.dll: 072bac0639c776d8c5b8b6ef3a3c0e8aaebcd96e0a19b874057cfead131f6867
- zluda-libs/libamd_comgr.so.3: e765f0913bc94de361b8e471e22c5085b2e6d967510cb80c2084b031e3f4781c

These are installed-file fingerprints, not a comparison against release archive hashes. Loaded runtime paths remain to inspect during an approved game session. Build and benchmark blockers remain; network/authentication blocker resolved.

## 2026-10-18 — manual GitHub Windows build prepared

User explicitly authorized pushing the experimental branch. Adapted existing .github/workflows/push_master.yml: manual dispatch on experiment/physx-diagnostics selects a Windows-only server job; existing full builds, GPU tests and release publication are skipped on that branch. Generates control and feature-enabled diagnostic servers, runs portable tests, and uploads README/provenance/SHA256 artifact for 14 days. No new workflow or default-branch change required. Branch push does not trigger this workflow (push trigger remains master only); user will dispatch it through Actions. actionlint 1.7.12 passed with existing custom gpu_small runner label configured locally; shellcheck/pyflakes disabled. git diff --check passed. Actual Windows build and GPU validation still pending.

## Windows CI validation — run 39159028893

GitHub diagnostic branch build at a6379f0000c5abffb731261f8de42cfda7d9150d succeeded. Windows portable tests and both server builds passed; full/GPU jobs skipped as intended. Job duration approximately 3m40s. Downloaded artifact into /private/tmp/zluda-physx-build-39159028893 and verified both executable SHA256 values against bundled manifest. No target installation or game testing. Next: prepare separate experimental tool integration for user approval; no FPS improvement established.

## Integration correction: matching proxy required

Before installation, inspected zluda32/src/ipc.rs::read_git_version: it rejects a server whose embedded Git SHA differs. Previous server-only artifact cannot be paired with original installed 32-bit DLL. No incompatible binary installed. Workflow corrected to build/package zluda32 for i686-pc-windows-msvc from the same commit as both servers; checksum manifest includes DLL. Preserve existing handshake, protocol and original 64-bit ZLUDA library. User authorized preparing/installing separate experimental tool; no game launch or Steam setting change authorized by this action.
