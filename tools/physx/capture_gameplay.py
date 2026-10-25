#!/usr/bin/env python3
"""Read-only Linux gameplay sampler. Run on the device; JSON goes to stdout.

CPU percentages use one logical core as 100%. This does not measure frame times
or distinguish GPU rendering from compute. No game/runtime settings are changed.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import statistics
import time


def process_ticks(path):
    # comm can contain spaces/parentheses; fields after its closing ')' start at 3.
    fields = (path / "stat").read_text().rsplit(")", 1)[1].split()
    return int(fields[11]) + int(fields[12])


def inspect_process(path):
    env = dict(
        item.split(b"=", 1)
        for item in (path / "environ").read_bytes().split(b"\0")
        if b"=" in item
    )
    flags = [
        "ZLUDA_PHYSX_TEXTURE_REUSE",
        "ZLUDA_PHYSX_TEXTURE_REUSE_TRANSFERS",
        "ZLUDA_PHYSX_RANGE_CACHE",
        "ZLUDA_PHYSX_TEXREF_CACHE",
        "ZLUDA_PHYSX_DIAGNOSTICS",
    ]
    libraries = sorted(
        {
            line.split()[-1]
            for line in (path / "maps").read_text().splitlines()
            if any(
                name in line.lower()
                for name in ("nvcuda.dll", "zluda64_server.exe", "libamdhip64.so")
            )
        }
    )
    hashes = {}
    for library in libraries:
        if Path(library).is_file():
            hashes[library] = hashlib.sha256(Path(library).read_bytes()).hexdigest()
    return {
        "pid": int(path.name),
        "flags": {
            key: env.get(key.encode(), b"<unset>").decode(errors="replace")
            for key in flags
        },
        "loaded_file_hashes": hashes,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--seconds", type=int, default=45)
    args = parser.parse_args()
    if not 1 <= args.seconds <= 300:
        parser.error("--seconds must be between 1 and 300")
    processes = {}
    for path in Path("/proc").iterdir():
        if not path.name.isdecimal():
            continue
        try:
            # Wine may name the server thread "main". Match argv[0], not comm
            # or the whole command line (which could also match a launcher).
            executable = (
                (path / "cmdline")
                .read_bytes()
                .split(b"\0")[0]
                .decode(errors="replace")
                .replace("\\", "/")
                .rsplit("/", 1)[-1]
                .lower()
            )
            role = {
                "shippingpc-bmgame.exe": "game",
                "zluda64_server.exe": "server",
            }.get(executable)
            if role:
                if role in processes:
                    raise RuntimeError(
                        f"More than one {role} process; close duplicate sessions"
                    )
                processes[role] = path
        except (FileNotFoundError, PermissionError):
            continue
    if set(processes) != {"game", "server"}:
        raise RuntimeError(
            "Expected one running Batman process and one Windows ZLUDA server"
        )
    metadata = {role: inspect_process(path) for role, path in processes.items()}
    sensor_paths = list(Path("/sys/class/drm").glob("card*/device/gpu_busy_percent"))
    for pattern in ("temp*_input", "power*_input", "freq*_input"):
        sensor_paths.extend(
            Path("/sys/class/drm").glob("card*/device/hwmon/hwmon*/" + pattern)
        )
    labels = {}
    for path in sensor_paths:
        label = path.with_name(path.name.replace("_input", "_label"))
        if path.name.endswith("_input") and label.is_file():
            labels[str(path)] = label.read_text().strip()
    samples = []
    previous = {role: process_ticks(path) for role, path in processes.items()}
    previous_threads = {
        int(p.name): process_ticks(p) for p in (processes["server"] / "task").iterdir()
    }
    hz = os.sysconf("SC_CLK_TCK")
    started_wall = time.time()
    stopped_reason = None
    start = last = time.monotonic()
    for index in range(args.seconds):
        time.sleep(max(0, start + index + 1 - time.monotonic()))
        now = time.monotonic()
        elapsed = now - last
        try:
            cpu = {}
            for role, path in processes.items():
                ticks = process_ticks(path)
                cpu[role] = (ticks - previous[role]) / hz / elapsed * 100
                previous[role] = ticks
            threads = {}
            for path in (processes["server"] / "task").iterdir():
                try:
                    threads[int(path.name)] = process_ticks(path)
                except FileNotFoundError:
                    pass
        except (FileNotFoundError, ProcessLookupError):
            stopped_reason = "Game or server exited during capture"
            break
        thread_cpu = {
            tid: (ticks - previous_threads[tid]) / hz / elapsed * 100
            for tid, ticks in threads.items()
            if tid in previous_threads
        }
        previous_threads = threads
        sensors = {}
        for path in sensor_paths:
            try:
                sensors[str(path)] = int(path.read_text())
            except (OSError, ValueError):
                pass
        samples.append(
            {
                "seconds": now - start,
                "cpu_percent": cpu,
                "server_thread_cpu_percent": thread_cpu,
                "sensors_raw": sensors,
            }
        )
        last = now
    means = {
        role: (
            statistics.mean(sample["cpu_percent"][role] for sample in samples)
            if samples
            else None
        )
        for role in processes
    }
    print(
        json.dumps(
            {
                "started_utc": time.strftime(
                    "%Y-%m-%dT%H:%M:%SZ", time.gmtime(started_wall)
                ),
                "duration_seconds": last - start,
                "complete": len(samples) == args.seconds,
                "stopped_reason": stopped_reason,
                "processes": metadata,
                "sensor_labels": labels,
                "mean_cpu_percent": means,
                "samples": samples,
            },
            indent=2,
        )
    )


if __name__ == "__main__":
    main()
