# Observations, logs and profiling evidence

## In-game performance

- Without GPU PhysX or on standard Proton, the user reported approximately 60 FPS in at least some scenes.
- With PhysX enabled, certain fog and physics-object scenes fall to approximately 10–16 FPS; another scene with police tape/physics objects ran around 24 FPS.
- An elevator scene subsequently reached about 48 FPS, so slowdown is not universally permanent.
- Switching PhysX from High to Normal did **not** cure the long-corridor fog slowdown (~16 FPS).
- Preserving visually convincing smoke/fog/fire is important; physical interaction is expendable. The immediate project instead investigates original GPU PhysX performance, not visual replacements.

## Instantaneous monitor sample near a ~24 FPS scene

Observed approximately:
- Batman CPU: 57–75% of one core
- ZLUDA server CPU: 76–83% of one core (instantaneous sampling)
- GPU busy (aggregate rendering + compute): 69–70%
- Batman RSS: 701–706 MB
- ZLUDA RSS: approximately 250 MB

These data do not isolate GPU PhysX compute utilization. Stable memory samples do not rule out short-lived allocation churn.

## Kernel perf (10-second server-process recording)

```text
Samples: 831, cpu-clock
53.91% [kernel.kallsyms] delay_halt_mwaitx
  delay_halt -> amdgpu_fence_wait_polling
  -> mes_v11_0_submit_pkt_and_poll_completion
  -> ... -> gmc_v11_0_flush_gpu_tlb_pasid
  -> amdgpu_vm_flush_compute_tlb
  -> kfd_ioctl_unmap_memory_from_gpu
  -> ioctl
6.50% [kernel.kallsyms] amdgpu_device_rreg.part.0
  -> ... -> kfd_ioctl_unmap_memory_from_gpu
3.73% [kernel.kallsyms] __schedule
2.53% libhsa-runtime64.so.1 [unsymbolized]
2.41% libc.so.6 __sched_yield
```

These percentages are **CPU profiling sample proportions**, not whole-game frame-time percentages. Stack unwinding or missing symbols may affect attribution. KFD unmap and GPU TLB flushes merit investigation.

## 10-second perf stat on ZLUDA server

```text
66,485 context-switches
881 cpu-migrations
0 page-faults
```

Note: many context switches can result from synchronization; their cause is not yet known.

## Kprobe measurements

Kernel symbol was visible:

```text
ffffffffc1827360 t kfd_ioctl_unmap_memory_from_gpu [amdgpu]
```

The built-in `perf probe -m amdgpu` route failed due to missing/unusable module symbol data. A tracefs dynamic kprobe worked.

- Systemwide unfiltered measurement: **24,309** calls / ~10 seconds.
- PID-filtered (`common_pid` for `zluda64_server.exe`) separate measurement: **24,168** calls / ~10 seconds.

These were **not simultaneous** measurements, so an exact fraction cannot be calculated, although the similar totals support the server as a major source. The filtered count means probes were triggered while that process was the current task; confirmation of precise causation should include more tracing and controlled A/B testing.

## What is NOT established

- Whether these calls are intrinsically unnecessary or optimizable.
- Whether the calls are caused by `hipFree`, transient memory registration/unregistration, mapped graphics memory, or HSA cleanup.
- How long each call takes and what share of actual frame time it causes.
- Whether PhysX CUDA kernels execute correctly and at expected rates.
- Whether a patch to ZLUDA alone can address the issue.
