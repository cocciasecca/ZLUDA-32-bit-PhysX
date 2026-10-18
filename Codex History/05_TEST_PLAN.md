# Benchmark and profiling plan

## Conditions to compare

| Run | Proton | PhysX | ZLUDA | Purpose |
|---|---|---|---|---|
| A | Known working standard Proton | Off | No | Rendering baseline |
| B | Current original Proton-ZLUDA | Normal | Yes | Original PhysX lower preset |
| C | Current original Proton-ZLUDA | High | Yes | Original PhysX full preset |
| D | Experimental Proton-ZLUDA | Same as C | Yes, patched | Controlled patch evaluation |

Keep resolution, graphics, power/TDP, clocks, scene, game version and external processes as constant as possible. Restart between runs if needed to control cache and persistent-state effects. Run multiple repetitions and record variability.

## Metrics

- Average FPS, 1% low FPS, frame-time p50/p95/p99 or equivalent distributions
- CPU utilization of game and server, thread/host CPU distributions
- GPU busy **aggregate** and GPU frequency/power if available
- RSS and VRAM trends
- KFD unmap calls/sec (systemwide and `common_pid` filtered)
- Call durations where feasible, and correlations to individual frame times
- HIP/CUDA allocations, frees, memory registration, sync, kernel launches
- visual presence of smoke, fog, fire, debris; crashes or graphical corruption

Use the same view or repeatable path in the long corridor with fog (previously ~16 FPS) and the police-tape scene (~24 FPS). A saved checkpoint and repeatable camera orientation are useful. Do not assume a test scene is identical across independent runs unless verified.

## Existing tracefs kprobe reference (not an automatic execution script)

Known kernel symbol: `kfd_ioctl_unmap_memory_from_gpu` in module `amdgpu`. A dynamically created tracefs probe named `kprobes:zluda_unmap` was used in prior tests. The event was removed afterward using `-:zluda_unmap` in `kprobe_events`.

**Do not blindly recreate/remove events:** inspect whether an event exists and avoid interfering with unrelated tracing sessions. Ensure cleanup on exit and use proper privileges. Filtering via `common_pid` may fail to attribute driver work done asynchronously by another kernel worker.

## Success criteria

An improvement is only meaningful if: frame times improve materially and reproducibly in the same scene; PhysX effects remain visible and correct; there are no new crashes, memory growth, or correctness regressions; any drop in unmap counts/latency is causally connected to the change.
