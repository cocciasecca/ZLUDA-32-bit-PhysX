# ZLUDA-only investigation after reuse trial

User excludes HIP modifications. Original texture reuse remains OFF; user reports24FPS OFF versus21ON, not a frame-time trace.

Source reviewed: zluda32/src/lib.rs GlobalState serializes calls under one mutex; ipc.rs remote_call_* signals/waits synchronously once per request. Server diagnostic timers exclude IPC wait and client lock/serialization overhead. This does not establish lock contention or explain all single-core usage.

First bounded candidate: cache successful cuMemGetAddressRange_v2 responses in 32-bit proxy, avoiding repeat IPC for known live allocation ranges. Prior baseline819 queries/s,0.194us mean server service; this tiny server cost is NOT total round-trip cost. Server range is allocator-rounded256 bytes; cache actual successful response, not requested allocation size. Cache under same mutex as calls; bound capacity, handle interior pointers/end boundaries/overflow, clear conservatively on allocation/free/module load/unload/context lifecycle and errors. Module globals share arena. Misses always use original request. Must audit all lifetime paths and error/ordering semantics before implementation. No GPU allocation pooling or skipped sync. Measure end-to-end per-op timings and lock wait before deciding expected value.

Secondary candidate: repeated texture flags/format settings currently always cross IPC despite backend refresh already checking state changes. Baseline~819/s each, but equality frequency not measured. Suppression would require state/lifetime/error semantics validation; do not equate call rate with redundant call count. Empty-unbind~819/s similarly tiny server time but skipping can alter error/ordering; lower priority.

Kernel launch constructs temporary parameter/global Vecs each call (main.rs cu_launch_kernel); baseline all host launch handling only0.62% service time, so preallocation low expected value. DtoH bridge explicitly waits before serializing host results: cannot simply remove wait. General GPU allocations absent in steady-state sample and arena already exists. No reason to replace mutex with parallel GPU calls without establishing contention and preserving contexts/order.

Conclusion: next measurement should time client end-to-end IPC (separate mutex wait from roundtrip) with bounded aggregate logging; subsequent smallest optimization is allocation-range metadata cache if repeat hits/IPC cost justify it. No source/runtime changed by this investigation; no large FPS claim. Dominant texture destruction still not resolved within these changes.
