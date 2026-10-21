# Outcome of the PhysX32 cache and texture-deferral path

Reached the practical limit supported by evidence in this path. This is not the maximum achievable ZLUDA performance.

- Keep range metadata caching as an optional, enabled installation baseline: observed cache hits eliminate repeated range queries. FPS gain remains unproven.
- Disable texture-unbind deferral and transfer retention by default: no successful modeled reuse across candidate kernels in the validated capture. Deferral merely moves cleanup for the observed episodes.
- Disable diagnostic logging for normal use. Retain source, tests and measured evidence for future investigations.
- Keep the control server paired with the matching 32-bit proxy. Historical experimental artifacts remain available; no native HIP changes.

Evidence: measurements/shadow-coverage-2026-10-21/{README.md,summary.json,capture.json}. 25.017 seconds in complete windows; 76,962 address calls, 17,810 launches; coverage reconciled, zero reported API errors; 17,100 candidate episodes terminated at noncandidate kernels. Outstanding episodes at window boundaries are not counted as reuse. These numbers describe server activity, not frame time or a controlled FPS speedup.

Cleanup updates only defaults and documentation; neither GPU lifetime/synchronization code nor historical experimental source is removed. The last installed diagnostic session remains unchanged until the next explicitly executed installation with Batman closed.
