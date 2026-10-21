# Opt-in texture object pool prototype

Local source only. User performs push/build/install. Profile texture-pool uses diagnostic server and ZLUDA_PHYSX_TEXTURE_POOL=1; split-unbind, simulated capture and global probe OFF. Range cache ON. Existing service timers retained with count-only pool/hit, pool/miss, pool/fallback rows (exclude these counts from API totals).

Capacity64 owned objects. Keys include native allocation address, byte length and validated88-byte host descriptor with object field zeroed. Limited to nonnormalized point-sampled linear uint32/float textures, channels1/2/4, supported address modes and default mipmap/sRGB settings. Pointer must be256-byte aligned and native offset validated zero. Other cases use native path. HIP, native nvcuda and drivers are unchanged; new calls use existing CUDA texture-object exports from native ZLUDA.

On a miss, execute a native bind, validate device global size88 and nonzero host/device object at offset72, then create an independent texture object through cuTexObjectCreate. Native unbind destroys ONLY the native-owned temporary. The pool publishes a descriptor copy to the module global; pooled handles NEVER go into host textureReference. Native host remains null. Every new host/global association must validate before using a cached object. Descriptor creation failures preserve native binding; publication failure is an error and retains ownership until restoration.

On hit, synchronize then publish descriptor. Logical unbind synchronizes, writes null object to the device copy and releases logical association without destroying cached object. Flags/format setters still run natively; unchanged effective descriptor needs no publication; changed active descriptor is rebound or falls back. Current proxy filter/address setters remain historical no-ops, so this is NOT a general CUDA texture implementation.

No LRU eviction in v1. Reaching64 distinct keys drains and disables pooling for the process. Drain synchronizes, restores all active references to native-owned bindings, then destroys pool objects. Memory/module/context/lifetime and unknown operations drain before dispatch. A previous request error also triggers drain. Transfers/kernels execute normally. Synchronization is preserved; no async destruction or CPU worker introduced.

Tests:10 standalone tests with -D warnings. Native-control-flow implementation runs against a portable mock validating that legacy setters never own pool objects and double destruction fails. Covers100 bind/unbind cycles, shared objects, changed/unchanged setter, capacity overflow, unsupported config, create failure, sync failure and failed device publication. Mocks do NOT prove ABI/GPU coherence. Integrated Windows tests and actual production build added to CI; not run by agent. Device correctness/FPS unverified.

First run: visually check PhysX against baseline; report glitches/crash immediately. At ci sono read pool counters and API errors before interpreting FPS. Fallback may be frequent because this deliberately restricted v1 favors correctness. Native fallback does not hide synchronization/publication failures. Backups remain install-script managed.

Build/install (Batman closed):
python3 tools/physx/push_build.py --yes --install --profile texture-pool

Use profile diagnostic on the SAME successful run to disable pool for comparison without rebuild. No claim of guaranteed speedup.
