# Counterfactual texture reuse measurement

The reuse server now maintains a separate bounded (64-handle) model. Actual barriers, synchronization and HIP operations are unchanged. A successful bind followed by unbind remains pending in the model across kernels classified NoTextureInstructions and permitted transfers/synchronization. A later successful identical pointer/size bind records `shadow/same_binding_after_candidate`; a different binding records `shadow/changed_binding_after_candidate`.

Texture-using/unknown kernels, lifetime/other boundaries, configuration setters and previous request errors terminate candidates. Setters terminate even when values are unchanged: zero hits therefore do NOT exclude all possible reuse. Capacity overflow resets the model and records capacity_reset. Remaining pending candidates at capture/shutdown are unresolved, not hits. Handles/pointers reused after lifetime operations cannot produce hits because the model is cleared.

CSV shadow rows have zero duration. Do not add their counts to API request totals or interpret them as measured time saved. They count texture-binding episodes, whereas cleanup timers count request boundaries. Hits are counterfactual opportunities, not proof of descriptor safety or guaranteed performance. Kernel execution failure causes a reset on the following request.

Three standalone tests exercise repeated unbinds, same/changed binding, intervening barriers/configuration, absence of candidate kernels and bounded capacity. CI additionally compiles the integrated Windows server and matching proxy. Runtime measurement requires this new build and a game restart; the currently installed revision cannot supply these counters.
