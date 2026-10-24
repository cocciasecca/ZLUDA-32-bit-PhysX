import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


HERE = Path(__file__).resolve().parent
installer = module('native_installer_test', HERE.parent/'legion_install.py')
rollback = module('native_rollback_test', HERE/'rollback_native.py')
sha = lambda p: hashlib.sha256(p.read_bytes()).hexdigest()


class InstallationTests(unittest.TestCase):
    def fixture(self, base, fails=False):
        root = base/'Steam/compatibilitytools.d/ZludaProton-Experiment'
        root.mkdir(parents=True)
        initial = {
            'proton': '#!/usr/bin/env python3\nprint("old proton")\n',
            'user_settings.py': 'user_settings = {}\n',
            'zluda/32/nvcuda.dll': 'old dll',
            'zluda/zluda64_server.exe': 'old server',
            'files/share/default_pfx/drive_c/windows/syswow64/nvcuda.dll': 'old template dll',
            'zluda-libs/libamd_comgr.so.3': 'unchanged comgr',
            'experiment-manifest.json': '{"original_files": {}}',
        }
        for name, data in initial.items():
            path = root/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data)
        (root/'proton').chmod(0o755)
        prefix = root.parents[1]/'steamapps/compatdata/35140/pfx'
        for arch in ('system32', 'syswow64'):
            path = prefix/'drive_c/windows'/arch/'nvcuda.dll'
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('game prefix unchanged')
        payload = base/'payload'
        names = ['32/nvcuda.dll', '32/native-dll-probe.exe', 'control/zluda64_server.exe', 'diagnostic/zluda64_server.exe']
        for name in names:
            path = payload/name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text('new '+name)
        (payload/'SHA256SUMS.txt').write_text('\n'.join(sha(payload/name)+'  '+name for name in names))
        (payload/'README.txt').write_text('Configuration: native-linux-physx-v1')
        native = payload/'native'
        native.mkdir()
        for name in ('zluda64_server', 'libnvcuda.so', 'rollback_native.py'):
            (native/name).write_text('native '+name)
        (native/'native_runtime.py').write_text('def preflight(*args):\n    '+('raise RuntimeError("GPU preflight failed")' if fails else 'pass')+'\n')
        files = {p.name: sha(p) for p in native.iterdir()}
        (native/'manifest.json').write_text(json.dumps({'schema': 1, 'revision': 'a'*40,
            'configuration': 'native-linux-physx-v1', 'files': files}))
        return root, initial, prefix

    def install(self, base, root):
        with patch.object(installer, 'gpu_preflight', side_effect=lambda runtime, *args: runtime.preflight(*args)), \
             patch.object(installer, 'ROOT', root), patch.object(installer, '__file__', str(base/'legion_install.py')), \
             patch.object(sys, 'argv', ['installer', '123', 'a'*40, 'native-linux']), \
             patch.object(installer.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)):
            installer.main()

    def test_failed_gpu_probe_leaves_working_installation_untouched(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root, initial, prefix = self.fixture(base, fails=True)
            with self.assertRaisesRegex(RuntimeError, 'GPU preflight failed'):
                self.install(base, root)
            for name, contents in initial.items():
                self.assertEqual((root/name).read_text(), contents)
            self.assertFalse((root/'native').exists())
            self.assertFalse((root/'native-install-last.json').exists())
            self.assertEqual((prefix/'drive_c/windows/syswow64/nvcuda.dll').read_text(), 'game prefix unchanged')

    def test_activation_and_exact_rollback(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root, initial, prefix = self.fixture(base)
            self.install(base, root)
            self.assertIn('ZLUDA_NATIVE_LAUNCHER_V1', (root/'proton').read_text())
            self.assertEqual((root/'proton.base').read_text(), initial['proton'])
            if os.name == 'posix':
                self.assertTrue((root/'native/zluda64_server').stat().st_mode & 0o111)
            with patch.object(rollback, '__file__', str(root/'native/rollback_native.py')), \
                 patch.object(rollback.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)):
                rollback.main()
            for name, contents in initial.items():
                self.assertEqual((root/name).read_text(), contents)
            self.assertFalse((root/'proton.base').exists())
            self.assertFalse((root/'native').exists())
            self.assertEqual((prefix/'drive_c/windows/system32/nvcuda.dll').read_text(), 'game prefix unchanged')

    def test_running_game_blocks_before_mutation(self):
        with patch.object(installer.subprocess, 'run', return_value=subprocess.CompletedProcess([], 0)):
            with self.assertRaisesRegex(RuntimeError, 'Chiudi Batman'):
                installer.closed()

    def test_rollback_refuses_to_overwrite_user_changes(self):
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            root, _, _ = self.fixture(base)
            self.install(base, root)
            (root/'user_settings.py').write_text('my later edits')
            with patch.object(rollback, '__file__', str(root/'native/rollback_native.py')), \
                 patch.object(rollback.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)):
                with self.assertRaisesRegex(RuntimeError, 'File modificato'):
                    rollback.main()
            self.assertEqual((root/'user_settings.py').read_text(), 'my later edits')
            self.assertTrue((root/'native').exists())


if __name__ == '__main__':
    unittest.main()

class KernelResultTests(unittest.TestCase):
    def test_clean_exit_without_kernel_proof_is_rejected(self):
        for output in ('', 'Windows DLL -> native server -> DLL: 100'):
            with self.assertRaisesRegex(RuntimeError, 'Manca la conferma'):
                installer.require_kernel_result(output)

    def test_expected_kernel_result_is_required(self):
        installer.require_kernel_result('Actual DLL -> Linux ZLUDA -> PTX32 kernel verified: 100 -> 101\n')
