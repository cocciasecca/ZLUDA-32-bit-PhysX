# Native Linux PhysX backend

The current code routes the existing Windows 32-bit nvcuda.dll wire protocol to a
Linux x86-64 `zluda64_server`, using the same CUDA dispatcher as the working Windows
server. Module loading, PTX32 argument handling, textures, streams, events, allocation
translation, range caching and texture-destruction deferral retain their existing
implementations. The old limited smoke-only RPC/DLL implementation has been removed.
This preserves the existing PhysX API coverage; it is not a claim to implement all CUDA.

Messages use versioned binary framing over loopback TCP, per-launch authentication,
matching Git revisions, monotonically increasing sequence numbers and a 256 MiB
payload limit. Buffers preserve the previous wire alignment and serialization layout.
Each DLL connection gets its own Linux process/context and allocation arena. No Windows
CUDA server is started in native mode. The native library is loaded by absolute path.
Host-to-device asynchronous copies explicitly finish before the receive buffer can be
reused, preventing lifetime errors; this initial port does not promise less overhead.

## Build and installation

Use the existing `tools/physx/push_build.py --yes --install` command. Actions builds:

- Matching Windows x86 DLL and probe, plus the Windows servers kept for fallback.
- Full Linux ZLUDA and server with texture deferral enabled, using ROCm 7.2.4 build
  dependencies. No HIP source or target-machine HIP installation is modified.

The first native build includes LLVM and can be substantially slower; LLVM build
outputs are cached for subsequent runs. There is one current install configuration.

Deployment downloads both artifacts from the same successful run. The installer checks
hashes/revisions and game/server closure, then runs the actual x86 DLL through the
existing Proton executable in a **separate test compatdata directory**. The test checks
copies and a PTX32 kernel on Linux ZLUDA. Missing libraries, errors or a timeout abort
before changing the working installation. The GPU test is not a Batman performance test.

Only after this preflight does installation activate a Proton wrapper. It starts the
Linux supervisor, passes ephemeral connection credentials to the DLL and terminates
all native workers when Proton exits. Existing native HIP/COMGR libraries and game
prefix/saves are preserved. Native logs go to `diagnostics/native-*.log` and `.csv`.

A backup and transaction manifest support rollback:

```sh
python3 ~/.local/share/Steam/compatibilitytools.d/ZludaProton-Experiment/native/rollback_native.py
```

Rollback also checks that Batman/server are closed and refuses to overwrite edits made
after installation. It restores the previous DLLs, settings and Proton launcher.

## Verification boundaries

Portable tests cover framing, transport, aligned buffer growth, configuration,
process lifetime, failed preflight, activation and rollback. Windows CI additionally
loads the actual DLL against a simulated peer using the real CUDA wire structs.
Linux CI compiles the full backend. Installation requires the real GPU kernel test.
None of these establishes a Batman FPS gain or absence of game-specific bugs.
