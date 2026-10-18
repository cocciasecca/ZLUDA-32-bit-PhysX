# Codex implementation assignment

## Scope

Act as a performance investigator and systems engineer. Work on a **separate checkout** of ZLUDA and relevant Proton-ZLUDA integration. Do not modify the currently working Steam tool in place.

Repositories:
- `https://github.com/vosen/ZLUDA`
- `https://github.com/UndeadBulwark/Proton-ZLUDA`

## Phase 1 — Reproducibility and code reconnaissance

1. Discover source tree architecture, especially 32-bit CUDA bridge to 64-bit server, HIP allocation, memory registration, module management and sync.
2. Identify exact release/tag/revision matching the running ZLUDA binaries if possible. Check upstream changes and pre-existing performance fixes first.
3. Prepare a minimal self-contained test that mimics likely allocation/unmap churn without Batman, if feasible.
4. Capture build requirements and licensing obligations. Avoid redistribution of NVIDIA/game-owned binary assets.

## Phase 2 — Non-invasive instrumentation

Add opt-in counters and timestamps, compiled behind a feature flag or enabled by an env var. Minimize steady-state overhead when tracing is disabled. Collect:
- CUDA API calls by name, count and latency
- HIP API calls by name, count and latency
- memory allocation/free/registration/unregistration by bytes and pointer/handle lifecycle
- sync/stream/queue waits, return codes, time per event
- IPC bridge request count and latency, where applicable

Prefer aggregated ring buffers or bounded logging; avoid logging every frame to terminal in normal mode. Do not assume `hipFree` is responsible without evidence.

## Phase 3 — Causality and baseline

Automate repeatable 10–30-second profiling runs in the same game area. Track frame time + kernel unmap events + userspace events. Run clean A/B comparisons including PhysX Off and original ZLUDA binary.

Proceed to code modifications only when a specific ZLUDA call path is found to trigger the bottleneck.

## Phase 4 — Minimal patch

Potential targets (only if validated): correct object lifetime issue, pool reusable allocations, amortize memory registration, batch safe operations, reduce unnecessary sync. Preserve CUDA/PhysX semantics. Do not skip free or unmap blindly.

Add unit/integration tests for correctness and rollback. Prefer smallest possible change set and a distinct opt-in switch for experimental behavior.

## Phase 5 — Verification and delivery

- Produce patch or commits, build instructions, and source attribution.
- Produce a separate Steam compatibility tool name (e.g. `ZludaProton-Experiment`) using a documented, supported integration workflow.
- Preserve current working `ZludaProton` untouched.
- Provide A/B median FPS, 1% lows, frame-time plots/numbers, unmap count/rate and latency, and smoke/fog correctness assessment.
- State whether the bottleneck appears to be ZLUDA, HIP/HSA or AMDGPU.
- If upstream patching is ineffective, provide a concise issue report with reproducer and evidence rather than inventing a fix.

## Autonomy and permissions

The device may be accessible via `ssh deck@steamdeck`. Start read-only. Ask approval before writes, installation, game launches/terminations, changing Steam settings, configuring kernel tracing or running privileged commands. Prefer working on a Mac/Linux development machine with the Legion Go only as test hardware.
