"""Temporary installation tree; never accesses the real device."""

import hashlib
import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent


def module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    result = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(result)
    return result


sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()


class InstallationFixture:
    def fixture(self, base):
        root = base / "Steam/compatibilitytools.d/ZludaProton-Experiment"
        root.mkdir(parents=True)
        initial = {
            "proton": '#!/usr/bin/env python3\nprint("old proton")\n',
            "user_settings.py": "user_settings = {}\n",
            "zluda/32/nvcuda.dll": "old dll",
            "zluda/zluda64_server.exe": "old server",
            "files/share/default_pfx/drive_c/windows/syswow64/nvcuda.dll": "old template dll",
            "zluda-libs/libamd_comgr.so.3": "unchanged comgr",
            "experiment-manifest.json": '{"original_files": {}}',
        }
        for name, data in initial.items():
            path = root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(data)
        (root / "proton").chmod(0o755)
        prefix = root.parents[1] / "steamapps/compatdata/35140/pfx"
        for arch in ("system32", "syswow64"):
            path = prefix / "drive_c/windows" / arch / "nvcuda.dll"
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("game prefix unchanged")
        payload = base / "payload"
        names = [
            "32/nvcuda.dll",
            "control/zluda64_server.exe",
            "diagnostic/zluda64_server.exe",
        ]
        for name in names:
            path = payload / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("new " + name)
        (payload / "SHA256SUMS.txt").write_text(
            "\n".join(sha(payload / name) + "  " + name for name in names)
        )
        (payload / "README.txt").write_text(
            "Configuration: windows-original-transfers-v1"
        )
        return root, initial, prefix
