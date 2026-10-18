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
