#!/usr/bin/env python3
"""Run portable checks once per source/compiler revision; Windows integration stays in CI."""
import ast
import hashlib
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path


def main():
    root = Path(__file__).resolve().parents[2]
    subprocess.run(["git", "diff", "--check"], cwd=root, check=True)
    scripts = sorted((root / "tools/physx").rglob("*.py"))
    for script in scripts:
        ast.parse(script.read_text(), filename=str(script))
    subprocess.run(
        [
            sys.executable,
            "-B",
            "-m",
            "unittest",
            "discover",
            "-s",
            str(root / "tools/physx/tests"),
            "-p",
            "test_*.py",
        ],
        check=True,
    )
    rustc = shutil.which("rustc")
    if not rustc:
        portable = Path("/private/tmp/zluda-rust-toolchain/bin/rustc")
        rustc = str(portable) if portable.is_file() else None
    if not rustc:
        print(
            "Python e diff verificati. Rust non disponibile: test Rust eseguiti da GitHub Actions."
        )
        return
    version = subprocess.check_output([rustc, "-vV"])
    names = ["diagnostics", "texture_diagnostics", "texture_reuse", "texture_shadow"]
    sources = [root / f"zluda64_server/src/{name}.rs" for name in names]
    sources.append(root / "zluda32/src/range_cache.rs")
    sources.append(root / "zluda32/src/texture_metadata.rs")
    digest = hashlib.sha256(version + Path(__file__).read_bytes())
    for source in sources:
        digest.update(str(source).encode())
        digest.update(source.read_bytes())
    cache = Path(tempfile.gettempdir()) / "zluda-local-checks"
    cache.mkdir(exist_ok=True)
    marker = cache / digest.hexdigest()
    if marker.exists():
        print(
            "Test portabili già superati per queste sorgenti e compilatore.", flush=True
        )
        return
    with tempfile.TemporaryDirectory(prefix="zluda-check-") as directory:
        for source in sources:
            binary = Path(directory) / (
                source.stem + (".exe" if sys.platform == "win32" else "")
            )
            print(f"Test: {source.stem}", flush=True)
            subprocess.run(
                [rustc, "--edition=2021", "--test", str(source), "-o", str(binary)],
                check=True,
            )
            subprocess.run([str(binary), "--quiet"], check=True)
    marker.touch()
    print(
        "Test portabili superati. Compilazione e integrazione Windows: GitHub Actions.",
        flush=True,
    )


if __name__ == "__main__":
    main()
