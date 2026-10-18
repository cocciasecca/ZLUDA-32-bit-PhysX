# Changes

Experimental branch: `experiment/physx-diagnostics`; separate worktree `/private/tmp/zluda-physx-diagnostics`.

- `zluda64_server/Cargo.toml`: optional `physx-diagnostics` feature, no default enablement or dependencies.
- `zluda64_server/src/diagnostics.rs`: aggregate service count, nonzero CUDA response count, summed/max nanoseconds per request type in five-second windows. New CSV path from ZLUDA_PHYSX_DIAGNOSTICS; create_new prevents truncation; file/I/O failure disables collection. Request types bound in-memory cardinality. No per-frame output. Headers identify PID, build commit and approximate wall-clock origin.
- `zluda64_server/src/main.rs`: hooks in all three supported request handler wrappers, finish BEFORE SignalObjectAndWait. Feature-disabled builds contain no timing hooks. Unsupported opcodes/startup/resize control messages not covered. No CUDA ordering or memory lifetime changes.

Limitations: file size grows over the run (use bounded 10–30 s captures); up to the final five seconds remain unflushed when idle or killed. No frame percentiles, HIP/KFD counters, transfer sizes or allocation lifetimes yet. CSV names are Rust input type names. Service times include serialization and nested synchronization; they are not GPU execution time or end-to-end IPC latency. Diagnostic I/O is outside measured service duration but delays response delivery.

Validation: git diff --check passes; two portable unit tests compiled and passed with isolated Rust 1.90.0 on aarch64 macOS. Full Windows build and CUDA compatibility tests not run: Windows/HIP builder unavailable. This prototype must not be installed as a validated optimization.

## Build / integration plan (not executed)

Follow docs/src/building.md and .github/workflows/pr_master.yml on a prepared Windows builder (Rust, C++/CMake, submodules, HIP SDK). Build baseline from 86ec0a9. Build server with `cargo build --release -p zluda64_server --features physx-diagnostics --target x86_64-pc-windows-msvc`; test the diagnostic module separately using `rustc --edition=2021 --test zluda64_server/src/diagnostics.rs -o <temporary-test-executable>`. Build normal server with feature omitted as a control.

Use the exact Proton-ZLUDA release's convert.sh only in a disposable staging area with an explicitly NEW output path: the upstream script deletes its output directory! Set TOOL_NAME=ZludaProton-Experiment, ZLUDA_DIR to matching Windows payload, HIPWINE_LIB to matching shim and ROCM_LIBS to a verified copy of the working library set including the COMGR fix. Do not let auto-bundling substitute host libraries. Hash manifest before/after. Approval required before target writes, launch changes or running the game.

Set ZLUDA_PHYSX_DIAGNOSTICS to a unique Wine-visible CSV file outside game/prefix paths for each approved run. Verify output and instrumented/uninstrumented overhead first. Preserve original LD_LIBRARY_PATH semantics but ensure experimental runs resolve the intended copied library set.

Rollback: switch back to original ZludaProton and exact original launch options after approval; preserve original tool, COMGR and prefix. Switching tools can modify prefix DLLs: back up affected files/configuration before any test and verify restoration hashes. Do not delete/reset prefix. No rollback currently needed on device: nothing changed.

## Existing GitHub workflow adaptation

.github/workflows/push_master.yml now routes experiment/physx-diagnostics to Build PhysX diagnostic server (Windows), using windows-2022, stable Rust, --locked, x86_64-pc-windows-msvc and release profile. Builds separate control and diagnostic binaries and tests the portable collector. Uses contents:read; does not publish a release on this branch. Existing jobs retain behavior on other refs. Output artifact: physx-servers-<full SHA>, containing control/, diagnostic/, README.txt and SHA256SUMS.txt. This is not a complete Proton package.

UI: Actions -> ZLUDA (push_master.yml) -> Run workflow -> branch experiment/physx-diagnostics -> Run workflow. Existing workflow is already on default branch, so manual dispatch can select our branch. Do not choose master for the diagnostic run. Windows build result remains unverified until execution.

## Windows CI validation — run 39159028893

GitHub diagnostic branch build at a6379f0000c5abffb731261f8de42cfda7d9150d succeeded. Windows portable tests and both server builds passed; full/GPU jobs skipped as intended. Job duration approximately 3m40s. Downloaded artifact into /private/tmp/zluda-physx-build-39159028893 and verified both executable SHA256 values against bundled manifest. No target installation or game testing. Next: prepare separate experimental tool integration for user approval; no FPS improvement established.

## Integration correction: matching proxy required

Before installation, inspected zluda32/src/ipc.rs::read_git_version: it rejects a server whose embedded Git SHA differs. Previous server-only artifact cannot be paired with original installed 32-bit DLL. No incompatible binary installed. Workflow corrected to build/package zluda32 for i686-pc-windows-msvc from the same commit as both servers; checksum manifest includes DLL. Preserve existing handshake, protocol and original 64-bit ZLUDA library. User authorized preparing/installing separate experimental tool; no game launch or Steam setting change authorized by this action.
