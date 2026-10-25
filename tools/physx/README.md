# Windows PhysX bridge workflow

The supported configuration uses the 32-bit CUDA proxy and Windows 64-bit server.
Texture destruction deferral, retention across transfers, and the allocation-range
cache are enabled by the installer. Host transfers use the original owned buffers.
HIP itself is unchanged. Diagnostic counters remain available and enabled by the
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
