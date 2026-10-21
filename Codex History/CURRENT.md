# Current state — 2026-10-20

Read this before older chronological notes. Historical entries describe the state at their date and may be superseded.

## Installed on Legion (last verified)

- ZludaProton-Experiment: code revision `c147f632660b6f218259480294ade962b2f3a187`, run `39466980451` (corrected shadow reuse counters).
- Range cache ON; texture reuse and transfer retention ON.
- Latest user observation: 24–25 FPS (earlier prototype: 27–28). Capture found no successful cached texture rebinds; kernel-boundary cleanup still dominated. This is not a controlled frame-time speedup result.
- Original Proton-ZLUDA and HIP/COMGR libraries preserved. User excludes HIP modifications.

## Installed diagnostic package

- Kernel texture-instruction analysis: code revision `43a1290e4324bcc27b1ce81f25a96134dfcdeb93`, successful run `39406852403`.
- Tests and Windows builds passed; four binary checksums, architectures and embedded revisions verified.
- Durable package in the original checkout: `Codex History/packages/kernel-analysis-39406852403/` (payload, installer, verification manifest). Binary package is intentionally not committed to Git.
- Installer: `tools/physx/update_kernel_analysis.py` on the experimental branch.
- This is diagnostic classification, not a new optimization: all pre-kernel cleanups still execute. Absence of texture instructions is not proof that cleanup can be skipped.

## Resume

User authorized and installation completed on 2026-10-20 with game closed. Backup: `update-backup-39406852403`. Launch and capture completed; see latest live result below. Offline installed-PhysX PTX census is in measurements/physx-ptx-census-2026-10-20. Next: quantify potential reuse across candidate kernels before changing barriers. Preserve all rollback backups.

## Workspace layout

Original IDE checkout stays on master. Experimental changes are pushed to `experiment/physx-diagnostics`, with working checkout `/private/tmp/zluda-physx-diagnostics`. The latest branch tip may contain documentation only; use the explicit build revision above for binary matching.

See [KERNEL_TEXTURE_ANALYSIS.md](KERNEL_TEXTURE_ANALYSIS.md) for implementation and validation, [PROGRESS.md](PROGRESS.md) for chronology, and [measurements/](measurements/) for evidence. The numbered original notes and old installers are historical records, not current deployment instructions.

Latest live result (2026-10-20):43a1290 verified, user24–25FPS. ~50.36% measuredserver service in cleanup before kernels classified without texture instructions;~25.93% before texture-using kernels. Diagnostic candidates only; no selective skipping authorized by this evidence alone. Details in KERNEL_TEXTURE_ANALYSIS.md.

Counterfactual reuse instrumentation committed as e8a4166; build run 39465899244 started. Not installed. See TEXTURE_SHADOW.md. Actual GPU cleanup unchanged; awaiting game closure and completed build for measurement.

2026-10-20: shadow build 39465899244 completed successfully and installed with game closed. Verified four binary hashes, PE architectures and embedded revision. Backup update-backup-39465899244. Original tool and prefix DLLs unchanged by installation; matching proxy is staged for next game launch. Await launch in same scene to collect shadow rows. No new runtime measurement yet.

Shadow capture e8a4166: user 25 FPS, no shadow events. Model defect found and fixed locally; do NOT interpret this as no reuse. See measurements/texture-shadow-2026-10-20. Corrected build/restart required.

Corrected shadow build 39466980451 successfully installed with game closed. Backup update-backup-39466980451. Four binary hashes/architectures/revisions verified; original tool, prefix during installation and COMGR preserved. Await user return to same scene; runtime c147f63 and new shadow rows must be verified. Prior e8a4166 zero-event result invalid for estimating reuse.

Corrected shadow runtime c147f63 captured: user26FPS, no errors, still no shadow events. Inconclusive: unconditional configuration invalidation can lose active binding before unbind. Do not claim zero reuse. Evidence: measurements/texture-shadow-corrected-2026-10-20. No further build installed.

2026-10-21 local verification: shadow configuration preservation fixed; coverage counters added. 20 standalone tests pass, including 65,536 generated transitions; both old bugs detected by mutation checks. No push/build/deployment. Installed c147f63 unchanged. See SHADOW_LOCAL_VERIFICATION.md.
