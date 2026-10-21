# Next path: explicit pre-unbind synchronization

Archived experiments: tag `archive/physx-texture-deferral-2026-10-21`. Texture reuse, shadow model, PTX classifier and their build variant are removed from the active branch. General diagnostics, texture request categories, range cache and module-unload metadata fix remain.

Optional new probe: diagnostic server plus ZLUDA_PHYSX_SPLIT_UNBIND=1 and a fresh ZLUDA_PHYSX_DIAGNOSTICS CSV path. Normal installer uses control server and disables this probe. No HIP source or binary changes.

On null texture address requests, measure cuCtxSynchronize then native unbind separately. Preserve errors: failed synchronization blocks unbind; native errors propagate. Three local tests cover ordering, disabled behavior and errors. Existing aggregation tests also pass. Integrated Windows compilation still required.

Labels probe/unbind_pre_sync and probe/unbind_after_sync are NESTED inside texture_address service time. Never sum them into total request service time. Includes already-unbound requests; interpret alongside bound/already-unbound request counts. Compare with a separate probe-OFF capture on identical warmed-up workload. Explicit synchronization perturbs scheduling and includes unrelated context work; a shorter unbind after sync is evidence of a waiting contribution, not exact attribution. Residual time is not automatically pure deallocation cost. No GPU run or performance improvement established.
