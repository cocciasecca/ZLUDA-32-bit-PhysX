# Relevant architecture at 86ec0a9

`Batman/PhysX/cudart32 -> zluda32 (32/nvcuda.dll) -> Win32 shared memory + events -> zluda64_server.exe -> 64-bit nvcuda.dll (zluda) -> HIP ABI -> hipwine -> Linux HIP/HSA -> AMDGPU/KFD`.

- `zluda32/src/lib.rs`: GlobalState mutex serializes proxy calls; local context/handle bookkeeping. cuMemHostAlloc uses libc::calloc, ignores flags; cuMemFreeHost uses libc::free. It does not expose a HIP-pinned allocation in this path. Do not confuse this with 64-bit memory.rs host allocation behavior.
- `zluda32/src/ipc.rs`: three request encodings (fixed, framed input, framed output); SignalObjectAndWait couples response notification and request wait.
- `zluda_server_common/src/lib.rs`: opcodes, rkyv encoding, shared-memory resizing/mapping. These are Windows CPU mappings, not evidence of KFD GPU mappings.
- `zluda64_server/src/main.rs`: one State and request loop. Allocator lazily calls cuMemAlloc_v2 for 512 MiB, suballocates 256-byte units, translates 32-bit offsets to 64-bit addresses. Free returns ranges; module globals share arena.
- Server DtoH async allocates a fresh Vec, invokes cuMemcpyDtoHAsync_v2, then cuStreamSynchronize before returning serialized bytes. Preserve this synchronization: output must be complete before serialization/drop. HtoD async passes the serialized input buffer into CUDA; any reuse optimization must first establish HIP completion/lifetime guarantees.
- `zluda/src/impl/memory.rs`: direct HIP allocation/free/copy functions; `stream.rs`, `event.rs`, `context.rs`: HIP stream/event/device synchronization; `module.rs`: HIP module unload. Resource management below these calls can trigger KFD even without explicit hipHostUnregister calls in ZLUDA.
- Existing PhysX-specific infrastructure includes the limited 32-bit proxy, pointer translation, module-global/kernel-argument handling and NVAPI spoofing; docs/src/physx32.md documents known reinitialization hazards. No new pooling warranted without runtime evidence.

## Texture lifecycle follow-up

User authorized autonomous technical direction; no authorization inferred for interrupting live game or privileged tracing. Target strings in libamdhip64.so.7 identify build source paths /usr/src/debug/hip-runtime/rocm-systems-rocm-7.2.4/... . This refines earlier generic HIP 7.2 notes; COMGR remains separately supplied 7.2.3. Build paths are provenance evidence, not proof that binary exactly matches unpatched upstream source.

Official ROCm tag rocm-7.2.4 source inspected:
- https://github.com/ROCm/rocm-systems/blob/rocm-7.2.4/projects/clr/hipamd/src/hip_texture.cpp : hipTexRefSetAddress destroys prior texture object, creates another, then copies reference to device. ihipCreateTextureObject allocates descriptor via ihipMalloc(sizeof(__hip_texture), CL_MEM_SVM_FINE_GRAIN_BUFFER). ihipDestroyTextureObject releases image/sampler and calls ihipFree. hipUnbindTexture also destroys texture object and nulls handle.
- https://github.com/ROCm/rocm-systems/blob/rocm-7.2.4/projects/clr/hipamd/src/hip_memory.cpp : ihipFree calls SyncAllStreams and memory release/SvmBuffer::free if allocation is not handled by memory pools.

This is a concrete source-level candidate for synchronization/allocation churn hidden under texture APIs, consistent with measured hotspot. Runtime KFD attribution remains unconfirmed. Do not assume public hipFree uprobes catch internal ihipFree calls.

Implemented bounded, feature-gated TextureHistory classification: bind unknown, rebind same address+size, bind same after unbind, changed address+size, unbind bound/already-unbound/unknown. Tracks only successful requests, resets at memory free/module unload and at 4096-entry bound. Equality explicitly excludes format/flags and resource/content identity: it is diagnostic, not a safe-cache predicate. Labels replace generic cuTexRefSetAddress request name in same aggregate CSV; no nested/double-counted timing. Classification occurs before timer; successful history update included in handler service time. Default build omits tracking. All CUDA calls, synchronization and error results retained.

Three new portable tests pass (bind/unbind sequence/size changes, failed-call nonmutation/invalidation, capacity bound), plus three existing collector tests pass. Workflow tests new module on Windows too. actionlint passed with existing custom runner configuration, diff check passed, changed Rust files formatted. No device runtime replaced; game still active at last check. Next Windows build then matched proxy/server update when user closes game; targeted repeat capture will establish which texture operation dominates before proposing reuse.

## 2026-10-19 — opt-in texture reuse prototype implemented

User approved prototype. Added texture_reuse.rs pure state machine and server adapter/dispatch barriers, gated by feature AND runtime environment. Preserves explicit cuCtxSynchronize at deferred unbind and reused bind; drains before other API requests, invalidates on lifetime/config changes, bounds tracked handles to 64. No direct HIP struct access or removal of ordering barriers. Separate cleanup timings prevent moving cost outside diagnostics unnoticed. See TEXTURE_REUSE.md for limits, semantics and A/B plan. 11 new portable tests pass, including 4096-operation model comparison; collector tests pass. Workflow updated to test/build additional reuse variant with explicit runtime enablement metadata. No Legion runtime modified or game restarted. Pending Windows build, then controlled target validation; performance remains unknown.
