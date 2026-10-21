# Local shadow-model verification — 2026-10-21

Local changes only; Legion remains on c147f63. No build, push, deployment or restart requested in this verification.

Fixed known unchanged flags/format preservation. Changed or unknown configuration invalidates binding metadata; kernel cleanup drops only pending bindings, preserving both active bindings and known configuration. Errors/lifetime/unknown boundaries reset all state. The model never changes real GPU barriers.

Added observed-address, observed-unbind, observed-kernel, candidate-kernel and pending-at-candidate counters. Acceptance for a future runtime capture: observed-address count must equal successful address classifications; observed-kernel count must equal dispatched launches in error-free windows; inspect candidate and pending coverage before interpreting absence of hits. Zero candidate events without coverage are inconclusive. Pending-at-candidate counts occurrences, not unique episodes or saved time.

Validation: 20 standalone tests passed with rustc -D warnings (13 existing reuse tests, 7 shadow tests). Generated comparison covers 256 deterministic sequences of 256 transitions each over three handles, checking pending and reusable bindings against the existing TextureReuse state machine with a CPU-only backend. Full cycles cover configuration order, repeated unbind, texture kernel, candidate kernels, identical rebind, changed flags/format/channels and error/lifetime reset. Tests use synthetic sequences: aggregate game logs cannot reconstruct exact event order.

Mutation checks: reintroducing either loss of active bindings at texture kernels or invalidation on identical flags causes the full-cycle regression test to fail. This confirms the test catches both previously missed defects.

Limits: no Windows integrated compilation in this step, no new live capture, no GPU safety proof and no demonstrated FPS gain. Future runtime acceptance must check coverage before drawing reuse conclusions. Capacity resets and unknown operations remain conservative exclusions.
