# Benchmarks

No new runtime measurements in this session. Historical evidence below is copied from 02_FINDINGS.md, not remeasured and not a controlled baseline.

| Metric | Historical observation |
|---|---|
| PID-filtered KFD unmaps | 24,168 / approximately 10 s |
| Server CPU samples in delay_halt_mwaitx | 53.91% of 831 samples |
| Server context switches | 66,485 / 10 s |
| Server CPU | 76–83% of one core, instantaneous |
| Aggregate GPU busy | 69–70% |
| Server RSS | approximately 250 MB |
| Scene FPS | approximately 16 in fog corridor, 24 near physics objects |

Baseline and experiment: NOT RUN. Average FPS, 1% lows, frame-time p50/p95/p99, unmap latency, memory trends, repeat variability and visual correctness all pending. No performance improvement claimed.

Use 05_TEST_PLAN.md: fixed scene/camera, power/TDP/resolution/settings; at least three bounded repetitions; separate Off/Normal/High runs and approved restarts (PhysX reinitialization is known problematic). Record binary/library checksums and scene timing, omit warm-up, preserve raw frame-time data. Define 1% low calculation explicitly. Compare diagnostic overhead before using trace runs to interpret normal performance.

## First live scene: police tape and electrical particles

User reports approximately 28 FPS versus earlier approximately 21 FPS in police-tape scene, with electrical spark/particle effects also visible (present in earlier run but previously unreported). User attributes these to PhysX; exact effect ownership not independently verified. Not a matched A/B or measured FPS trace; no optimization conclusion.

Confirmed running experimental server (Linux PID 4308), matching prefix DLL SHA256 0901df0c0a0c1a3bd995f93b681abc20071b82295fbfa44a8cd83fa6ad949c82, and experimental hipwine/HIP/HSA/COMGR paths in /proc/4308/maps. GPU/KFD handles present; this alone does not validate individual effects.

Diagnostic failure found: CSV only 150-byte headers, tagged Windows PID 416 (0x1a0). Proton log shows TWO server starts, Windows PIDs 0x1a0 and 0x594. Both inherit same per-Proton-invocation CSV path; create_new lets first server create file and prevents second from recording. Active server has no CSV fd. Existing portable tests did not cover launcher+game inheritance. Fix: append server PID to configured CSV base path, retain create_new/no overwrite. Three portable tests now pass including two-process-path collision protection and preservation of existing data. This run provides no CUDA timing breakdown. Need rebuilt matching proxy/server and user-approved restart/update before repeating. No current game interruption or runtime replacement.

Read-only live sample (PID 4308, 10.000927 s): server CPU 78.4927% of one core from process utime+stime ticks; main-thread context switches 8,327.23/s from /proc/PID/status (NOT all-thread aggregate); RSS 226,936 -> 226,824 kB; aggregate GPU busy 70% at both endpoints (NOT interval average). No KFD counter or frame-time samples collected. Scene held constant was requested, not independently verified. These data remain compatible with historical overhead but do not establish cause or regression/improvement. Corrected build triggered: https://github.com/cocciasecca/ZLUDA-32-bit-PhysX/actions/runs/39161675790 at e8541ca. Do not replace running binaries; require user-approved stop/update/restart for next capture.

## First successful in-scene request profile — police tape

User confirms identical position/scene/settings, reports approximately 25 FPS (user observation, not captured frame-time mean). Server Linux PID 5156; active CSV Windows PID 592, revision e8541ca. Launcher PID 416 CSV remains header-only; game CSV now grows, confirming per-process filename fix. Capture: 30.029 s of read-only /proc/sysfs samples; five COMPLETE 5 s CSV windows strictly after the capture's initial last-completed-window boundary, totaling 25.0018556 s. Excluded partly elapsed first window. Files: measurements/police-tape-2026-10-18/{selected-windows.csv,summary.json,samples.json}. Full raw CSV/maps retained locally at /private/tmp/zluda-scene-capture/police-tape-30s.json.

