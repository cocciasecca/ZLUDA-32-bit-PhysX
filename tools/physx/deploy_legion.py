#!/usr/bin/env python3
"""Download and verify a successful PhysX build, then install with game closed."""
import json
import hashlib
import re
import shlex
import struct
import subprocess
import tempfile
from pathlib import Path

REPO = "cocciasecca/ZLUDA-32-bit-PhysX"
BRANCH = "experiment/physx-diagnostics"
# Fixed upstream base of the tested changes; do not silently benchmark a newer revision.
REFERENCE_SHA = "86ec0a93cc83ba5b52e645892523e339dfa884f7"


def run(*args):
    return subprocess.check_output(args, text=True)


def validate_run(info, run_id):
    """Reject incomplete builds before connecting to the device or downloading files."""
    url = f"https://github.com/{REPO}/actions/runs/{run_id}"
    if info["headBranch"] != BRANCH or info["workflowName"] != "ZLUDA":
        raise RuntimeError(
            f"Run non compatibile: serve il workflow ZLUDA del ramo {BRANCH}. {url}"
        )
    if info["status"] != "completed":
        raise RuntimeError(
            f"Build non ancora terminata (stato: {info['status']}). "
            f"Attendi il completamento, poi ripeti lo stesso comando di installazione. {url}"
        )
    if info["conclusion"] != "success":
        raise RuntimeError(
            f"Build terminata con esito {info['conclusion']}; installazione non eseguita. {url}"
        )


def deploy(run_id, host="deck@192.168.10.67", key=None, profile="texture-deferral"):
    if profile not in (
        "texture-deferral",
        "comparison-reference",
        "comparison-modified",
    ):
        raise RuntimeError("Profilo non valido")
    run_id = str(run_id)
    if not run_id.isdecimal() or not re.fullmatch(
        r"[A-Za-z0-9_.-]+@[A-Za-z0-9.-]+", host
    ):
        raise RuntimeError("Run o host non valido")
    key = str(Path(key or "~/.ssh/zluda_legion_ed25519").expanduser())
    info = json.loads(
        run(
            "gh",
            "run",
            "view",
            run_id,
            "--repo",
            REPO,
            "--json",
            "headSha,headBranch,conclusion,workflowName,status",
        )
    )
    validate_run(info, run_id)
    run_sha = info["headSha"]
    reference = profile == "comparison-reference"
    sha = REFERENCE_SHA if reference else run_sha
    artifact = ("physx-reference-" if reference else "physx-servers-") + run_sha
    options = [
        "-i",
        key,
        "-o",
        "IdentitiesOnly=yes",
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
    ]
    # Check before downloading; remote installer checks again before any replacement.
    run(
        "ssh",
        *options,
        host,
        "python3 -c \"import subprocess; assert subprocess.run(['pgrep','-f','[z]luda64_server|[S]hippingPC-BmGame'],capture_output=True).returncode==1, 'Chiudi Batman prima di installare'\"",
    )
    with tempfile.TemporaryDirectory(prefix="zluda-deploy-") as temp:
        root = Path(temp)
        payload = root / "payload"
        run(
            "gh",
            "run",
            "download",
            run_id,
            "--repo",
            REPO,
            "-n",
            artifact,
            "-D",
            str(payload),
        )
        configuration = (
            "windows-reference-86ec0a9-v1" if reference else "windows-texref-cache-v1"
        )
        if "Configuration: " + configuration not in (payload / "README.txt").read_text(
            encoding="utf-8-sig"
        ):
            raise RuntimeError(
                "Build precedente incompatibile: esegui la build della configurazione corrente."
            )
        expected = {
            "32/nvcuda.dll",
            "control/zluda64_server.exe",
            "diagnostic/zluda64_server.exe",
        }
        seen = set()
        for row in (
            (payload / "SHA256SUMS.txt").read_text(encoding="utf-8-sig").splitlines()
        ):
            digest, name = row.split(None, 1)
            name = name.replace("\\", "/")
            if name not in expected or name in seen:
                raise RuntimeError("Manifest inatteso")
            seen.add(name)
            data = (payload / name).read_bytes()
            if hashlib.sha256(data).hexdigest() != digest:
                raise RuntimeError(f"Hash errato: {name}")
            machine = struct.unpack_from(
                "<H", data, struct.unpack_from("<I", data, 60)[0] + 4
            )[0]
            if machine != (0x14C if name.startswith("32/") else 0x8664):
                raise RuntimeError(f"Architettura errata: {name}")
            if sha[:7].encode() not in data and sha[:7].encode("utf-16-le") not in data:
                raise RuntimeError(f"Revisione errata: {name}")
        if seen != expected:
            raise RuntimeError("Pacchetto incompleto")
        remote = run(
            "ssh", *options, host, "mktemp -d /home/deck/zluda-install-XXXXXXXX"
        ).strip()
        if not re.fullmatch(r"/home/deck/zluda-install-[A-Za-z0-9]+", remote):
            raise RuntimeError("Directory remota inattesa")
        subprocess.run(
            [
                "scp",
                *options,
                "-r",
                str(payload),
                str(Path(__file__).with_name("legion_install.py")),
                f"{host}:{remote}/",
            ],
            check=True,
        )
        subprocess.run(
            [
                "ssh",
                *options,
                host,
                "python3 -u "
                + shlex.quote(remote + "/legion_install.py")
                + " "
                + run_id
                + " "
                + shlex.quote(sha)
                + " "
                + shlex.quote(profile),
            ],
            check=True,
        )
        print(
            "Legion aggiornata. Backup nella copia sperimentale; puoi avviare Batman."
        )


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_id")
    parser.add_argument("--host", default="deck@192.168.10.67")
    parser.add_argument("--key")
    args = parser.parse_args()
    deploy(args.run_id, args.host, args.key)
