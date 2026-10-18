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
