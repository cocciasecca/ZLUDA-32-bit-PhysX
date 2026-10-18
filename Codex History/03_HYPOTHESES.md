# Hypotheses and falsifiable checks

## H1 — Excessive GPU memory registration/unregistration (priority: high)

ZLUDA/HIP could trigger many `kfd_ioctl_unmap_memory_from_gpu` calls while processing PhysX. This could cause costly GPU TLB flushes and fence waits. **Supporting evidence:** profiled stack, ~2.4k kernel unmap calls/sec in ZLUDA context. **Missing:** map to higher-level HIP/CUDA events and wall-time attribution.

Tests: count HIP allocations/frees, host-memory mapping/unmapping, module loads and synchronization; include call-site, bytes, GPU handle and timestamp. Correlate with KFD unmap trace and frame-time peaks.

## H2 — Synchronization and submission overhead (priority: high)

The AMDGPU driver may be waiting on GPU commands or flushing translation state frequently, regardless of whether unmap calls are redundant.

Tests: measure operation latency and overlap; compare CPU-side and GPU-side timing, timeline annotations, kernel waits, and queue activity. Do not remove required barriers.

## H3 — Version/build-specific HIP/ROCm interactions (priority: medium)

The installed HIP 7.2 runtime and COMGR 7.2.3 could have an interaction affecting the Ryzen Z1 Extreme / `gfx1103` under SteamOS. The COMGR fix made HIP detect the GPU, but does not prove optimal support.

Tests: compare known compatible bundled library sets, ensure exact version provenance, inspect debug symbols and runtime logs, avoid system-wide glibc upgrades.

## H4 — PhysX-specific workloads or fallback (priority: medium)

Individual PhysX simulations may use CPU fallback, excessive CUDA dispatches or large sparse transfers. Normal and High both exhibit slowdown.

Tests: validate CUDA device enumeration and actual kernel launches; instrument server request mix; correlate with exact scenes and effects.

## H5 — General rendering bottleneck unrelated to PhysX (priority: lower but must exclude)

The GPU busy counter combines graphics and compute; unrelated rendering or shader compilation could explain portions of the slowdown.

Tests: compare the **same scene and same settings** with PhysX Off, Normal and High; monitor frame-time percentiles, not only FPS.

## Interpretation cautions

- High CPU utilization of a server process does not automatically mean PhysX is running on the CPU.
- A high call count is a correlation, not a proof of avoidable work.
- A kprobe `common_pid` filter identifies which task ran the kernel function, not necessarily which user-space function logically owns every resource.
- A safe patch must preserve CUDA ordering, object lifetime, stream semantics, resource residency and error propagation.
