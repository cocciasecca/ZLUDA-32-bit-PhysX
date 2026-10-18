# Environment and working configuration

## Target system

- Device: Lenovo Legion Go
- CPU/APU: AMD Ryzen Z1 Extreme (Phoenix1)
- GPU: Radeon 780M, HSA agent `gfx1103`, PCI ID reported as `15bf`
- OS: SteamOS (user connected using SSH as `deck@steamdeck`)
- Kernel observed in Proton log: `6.18.50-valve2-1-neptune-618-gc7289a96b14d`
- glibc: `2.41`
- Game: Batman: Arkham Asylum GOTY, Steam AppID `35140` (Windows 32-bit)
- Proton-ZLUDA tool release used: `tool-11.0-20261005-r2`, based on proton-cachyos

## Paths on the device

```text
/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton/
/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton/zluda/
/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton/zluda/32/nvcuda.dll
/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton/zluda/zluda64_server.exe
/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton/zluda-libs/
/home/deck/.local/share/Steam/steamapps/compatdata/35140/pfx/
/home/deck/.local/share/Steam/steamapps/common/Batman Arkham Asylum GOTY/Binaries/
/home/deck/steam-35140.log
```

Paths should be checked before use; the Steam installation location could differ in future.

## Steam launch option known to work for starting the game

```bash
LD_LIBRARY_PATH="$HOME/.local/share/Steam/compatibilitytools.d/ZludaProton/zluda-libs${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}" PROTON_LOG=1 %command%
```

Use `ZludaProton` as the forced compatibility tool. `PROTON_LOG=1` is diagnostic only. Avoid `WINEDLLOVERRIDES="nvcuda="`, since that disables CUDA during testing.

## Fixes already applied

1. `libgflags.so.2.2` existed in `zluda-libs`, but was not initially found; setting `LD_LIBRARY_PATH` resolved that dependency.
2. HIP initially returned `hipErrorNoDevice (100)` due to missing COMGR; detailed AMD logs showed `Failed to load COMGR library` and `Code object manager initialization failed for HSA device gfx1103`.
3. An Arch/CachyOS COMGR binary required `GLIBC_2.44` and could not run on glibc 2.41. It was replaced with `libamd_comgr.so.3` extracted from AMD's **ROCm 7.2.3 Ubuntu 22.04** `.deb` package at `repo.radeon.com`. This library is in `ZludaProton/zluda-libs/` and passes the local `ldd` dependency check.
4. HIP subsequently returned `hipGetDeviceCount = 0`, device count `1`.
5. NVIDIA PhysX System Software `9.23.1019` was installed **inside the game's Proton prefix**, not as a host NVIDIA driver. The game loaded PhysX components from `Program Files (x86)/NVIDIA Corporation/PhysX/`.

## Validation already performed

- HSA initialized with `hsa_init() = 0`, enumerating the CPU and `gfx1103` GPU.
- SHA256 of `syswow64/nvcuda.dll` matched `ZludaProton/zluda/32/nvcuda.dll`.
- Game loaded `PhysXCore.dll`, `PhysXCooking.dll`, `PhysXLoader.dll`, `cudart32_65.dll`, and ZLUDA's `nvcuda.dll`.
- `zluda64_server.exe` was running during gameplay.
- Standard Proton previously ran the game correctly. Proton-ZLUDA now starts it, but GPU PhysX performance is inconsistent.

Do not infer that CUDA kernel execution or GPU acceleration is fully correct merely because HIP sees one GPU or the DLLs load.
