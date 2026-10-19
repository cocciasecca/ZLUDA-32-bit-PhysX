# Conservative texture reuse prototype

Status: experimental source, opt-in; no GPU/game validation or speedup claim. User approved implementing prototype. Existing Legion diagnostic runtime remains unchanged.

## Scope and mechanism

Feature `physx-texture-reuse` implies diagnostics; runtime additionally requires exact `ZLUDA_PHYSX_TEXTURE_REUSE=1`. Applies only to serialized 32-bit server, not general 64-bit CUDA. No HIP struct layout access or GPU buffer cache. A native successful texture address call records (handle, pointer, size, byte offset). A later null-address unbind executes cuCtxSynchronize before marking logical unbound and retaining the physical binding. Same-address/size rebind cancels pending destruction only after another cuCtxSynchronize. Direct repeated binds without intervening unbind still call native CUDA.

Only address, flags, format, and local arena address-range lookup are allowed while native destruction is deferred. Flags/format always invoke native setters; changed/unknown values force native pending unbind first and invalidate cached binding/offset. All other opcodes (including launches, memory copies, synchronization, unsupported requests) drain pending unbinds before dispatch. Thus a launch cannot observe a logically unbound retained texture. Memory free, module unload, context detach/init also invalidate bookkeeping before handles/address ranges can be reused. Existing bridge supports no texture-state getter that bypasses this barrier.

64 tracked handles maximum: capacity overflow drains pending objects before forgetting metadata. Errors invalidate reuse state and propagate; failed barrier aborts the following request. Resource-destruction errors can now occur at deferred cleanup boundary, an important behavior to investigate in compatibility validation. Explicit context barriers are stronger than some native empty-unbind operations and may reduce benefit. Process teardown relies on existing process/runtime cleanup, as baseline does.

This preserves ordering conservatively; it is NOT a proof of complete CUDA compatibility. CPU-side logical state may differ from physical HIP state only inside the explicitly whitelisted bridge sequence. Future API additions must preserve default drain behavior. No skipping native getters/validation outside the cached successful address path.

## Measurement

CSV header reports feature and runtime enablement. Existing texture categories time the optimized path. Deferred cleanup before other requests is separately timed as `texture_reuse/drain_before_request`; include this row in total service time to avoid hiding moved work. Classification overhead is outside request timer as in preceding diagnostic build. Compare reuse binary OFF/ON with same binary/proxy/tool and fixed scene; then compare against original and diagnostic-only builds. Record actual frame times, errors, memory growth and visual correctness. Source-level HIP frees are a mechanism candidate; runtime KFD attribution remains unmeasured.

## Validation and delivery

11 portable tests cover barriers, returned offsets, changes while bound/unbound, failed sync/native operations, per-handle separation, overflow cleanup, and a 4096-operation deterministic reference-model comparison at observable boundaries. Existing logger/classifier tests also retained. Windows workflow builds control/, diagnostic/, reuse/ server variants plus matching 32/nvcuda.dll and manifest; all share commit SHA. GPU execution/PhysX smoke-fog-fire correctness, memory pressure, sustained stability and performance remain untested.

No automatic deployment or enablement: prepare reviewed package, verify build, then schedule a closed-game update. Restore diagnostic server and paired proxy from backup to roll back; never overwrite original Proton-ZLUDA or reset game prefix. Keep experimental activation explicit and reversible.
