# Combined pool feasibility capture

Local changes, no push/build/deployment by agent. Profile pool-capture: diagnostic server, range cache ON, texture reuse OFF, split-unbind OFF, bounded global read probe ON, combined collector ON.

Start gate: after user reaches scene, create/touch diagnostics/pool-capture.start on Legion. Timestamp must be newer than server startup. Collector polls once per second until armed. Old marker cannot trigger next process. Capture lasts60seconds of subsequent requests; no restart needed to arm. At completion writes last3000 events once and stops collecting. If game stops issuing requests finalization waits for next request. File must contain completion marker before treating it as complete.

Capture file: diagnostic base + .pool.pid-N.txt. Existing service CSV remains unchanged. All observation/file writes occur outside API service timers, on receiving the next request; wall-clock overhead still exists. Previous result status is read before dispatching new request. Returned bind offsets are recorded and nonzero offsets excluded from LRU eligibility. Errors and unknown/lifetime operations clear simulated state. Allocation-generation counter increments on resets; conservative global invalidation prevents reuse across allocation/context lifetime. Pointer/size/format/channels/flags/generation key, intentionally shared across texref handles.

Five LRU capacities8/16/32/64/128. Cumulative snapshots every5seconds: hits,misses,evictions; coverage binds/unbinds/launches/unknown/error/reset counts and cumulative distinct keys capped4096 (truncation flag). Configuration table capped512; names capped512 and512bytes each, hex encoded. Ring records successful/failed events, setter arguments, offsets, launch handles and reset boundaries. Named texrefs/functions recorded at lookup. Metadata/name registration is bounded but occurs within those lookup request handlers. Other bridge operations retain opcode in ring. Lifetime reset events currently do not retain original opcode. Symbol snapshot can be ambiguous if a handle is reused; do not interpret it across reset boundaries without checking.

This model estimates descriptor-key locality, NOT safe pool hits: it does not model pinning, in-flight GPU references or native create failures. High hits justify further engineering, not automatic pooling. Filter/address mode APIs remain existing proxy no-ops and are not visible to server: pool cannot be declared generally correct on this basis. No GPU writes performed. Exact per-object memory costs and dynamic allocation generations beyond conservative global resets are not measured. Same-source service timings and byte offsets are available in this run; no synthetic pre-sync.

Six local tests passed, including2000 requests checked against independent recent-history LRU oracle, capacity limits, unknown/error/lifetime exclusions, nonzero offsets and one-time bounded ring flush. Windows integrated tests added to workflow; user build still required.

User command with Batman closed:
python3 tools/physx/push_build.py --yes --install --profile pool-capture
Then user returns to scene; assistant arms data capture after user says ci sono. No need to push/build/install again to arm it. Do not promise both observed workload regimes in one stationary capture; report which actually occurs.
