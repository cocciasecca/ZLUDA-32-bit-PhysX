#!/usr/bin/env python3
"""Exercise Linux ZLUDA in a separate native process. Not the game's IPC transport."""
import argparse
import ctypes as C
import json
from pathlib import Path
import subprocess
import sys

PTX = b'''.version 6.0
.target sm_30
.address_size 64
.visible .entry increment(.param .u64 data) {
 .reg .b64 %rd<2>;
 .reg .b32 %r<2>;
 ld.param.u64 %rd0, [data];
 ld.global.u32 %r0, [%rd0];
 add.u32 %r1, %r0, 1;
 st.global.u32 [%rd0], %r1;
 ret;
}
'''


class Cuda:
    def __init__(self, library):
        self.lib = C.CDLL(str(Path(library).resolve(strict=True)))
        ptr = C.c_void_p
        u64 = C.c_uint64
        uint = C.c_uint
        signatures = {
            'cuInit': [uint],
            'cuDeviceGet': [C.POINTER(C.c_int), C.c_int],
            'cuCtxCreate_v2': [C.POINTER(ptr), uint, C.c_int],
            'cuCtxDestroy_v2': [ptr],
            'cuMemAlloc_v2': [C.POINTER(u64), C.c_size_t],
            'cuMemFree_v2': [u64],
            'cuMemcpyHtoD_v2': [u64, ptr, C.c_size_t],
            'cuMemcpyDtoH_v2': [ptr, u64, C.c_size_t],
            'cuModuleLoadData': [C.POINTER(ptr), ptr],
            'cuModuleGetFunction': [C.POINTER(ptr), ptr, C.c_char_p],
            'cuModuleUnload': [ptr],
            'cuLaunchKernel': [ptr, uint, uint, uint, uint, uint, uint, uint,
                               ptr, C.POINTER(ptr), C.POINTER(ptr)],
            'cuCtxSynchronize': [],
        }
        for name, args in signatures.items():
            fn = getattr(self.lib, name)
            fn.argtypes = args
            fn.restype = C.c_int

    def call(self, name, *args):
        status = getattr(self.lib, name)(*args)
        if status:
            raise RuntimeError(f'{name}: CUDA status {status}')


def exercise(cuda):
    """All handles stay in this process. Cleanup also runs on failures."""
    ctx, module, function = C.c_void_p(), C.c_void_p(), C.c_void_p()
    device, allocation = C.c_int(), C.c_uint64()
    cleanup = []
    try:
        cuda.call('cuInit', 0)
        cuda.call('cuDeviceGet', C.byref(device), 0)
        cuda.call('cuCtxCreate_v2', C.byref(ctx), 0, device)
        cleanup.append(('cuCtxDestroy_v2', ctx))
        cuda.call('cuMemAlloc_v2', C.byref(allocation), 4)
        cleanup.append(('cuMemFree_v2', allocation))
        cuda.call('cuModuleLoadData', C.byref(module), C.c_char_p(PTX))
        cleanup.append(('cuModuleUnload', module))
        cuda.call('cuModuleGetFunction', C.byref(function), module, b'increment')
        args = (C.c_void_p * 1)(C.cast(C.byref(allocation), C.c_void_p))
        results = []
        for initial in (41, 99):
            value = C.c_uint32(initial)
            cuda.call('cuMemcpyHtoD_v2', allocation, C.byref(value), 4)
            cuda.call('cuLaunchKernel', function, 1, 1, 1, 1, 1, 1, 0, None, args, None)
            cuda.call('cuCtxSynchronize')
            cuda.call('cuMemcpyDtoH_v2', C.byref(value), allocation, 4)
            if value.value != initial + 1:
                raise RuntimeError(f'Wrong kernel result: {value.value}, expected {initial + 1}')
            results.append(value.value)
        return {'backend': 'linux-native', 'results': results}
    finally:
        # Attempt every release and preserve an earlier CUDA error if present.
        failed = sys.exc_info()[0] is not None
        errors = []
        for name, handle in reversed(cleanup):
            try:
                cuda.call(name, handle)
            except Exception as error:
                errors.append(str(error))
        if errors:
            if failed:
                print('Cleanup: ' + '; '.join(errors), file=sys.stderr)
            else:
                raise RuntimeError('; '.join(errors))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--library', required=True, help='Absolute path to Linux ZLUDA libcuda.so')
    parser.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    args = parser.parse_args()
    if sys.platform != 'linux' or C.sizeof(C.c_void_p) != 8:
        parser.error('Serve Linux a 64 bit con ZLUDA e HIP nativi disponibili.')
    if args.worker:
        print(json.dumps(exercise(Cuda(args.library))))
    else:
        # Isolate context ownership and crashes; fail rather than hanging indefinitely.
        subprocess.run([sys.executable, str(Path(__file__).resolve()), '--worker',
                        '--library', str(Path(args.library).resolve(strict=True))],
                       check=True, timeout=180)


if __name__ == '__main__':
    main()
