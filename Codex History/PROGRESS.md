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
