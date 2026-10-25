# Windows PhysX bridge workflow

The supported configuration uses the 32-bit CUDA proxy and Windows 64-bit server.
Texture destruction deferral, retention across transfers, and the allocation-range
cache are enabled by the installer. Host transfers use the original owned buffers.
HIP itself is unchanged. Diagnostic counters remain available but are disabled by the
current installer; kernel classification never permits skipping a cleanup barrier.

Use [PUSH_BUILD.md](PUSH_BUILD.md) for the single push/build/install entry point.
`check_local.py` runs installer rollback tests and portable Rust state-machine tests.
Windows compilation and PTX classification tests run in GitHub Actions.

## Retained code

- `zluda32/src/range_cache.rs`: bounded successful range replies, invalidated on
  lifetime changes, unknown requests and errors by the proxy.
- `zluda64_server/src/texture_reuse.rs`: bounded pending unbinds, explicit
  synchronization and mandatory drains before kernel/lifetime/unknown requests.
- Server diagnostics and shadow counters: observation only, not GPU scheduling.
- `deploy_legion.py` and `legion_install.py`: artifact verification, closed-game
  checks, backups, transactional replacement and native-to-Windows migration.

## Retired experiments

Linux TCP/shared-memory backends, texture pools and borrowed-transfer prototypes
are not part of the supported runtime. The native experiment is recoverable from
commit `c17a028`; the last default before this cleanup is `e8dac2e`. Historical
fixed-run installers were removed to prevent accidentally installing an old build.
Existing migration support is retained for devices that still have a native wrapper.

`Codex History/` contains historical measurements and session notes, not current
installation instructions. Reported FPS are observations, not controlled evidence
of the improvement from an individual optimization.

## Texture metadata IPC prototype

`ZLUDA_PHYSX_TEXREF_CACHE=1` enables a 64-handle proxy cache of successful format
and flag setters. An exact repeat returns the previous successful result without
an IPC round trip. Changed values and misses always call the native setter.
Errors, context/lifetime changes and unknown opcodes clear the cache. Address
binds and synchronization still execute; no writes are deferred to a later launch.
This relies on the supported single-device Windows bridge and its host-only HIP
metadata setters, not a general guarantee for arbitrary CUDA implementations.
The installer enables it for modified/default and disables it for the reference.
No measured FPS improvement is established. The diagnostic-work gate is also in
this pending build, so a whole-build comparison does not isolate the two changes.

## Reusable launch preparation

The Windows server retains the capacity of its kernel-parameter pointer array
and 32-bit global-offset array between requests. Values are rebuilt for every
launch, and both logical arrays are cleared after the native call, including
error returns. All global storage growth finishes before pointers into it are
formed. The native argument order and synchronous parameter-consumption lifetime
match the previous temporary-vector implementation. No GPU waits are removed.

This is a small host-side allocation optimization. Historical measurements put
all host launch service at about 0.62% of measured server service time in one
capture, so this alone is not expected to produce a large FPS improvement.

## Reusable proxy argument buffers

The 32-bit proxy caches each function's packed argument layout when argument
metadata is received. It checks the packed byte count before reading the caller's
buffer, then copies argument values into reusable owned byte vectors. Unused
slots are retained for subsequent functions with more arguments. The outer vector
and inner buffers are recovered after both successful and failed IPC requests.
The existing `cuLaunchKernelIn` archive and synchronous IPC exchange are unchanged;
this does not revive the retired host/device transfer-copy experiments. Capacity
tracks the largest argument shapes encountered by this process.
