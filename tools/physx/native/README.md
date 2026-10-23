# Linux native backend feasibility: first step

This is an isolated native CUDA ABI smoke test, not a Proton bridge or a performance
prototype. It explicitly loads the supplied Linux ZLUDA library in a child process,
creates a context, allocates four bytes, loads a PTX increment kernel, copies data,
launches twice, checks 42/100 and releases resources. Nonzero CUDA status, wrong output,
child crashes and timeouts fail the command. It does not install or modify anything.

On Linux x86-64, with a built ZLUDA shared library and its HIP/ROCm dependencies:

```sh
python3 tools/physx/native/smoke.py --library /absolute/path/to/libnvcuda.so
```

Use the actual built ZLUDA library path (a distribution may name it `libcuda.so`).
Library dependencies must already be available to the dynamic loader. This command
does not install ROCm or replace the working game backend.

Local/CI tests use a fake CUDA API to test orchestration, errors and cleanup. They do
not validate PTX compilation, the real CUDA ABI on GPU, or driver compatibility.

Next steps after a successful native GPU run: define a versioned binary IPC protocol
with opaque server-owned handles; implement the corresponding Windows 32-bit client;
port server dispatch and allocation mapping. Existing IPC uses Windows handles/events
and cannot simply be attached to this process. The production transport must not use
Python/JSON per CUDA call. Preserve the current texture-deferral baseline throughout.
