# ZLUDA PhysX performance investigation — Legion Go

This folder is a self-contained handoff for Codex. Start with [CURRENT.md](CURRENT.md) for the installed build, prepared package and next action; consult historical notes for evidence and context.

## Goal

Investigate severe frame-time regressions in **Batman: Arkham Asylum GOTY** on a **Lenovo Legion Go (Ryzen Z1 Extreme / Radeon 780M, gfx1103)** with Proton-ZLUDA and NVIDIA GPU PhysX effects enabled. Identify whether ZLUDA, ROCm/HIP, HSA or the AMDGPU/KFD driver is responsible; implement and benchmark a safe patch **only if the evidence supports it**.

## Read order

1. [01_ENVIRONMENT.md](01_ENVIRONMENT.md) — hardware, paths, components
2. [02_FINDINGS.md](02_FINDINGS.md) — confirmed experiments and profiling numbers
3. [03_HYPOTHESES.md](03_HYPOTHESES.md) — questions, confidence and pitfalls
4. [04_CODEX_TASKS.md](04_CODEX_TASKS.md) — investigation and implementation plan
5. [05_TEST_PLAN.md](05_TEST_PLAN.md) — A/B comparison and profiling
6. [06_SAFETY_AND_ROLLBACK.md](06_SAFETY_AND_ROLLBACK.md) — modification boundaries

## Repositories

- https://github.com/vosen/ZLUDA
- https://github.com/UndeadBulwark/Proton-ZLUDA

Do not assume the current HEAD corresponds to the installed binary. Record exact Git revisions, patches, build flags and provenance before comparing binaries.

## Important

The suspected GPU-memory-management bottleneck is **not yet proven causal**. A kernel perf stack and high unmap counts establish a strong lead, not a validated fix. Avoid treating context switches, process CPU usage, and total GPU busy as precise measures of time spent running CUDA kernels.
