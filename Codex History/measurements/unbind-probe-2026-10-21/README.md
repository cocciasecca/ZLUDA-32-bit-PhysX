# Pre-unbind synchronization capture — 2026-10-21

Verified runtime19714c9; probe ON. User reports26–27FPS. 30 second read-only capture, five fully contained windows totaling25.0011633 seconds. All46473 unbind requests have matching pre-sync and post-sync measurements; zero reported errors.23235 bound unbinds and23238 already-unbound requests.

Pre-sync1.0963609 seconds; subsequent native unbind15.3811354 seconds, ~93.35% of the sum of the two phases. Total server service19.818234 seconds EXCLUDES nested probe rows. Bound-unbind parent16.0742793 seconds; already-unbound parent0.4259502 seconds. Phase timers aggregate both bound/already-unbound; do not attribute their average exclusively to resource destruction.

Interpretation: explicit context synchronization does not remove the dominant observed unbind cost. Evidence against the cost being mostly outstanding prior GPU work in this trial. It does not prove pure CPU deallocation: HIP-internal work or synchronization within unbind can remain. This is a scheduling perturbation, and a matched probe-OFF capture is still required for causal performance comparison. User FPS is not controlled A/B. Do not sum nested rows into service time.

Next source investigation: inspect ZLUDA texture lifetime/API usage for avoidable resource churn while leaving HIP untouched. No optimization or further deployment performed from this capture.
