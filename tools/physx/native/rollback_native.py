#!/usr/bin/env python3
"""Restore the exact preceding installation, only with game/server closed."""
import hashlib
import json
from pathlib import Path
import os
import shutil
import subprocess


def main():
    root = Path(__file__).resolve().parents[1]
    if subprocess.run(['pgrep', '-f', '[z]luda64_server|[S]hippingPC-BmGame'], capture_output=True).returncode != 1:
        raise RuntimeError('Chiudi Batman prima del ripristino.')
    manifest = root/'native-install-last.json'
    record = json.loads(manifest.read_text())
    backup = Path(record['backup'])
    if backup.parent.resolve() != root.resolve():
        raise RuntimeError('Percorso backup inatteso')
    digest = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
    for name, expected in record['files'].items():
        if not (root/name).resolve().is_relative_to(root.resolve()) or digest(root/name) != expected:
            raise RuntimeError('File modificato dopo installazione; ripristino fermato: '+name)
    for name, expected in record['old_files'].items():
        if not (root/name).resolve().is_relative_to(root.resolve()):
            raise RuntimeError('Percorso non valido')
        if expected is not None and digest(backup/name) != expected:
            raise RuntimeError('Backup non valido: '+name)
    for name, expected in reversed(list(record['old_files'].items())):
        if expected is None:
            (root/name).unlink(missing_ok=True)
        else:
            shutil.copy2(backup/name, root/name)
    shutil.rmtree(root/'native')
    if (backup/'native').exists():
        os.replace(backup/'native', root/'native')
    if (backup/'native-install-last.json').exists():
        shutil.copy2(backup/'native-install-last.json', manifest)
    else:
        manifest.unlink()
    print('Configurazione precedente ripristinata. Prefix e salvataggi invariati.')


if __name__ == '__main__':
    main()
