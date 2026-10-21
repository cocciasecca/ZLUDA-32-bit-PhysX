# Texture global feasibility probe

Local changes only. User owns push/build/install. No pool or device-memory writes implemented.

Profile texture-global selects diagnostic server, range cache ON, split-unbind OFF, texture-global probe ON. At most eight distinct successful non-null texture-bind handles are sampled per process. Native cuModuleGetGlobal_v2 (not bridge arena lookup) retrieves the symbol; only exactly88 bytes are accepted. A guarded ReadProcessMemory reads the host texref. Native cuMemcpyDtoH_v2 reads the device global. Compare nonzero eight-byte object field at offset72 against the current repo LLVM emitter/HIP bindings layout. A match is a candidate ABI observation, not proof that writing it is supported or safe. Size/access/API errors are logged, never propagated as bind failure. No HIP library replacement.

CSV path: diagnostics base + .texture-global.pid-N.csv. Symbol names are UTF-8 bytes encoded in hex. Writes occur after the existing request timer finishes and before returning to client. The extra device copy can synchronize, perturb timing and surface asynchronous errors: do NOT benchmark this run as a performance optimization. Sampling stops after eight handles, including failed attempts; >512-byte names are ignored. Module unload/reused handles may cause later sampling to be skipped, never stale pointer dereference.

Two local tests reject null/mismatched objects and incorrect sizes. Windows CI now compiles/tests integrated diagnostic probe. GPU behavior unverified until the user's run.

Next gate: inspect lookup success, exact size and object match. A negative result prevents this direct-global pool design; it does not prove every repo-only texture strategy impossible. Only after a positive result consider bounded sequence/LRU instrumentation; neither is implemented in this step.

Run with Batman closed:
python3 tools/physx/push_build.py --yes --install --profile texture-global
