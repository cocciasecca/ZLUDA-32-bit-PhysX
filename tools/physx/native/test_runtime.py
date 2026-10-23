import os
from pathlib import Path
import tempfile
import unittest
from native_runtime import ServerSession, environment, run_proton


class EnvironmentTests(unittest.TestCase):
    def test_gpu_settings_are_preserved_for_native_process(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            (root/'user_settings.py').write_text('user_settings = {"HSA_OVERRIDE_GFX_VERSION": "11.0.0", "PROTON_LOG": "1"}\n')
            env = environment(root, root/'native')
            self.assertEqual(env['HSA_OVERRIDE_GFX_VERSION'], '11.0.0')
            self.assertEqual(env['ZLUDA_PHYSX_TEXTURE_REUSE'], '1')
            self.assertEqual(env['ZLUDA_NATIVE_LIBRARY'], str(root/'native/libnvcuda.so'))


@unittest.skipUnless(os.name == 'posix', 'Linux supervisor lifecycle')
class LifecycleTests(unittest.TestCase):
    def fake(self, root, ready=True):
        native = root/'native'
        native.mkdir()
        server = native/'zluda64_server'
        server.write_text('#!/usr/bin/env python3\nimport time\nprint("'+('127.0.0.1:12345' if ready else 'bad startup')+'", flush=True)\ntime.sleep(60)\n')
        server.chmod(0o755)
        return native

    def test_exit_reaps_supervisor(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            session = ServerSession(root, self.fake(root))
            with session:
                self.assertEqual(session.env['ZLUDA_NATIVE_ADDR'], '127.0.0.1:12345')
                self.assertEqual(len(session.env['ZLUDA_NATIVE_TOKEN']), 64)
                self.assertIsNone(session.process.poll())
            self.assertIsNotNone(session.process.poll())

    def test_bad_startup_reaps_process(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            session = ServerSession(root, self.fake(root, False))
            with self.assertRaisesRegex(RuntimeError, 'Avvio server Linux fallito'):
                session.__enter__()
            self.assertIsNotNone(session.process.poll())

    def test_wrapper_preserves_proton_exit_code(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            self.fake(root)
            (root/'proton.base').write_text('import os, sys\nassert os.environ["ZLUDA_NATIVE_BACKEND"] == "1"\nsys.exit(7)\n')
            self.assertEqual(run_proton(root, ['run']), 7)


if __name__ == '__main__':
    unittest.main()