- cuTexRefSetAddress_v2: 64,151 calls; 2,565.85/s; 18.4036043 s service time; 286.88 us mean; 1.7586 ms maximum; **94.42% of total measured request service time**.
- cuMemcpyDtoHAsync_v2 (includes internal stream synchronization): 0.6888136 s, 3.53%.
- cuMemcpyHtoDAsync_v2: 0.1838876 s, 0.94%.
- cuLaunchKernel: 18,083 calls; 0.1360875 s host service time, NOT GPU kernel duration.
- All recorded request statuses zero in selected windows. No cuMemAlloc/cuMemFree requests in those windows.
- Total measured service time 19.4912619 s over 25.0018556 s; excludes IPC wait and diagnostic writes, includes decoding/serialization and nested calls.
- Over 30.029 s sample: server CPU 81.12% of one core; main-thread context switches 7,005.94/s (not all-thread aggregate); RSS 231,420 -> 231,320 kB. GPU busy arithmetic mean of 31 approximately 1 Hz aggregate samples 69.55% (not dedicated compute utilization).

Source trace: zluda64_server cu_tex_ref_set_address_v2 -> cuTexRefSetAddress_v2 -> zluda/src/impl/tex.rs::ref_set_address_v2 -> hipUnbindTexture for null pointer, hipTexRefSetAddress otherwise. Current counter groups bind AND unbind; cannot distinguish their rates or time. Flags/format setters already refresh only when changed. Need separate null/non-null, binding identity and bytes/redundancy counters or HIP tracing before optimization. At roughly 3 address calls per format/flags call, a bind/unbind pattern is plausible but NOT established.

This supersedes pageable transfer overhead as the first investigation target for this scene. Similarity between 2,566 address calls/s and historic approximately 2,417 KFD unmaps/s is a lead, not a causal match: captures are not simultaneous. No KFD measurement or userspace unmap stacks taken in this run. No frame-time improvement demonstrated. Next: attribute bind/unbind costs and KFD stacks; consider caching only with texture state, null-unbind semantics, kernel ordering and resource lifetime preserved. Do not skip synchronization or blindly cache texture addresses.

## 2026-10-19 — texture lifecycle profile, police-tape scene

User returned to scene after update. Verified prefix x86 nvcuda SHA256 db825c4c0c6d4e3d428d5ea6d6b3bb1cd12f547561080a7d49ffe574e6b8802e, active server Linux PID 8396 and growing CSV Windows PID 1400 at fa5757a. Captured 30.0293 s; five fully contained aggregate windows total 25.0021308 s, excluding initial partly elapsed window. Raw capture /private/tmp/zluda-texture-capture/police-tape-30s.json; selected windows/summary/samples preserved in measurements/police-tape-textures-2026-10-19/.

- unbind_bound: 20,480 successful calls, 819.13/s, 16.0131859 s, mean 781.89 us, **82.20% of total measured request service time**.
- bind_same_after_unbind: 20,472 successful calls, 818.81/s, 2.4217113 s, mean 118.29 us, **12.43%**.
- unbind_already_unbound: 20,472 calls, 0.0093581 s, mean 0.457 us, only 0.048%. Optimizing redundant empty unbind alone would not address hotspot.
- DtoH async incl stream wait: 0.642118 s, 3.30%; HtoD async 0.1897017 s, 0.97%.
- No bind_changed_address_size or direct rebind_same_address_size rows in selected windows. All observed non-null binds reuse last successful address+size after unbind. This does NOT prove same format/flags, contents, memory generation, or a safe lifetime shortcut. 8-call difference can straddle window boundaries; do not infer exact per-texture sequence count.
- Total service 19.4808581 s. Server CPU 80.55% of one core; main-thread context switches 6,651.30/s; RSS 231,980 -> 231,944 kB; 31 approximately 1Hz aggregate GPU-busy samples average 69.52%. GPU kernel launch count differs from earlier capture (14,224 here vs 18,083 earlier); cannot assume workload equivalence solely from same scene. No measured frame-time trace or new FPS claim.

Evidence now narrows costly CUDA operation to null-address unbind -> HIP texture destruction/free, rather than repeated no-op unbind, plain device free or bulk copies. Source HIP 7.2.4 destruction releases image/sampler/descriptor, candidate for KFD churn. Runtime KFD call attribution and proportionality are STILL unmeasured. Historical ~2,417 unmaps/s is from a different session and must not be treated as matched with current ~819 texture destroys/s.

Next engineering step: investigate bounded texture resource reuse preserving logical unbound state, CUDA synchronization, flags/format/offsets, module and allocation lifetimes, error behavior and kernel visibility. Do not simply omit hipUnbindTexture; it currently includes implicit memory-free synchronization in HIP. Must retain required ordering and validate with focused lifecycle tests and same-scene frame-time A/B. No optimization installed. User can leave scene; no further live capture needed for this data point.
