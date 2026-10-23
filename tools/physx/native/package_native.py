#!/usr/bin/env python3
"""Package the full native Linux server and ZLUDA built from the same revision."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import struct


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', required=True, type=Path)
    parser.add_argument('--revision', required=True)
    args = parser.parse_args()
    if not re.fullmatch('[0-9a-f]{40}', args.revision):
        parser.error('Expected full Git revision')
    root = Path(__file__).resolve().parents[3]
    args.output.mkdir(parents=True, exist_ok=True)
    files = {}
    for name in ('zluda64_server', 'libnvcuda.so'):
        source = root/'target/release'/name
        data = source.read_bytes()
        if data[:6] != b'\x7fELF\x02\x01' or struct.unpack_from('<H', data, 18)[0] != 62:
            raise RuntimeError('Expected ELF Linux x86-64: ' + name)
        shutil.copy2(source, args.output/name)
        files[name] = hashlib.sha256(data).hexdigest()
    for name in ('native_runtime.py', 'rollback_native.py'):
        source = Path(__file__).with_name(name)
        shutil.copy2(source, args.output/name)
        files[name] = hashlib.sha256(source.read_bytes()).hexdigest()
    (args.output/'manifest.json').write_text(json.dumps({
        'schema': 1, 'configuration': 'native-linux-physx-v1', 'revision': args.revision,
        'files': files, 'hip_runtime': 'external, existing Proton-ZLUDA libraries',
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
