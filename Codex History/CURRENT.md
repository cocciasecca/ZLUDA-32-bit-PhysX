# Current state — 2026-10-20

Read this before older chronological notes. Historical entries describe the state at their date and may be superseded.

## Installed on Legion (last verified)

- ZludaProton-Experiment: code revision `d9a00719f8231c36e42a730aadb989d3b58060ab`, run `39291351130`.
- Range cache ON; texture reuse and transfer retention ON.
- User observed 27–28 FPS. Capture found no successful cached texture rebinds; kernel-boundary cleanup still dominated. This is not a controlled frame-time speedup result.
- Original Proton-ZLUDA and HIP/COMGR libraries preserved. User excludes HIP modifications.

## Ready, NOT installed

- Kernel texture-instruction analysis: code revision `43a1290e4324bcc27b1ce81f25a96134dfcdeb93`, successful run `39406852403`.
- Tests and Windows builds passed; four binary checksums, architectures and embedded revisions verified.
- Durable package in the original checkout: `Codex History/packages/kernel-analysis-39406852403/` (payload, installer, verification manifest). Binary package is intentionally not committed to Git.
- Installer: `tools/physx/update_kernel_analysis.py` on the experimental branch.
- This is diagnostic classification, not a new optimization: all pre-kernel cleanups still execute. Absence of texture instructions is not proof that cleanup can be skipped.

## Resume

The user says Legion is off network and will write OK when ready. Do not connect or install before then. After OK, check game closed, install the paired server/proxy with backup, and capture the same scene. Compare cleanup time for kernels with texture instructions, without them, and unknown cases. Preserve all rollback backups.

## Workspace layout

Original IDE checkout stays on master. Experimental changes are pushed to `experiment/physx-diagnostics`, with working checkout `/private/tmp/zluda-physx-diagnostics`. The latest branch tip may contain documentation only; use the explicit build revision above for binary matching.

See [KERNEL_TEXTURE_ANALYSIS.md](KERNEL_TEXTURE_ANALYSIS.md) for implementation and validation, [PROGRESS.md](PROGRESS.md) for chronology, and [measurements/](measurements/) for evidence. The numbered original notes and old installers are historical records, not current deployment instructions.
