"""Verify Windows installation and native-wrapper rollback in temporary directories."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from unittest.mock import patch
import install_fixtures as fixtures

installer = fixtures.module(
    "windows_installer_test", fixtures.HERE.parent / "legion_install.py"
)


class WindowsMigrationTests(unittest.TestCase):
    def prepare(self, base):
        root, _, prefix = fixtures.InstallationFixture().fixture(base)
        (base / "payload/README.txt").write_text(
            "Configuration: windows-texref-cache-v1"
        )
        for name in ["zluda64_server.exe", "README.txt", "SHA256SUMS.txt"]:
            target = root / "diagnostic-control" / name
            target.parent.mkdir(exist_ok=True)
            target.write_text("old control")
        original = root.parent / "ZludaProton/zluda-libs/libamd_comgr.so.3"
        original.parent.mkdir(parents=True)
        original.write_text("unchanged comgr")
        (root / "proton").write_text("# ZLUDA_NATIVE_LAUNCHER_V1\n")
        (root / "proton.base").write_text("# original proton\n")
        (root / "native-install-last.json").write_text('{"commit":"old native"}')
        return root, prefix

    def run_install(self, base, root, mode="texture-deferral"):
        with patch.object(installer, "ROOT", root), patch.object(
            installer, "__file__", str(base / "legion_install.py")
        ), patch.object(
            sys, "argv", ["installer", "456", "b" * 40, mode]
        ), patch.object(
            installer.subprocess, "run", return_value=subprocess.CompletedProcess([], 1)
        ):
            installer.main()

    def test_windows_launcher_and_texture_settings_replace_native_mode(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, prefix = self.prepare(base)
            self.run_install(base, root)
            self.assertEqual((root / "proton").read_text(), "# original proton\n")
            settings = {}
            exec((root / "user_settings.py").read_text(), settings)
            self.assertEqual(settings["user_settings"]["ZLUDA_NATIVE_BACKEND"], "0")
            self.assertEqual(
                settings["user_settings"]["ZLUDA_PHYSX_TEXTURE_REUSE"], "1"
            )
            self.assertFalse((root / "native-install-last.json").exists())
            self.assertEqual(
                (prefix / "drive_c/windows/syswow64/nvcuda.dll").read_text(),
                "game prefix unchanged",
            )
            self.assertEqual(
                (root / "update-backup-456-texture-deferral/proton").read_text(),
                "# ZLUDA_NATIVE_LAUNCHER_V1\n",
            )
            self.run_install(base, root)  # Idempotent retry.

    def test_failed_settings_activation_restores_launcher_and_dlls(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, _ = self.prepare(base)
            before = {
                p: p.read_bytes()
                for p in [
                    root / "proton",
                    root / "user_settings.py",
                    root / "zluda/32/nvcuda.dll",
                ]
            }
            replace = installer.os.replace

            def failing_replace(source, target):
                if Path(target) == root / "user_settings.py":
                    raise OSError("simulated write failure")
                return replace(source, target)

            with patch.object(installer.os, "replace", side_effect=failing_replace):
                with self.assertRaisesRegex(OSError, "simulated"):
                    self.run_install(base, root)
            for path, data in before.items():
                self.assertEqual(path.read_bytes(), data)
            self.assertTrue((root / "native-install-last.json").exists())

    def test_comparison_switches_matching_payload_and_disables_diagnostics(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, _ = self.prepare(base)
            library = (root / "zluda-libs/libamd_comgr.so.3").read_bytes()
            for mode, configuration, enabled, server in [
                (
                    "comparison-reference",
                    "windows-reference-86ec0a9-v1",
                    "0",
                    "control",
                ),
                (
                    "comparison-modified",
                    "windows-texref-cache-v1",
                    "1",
                    "diagnostic",
                ),
                (
                    "comparison-reference",
                    "windows-reference-86ec0a9-v1",
                    "0",
                    "control",
                ),
                (
                    "texture-deferral",
                    "windows-texref-cache-v1",
                    "1",
                    "diagnostic",
                ),
            ]:
                (base / "payload/README.txt").write_text(
                    "Configuration: " + configuration
                )
                self.run_install(base, root, mode)
                settings = {}
                exec((root / "user_settings.py").read_text(), settings)
                settings = settings["user_settings"]
                for flag in (
                    "TEXTURE_REUSE",
                    "TEXTURE_REUSE_TRANSFERS",
                    "RANGE_CACHE",
                    "TEXREF_CACHE",
                ):
                    self.assertEqual(settings["ZLUDA_PHYSX_" + flag], enabled)
                self.assertEqual("ZLUDA_PHYSX_DIAGNOSTICS" in settings, False)
                self.assertEqual(
                    (root / "zluda/zluda64_server.exe").read_bytes(),
                    (base / "payload" / server / "zluda64_server.exe").read_bytes(),
                )
                self.assertEqual(
                    (root / "zluda/32/nvcuda.dll").read_bytes(),
                    (base / "payload/32/nvcuda.dll").read_bytes(),
                )
                self.assertEqual(
                    (root / "zluda-libs/libamd_comgr.so.3").read_bytes(), library
                )
                record = json.loads(
                    (root / f"update-manifest-456-{mode}.json").read_text()
                )
                self.assertEqual(record["diagnostics_enabled"], False)
                self.assertEqual(record["range_cache_enabled"], enabled == "1")

    def test_reference_rejects_modified_package_before_replacement(self):
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            root, _ = self.prepare(base)
            before = (root / "zluda/32/nvcuda.dll").read_bytes()
            with self.assertRaisesRegex(AssertionError, "Incompatible package"):
                self.run_install(base, root, "comparison-reference")
            self.assertEqual((root / "zluda/32/nvcuda.dll").read_bytes(), before)
            self.assertFalse(list(root.glob("update-backup-*")))
