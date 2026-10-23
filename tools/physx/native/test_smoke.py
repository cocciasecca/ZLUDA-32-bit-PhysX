import ctypes as C
import unittest
from smoke import exercise


class FakeCuda:
    def __init__(self, fail=None, wrong=False):
        self.calls = []
        self.fail = fail
        self.wrong = wrong
        self.value = 0

    def call(self, name, *args):
        self.calls.append(name)
        if name == self.fail:
            raise RuntimeError(name)
        if name in ('cuCtxCreate_v2', 'cuModuleLoadData', 'cuModuleGetFunction'):
            C.cast(args[0], C.POINTER(C.c_void_p))[0] = 123
        elif name == 'cuMemAlloc_v2':
            C.cast(args[0], C.POINTER(C.c_uint64))[0] = 256
        elif name == 'cuMemcpyHtoD_v2':
            self.value = C.cast(args[1], C.POINTER(C.c_uint32))[0]
        elif name == 'cuLaunchKernel':
            self.value += 0 if self.wrong else 1
        elif name == 'cuMemcpyDtoH_v2':
            C.cast(args[0], C.POINTER(C.c_uint32))[0] = self.value


class SmokeTests(unittest.TestCase):
    def test_results_and_cleanup(self):
        cuda = FakeCuda()
        self.assertEqual(exercise(cuda)['results'], [42, 100])
        self.assertEqual(cuda.calls[-3:], ['cuModuleUnload', 'cuMemFree_v2', 'cuCtxDestroy_v2'])

    def test_partial_initialization_cleanup(self):
        cuda = FakeCuda(fail='cuModuleLoadData')
        with self.assertRaisesRegex(RuntimeError, 'cuModuleLoadData'):
            exercise(cuda)
        self.assertEqual(cuda.calls[-2:], ['cuMemFree_v2', 'cuCtxDestroy_v2'])
        self.assertNotIn('cuModuleUnload', cuda.calls)

    def test_incorrect_gpu_result_rejected(self):
        with self.assertRaisesRegex(RuntimeError, 'Wrong kernel result'):
            exercise(FakeCuda(wrong=True))

    def test_release_failure_does_not_skip_other_releases(self):
        cuda = FakeCuda(fail='cuModuleUnload')
        with self.assertRaisesRegex(RuntimeError, 'cuModuleUnload'):
            exercise(cuda)
        self.assertEqual(cuda.calls[-1], 'cuCtxDestroy_v2')


if __name__ == '__main__':
    unittest.main()
