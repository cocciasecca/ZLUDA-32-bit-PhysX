"""Exercise the future Linux-to-Windows installation without touching the Legion."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import test_install as fixtures

installer = fixtures.module('windows_installer_test', fixtures.HERE.parent/'legion_install.py')

class WindowsMigrationTests(unittest.TestCase):
    def prepare(self, base):
        root, _, prefix = fixtures.InstallationTests().fixture(base)
        (base/'payload/README.txt').write_text('Configuration: windows-borrowed-transfers-v1')
        for name in ['zluda64_server.exe', 'README.txt', 'SHA256SUMS.txt']:
            target = root/'diagnostic-control'/name
            target.parent.mkdir(exist_ok=True)
            target.write_text('old control')
        original = root.parent/'ZludaProton/zluda-libs/libamd_comgr.so.3'
        original.parent.mkdir(parents=True)
        original.write_text('unchanged comgr')
        (root/'proton').write_text('# ZLUDA_NATIVE_LAUNCHER_V1\n')
        (root/'proton.base').write_text('# original proton\n')
        (root/'native-install-last.json').write_text('{"commit":"old native"}')
        return root, prefix

    def run_install(self, base, root):
        with patch.object(installer, 'ROOT', root), patch.object(installer, '__file__', str(base/'legion_install.py')), \
             patch.object(sys, 'argv', ['installer', '456', 'b'*40]), \
             patch.object(installer.subprocess, 'run', return_value=subprocess.CompletedProcess([], 1)):
            installer.main()

    def test_windows_launcher_and_texture_settings_replace_native_mode(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, prefix = self.prepare(base)
            self.run_install(base, root)
            self.assertEqual((root/'proton').read_text(), '# original proton\n')
            settings = {}
            exec((root/'user_settings.py').read_text(), settings)
            self.assertEqual(settings['user_settings']['ZLUDA_NATIVE_BACKEND'], '0')
            self.assertEqual(settings['user_settings']['ZLUDA_PHYSX_TEXTURE_REUSE'], '1')
            self.assertFalse((root/'native-install-last.json').exists())
            self.assertEqual((prefix/'drive_c/windows/syswow64/nvcuda.dll').read_text(), 'game prefix unchanged')
            self.assertEqual((root/'update-backup-456-texture-deferral/proton').read_text(), '# ZLUDA_NATIVE_LAUNCHER_V1\n')
            self.run_install(base, root)  # Idempotent retry.

    def test_failed_settings_activation_restores_launcher_and_dlls(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, _ = self.prepare(base)
            before = {p: p.read_bytes() for p in [root/'proton', root/'user_settings.py', root/'zluda/32/nvcuda.dll']}
            replace = installer.os.replace
            def failing_replace(source, target):
                if Path(target) == root/'user_settings.py':
                    raise OSError('simulated write failure')
                return replace(source, target)
            with patch.object(installer.os, 'replace', side_effect=failing_replace):
                with self.assertRaisesRegex(OSError, 'simulated'):
                    self.run_install(base, root)
            for path, data in before.items():
                self.assertEqual(path.read_bytes(), data)
            self.assertTrue((root/'native-install-last.json').exists())
