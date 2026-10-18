# Relevant architecture at 86ec0a9

`Batman/PhysX/cudart32 -> zluda32 (32/nvcuda.dll) -> Win32 shared memory + events -> zluda64_server.exe -> 64-bit nvcuda.dll (zluda) -> HIP ABI -> hipwine -> Linux HIP/HSA -> AMDGPU/KFD`.

- `zluda32/src/lib.rs`: GlobalState mutex serializes proxy calls; local context/handle bookkeeping. cuMemHostAlloc uses libc::calloc, ignores flags; cuMemFreeHost uses libc::free. It does not expose a HIP-pinned allocation in this path. Do not confuse this with 64-bit memory.rs host allocation behavior.
- `zluda32/src/ipc.rs`: three request encodings (fixed, framed input, framed output); SignalObjectAndWait couples response notification and request wait.
- `zluda_server_common/src/lib.rs`: opcodes, rkyv encoding, shared-memory resizing/mapping. These are Windows CPU mappings, not evidence of KFD GPU mappings.
- `zluda64_server/src/main.rs`: one State and request loop. Allocator lazily calls cuMemAlloc_v2 for 512 MiB, suballocates 256-byte units, translates 32-bit offsets to 64-bit addresses. Free returns ranges; module globals share arena.
- Server DtoH async allocates a fresh Vec, invokes cuMemcpyDtoHAsync_v2, then cuStreamSynchronize before returning serialized bytes. Preserve this synchronization: output must be complete before serialization/drop. HtoD async passes the serialized input buffer into CUDA; any reuse optimization must first establish HIP completion/lifetime guarantees.
- `zluda/src/impl/memory.rs`: direct HIP allocation/free/copy functions; `stream.rs`, `event.rs`, `context.rs`: HIP stream/event/device synchronization; `module.rs`: HIP module unload. Resource management below these calls can trigger KFD even without explicit hipHostUnregister calls in ZLUDA.
- Existing PhysX-specific infrastructure includes the limited 32-bit proxy, pointer translation, module-global/kernel-argument handling and NVAPI spoofing; docs/src/physx32.md documents known reinitialization hazards. No new pooling warranted without runtime evidence.
