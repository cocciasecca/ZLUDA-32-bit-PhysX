#!/usr/bin/env python3
"""Verify, GPU-preflight and transactionally install the current native PhysX package."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import uuid

ROOT = Path('/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton-Experiment')
CONFIGURATION = 'native-linux-physx-v1'
LAUNCHER = '''#!/usr/bin/env python3
# ZLUDA_NATIVE_LAUNCHER_V1
import sys
from pathlib import Path
root = Path(__file__).resolve().parent
sys.path.insert(0, str(root/'native'))
from native_runtime import run_proton
sys.exit(run_proton(root, sys.argv[1:]))
'''


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def closed():
    result = subprocess.run(['pgrep', '-f', '[z]luda64_server|[S]hippingPC-BmGame'], capture_output=True)
    if result.returncode != 1:
        raise RuntimeError('Chiudi Batman prima di installare (oppure server ancora attivo).')


def restore_files(root, backup, records):
    for relative, old_hash in reversed(list(records.items())):
        target = root/relative
        if old_hash is None:
            target.unlink(missing_ok=True)
        else:
            shutil.copy2(backup/relative, target)


def main():
    run_id, revision = sys.argv[1:3]
    if not run_id.isdecimal() or len(revision) != 40 or any(c not in '0123456789abcdef' for c in revision):
        raise RuntimeError('Invalid build identity')
    if len(sys.argv) > 3 and sys.argv[3] != 'native-linux':
        raise RuntimeError('This installer only accepts the current native Linux configuration')
    root = ROOT
    original = root.parent/'ZludaProton'
    payload = Path(__file__).resolve().parent/'payload'
    closed()
    for relative, info in json.loads((root/'experiment-manifest.json').read_text())['original_files'].items():
        if 'sha256' in info:
            assert sha(original/relative) == info['sha256'], ('Original changed', relative)
        else:
            assert os.readlink(original/relative) == info['symlink']
    expected = {'32/nvcuda.dll', '32/native-dll-probe.exe', 'control/zluda64_server.exe', 'diagnostic/zluda64_server.exe'}
    seen = set()
    for line in (payload/'SHA256SUMS.txt').read_text(encoding='utf-8-sig').splitlines():
        digest, relative = line.split(None, 1)
        relative = relative.replace('\\', '/')
        assert relative in expected and relative not in seen, 'Unexpected binary manifest'
        assert sha(payload/relative) == digest, relative
        seen.add(relative)
    assert seen == expected, 'Incomplete Windows payload'
    native_manifest = json.loads((payload/'native/manifest.json').read_text())
    assert native_manifest['schema'] == 1 and native_manifest['revision'] == revision
    assert native_manifest['configuration'] == CONFIGURATION
    assert set(native_manifest['files']) == {'zluda64_server', 'libnvcuda.so', 'native_runtime.py', 'rollback_native.py'}
    for relative, digest in native_manifest['files'].items():
        assert sha(payload/'native'/relative) == digest, relative
    assert 'Configuration: '+CONFIGURATION in (payload/'README.txt').read_text(encoding='utf-8-sig')
    prefix = root.parents[1]/'steamapps/compatdata/35140/pfx'
    prefix_paths = ['drive_c/windows/syswow64/nvcuda.dll', 'drive_c/windows/system32/nvcuda.dll']
    prefix_before = {name: sha(prefix/name) for name in prefix_paths}
    comgr_before = sha(root/'zluda-libs/libamd_comgr.so.3')
    last_path = root/'native-install-last.json'
    if last_path.exists():
        previous = json.loads(last_path.read_text())
        if previous['commit'] == revision and all((root/name).is_file() and sha(root/name) == value for name, value in previous['files'].items()):
            print('Già installato e verificato.'); return
    if '# ZLUDA_NATIVE_LAUNCHER_V1' in (root/'proton').read_text() and not (root/'proton.base').is_file():
        raise RuntimeError('Wrapper presente senza proton.base; ripristinare il backup prima di proseguire.')
    with tempfile.TemporaryDirectory(prefix='.native-stage-', dir=root) as temporary:
        stage = Path(temporary)
        (stage/'native').mkdir()
        for name in [*native_manifest['files'], 'manifest.json']:
            shutil.copy2(payload/'native'/name, stage/'native'/name)
        (stage/'native/zluda64_server').chmod(0o755)
        spec = importlib.util.spec_from_file_location('native_runtime_preflight', stage/'native/native_runtime.py')
        runtime = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(runtime)
        print('Verifica DLL a 32 bit, copie e kernel PTX su ZLUDA Linux; prefix di prova separato.', flush=True)
        runtime.preflight(root, stage/'native', payload/'32')
        closed()
        backup = root/f'update-backup-{run_id}-native-{uuid.uuid4().hex[:8]}'
        backup.mkdir()
        old = {}
        replacements = {
            'zluda/32/nvcuda.dll': payload/'32/nvcuda.dll',
            'files/share/default_pfx/drive_c/windows/syswow64/nvcuda.dll': payload/'32/nvcuda.dll',
            'zluda/zluda64_server.exe': payload/'diagnostic/zluda64_server.exe',
        }
        if not (root/'proton.base').exists():
            shutil.copy2(root/'proton', stage/'proton.base')
            replacements['proton.base'] = stage/'proton.base'
        settings = (root/'user_settings.py').read_text() + '''
# Current native Linux PhysX backend. Connection credentials are per-launch only.
user_settings["ZLUDA_NATIVE_BACKEND"] = "1"
user_settings["ZLUDA_PHYSX_TEXTURE_REUSE"] = "1"
user_settings["ZLUDA_PHYSX_TEXTURE_REUSE_TRANSFERS"] = "1"
user_settings["ZLUDA_PHYSX_RANGE_CACHE"] = "1"
user_settings["ZLUDA_PHYSX_TEXTURE_POOL"] = "0"
user_settings["ZLUDA_PHYSX_POOL_CAPTURE"] = "0"
user_settings["ZLUDA_PHYSX_TEXTURE_GLOBAL_PROBE"] = "0"
user_settings["ZLUDA_PHYSX_SPLIT_UNBIND"] = "0"
'''
        compile(settings, 'user_settings.py', 'exec')
        (stage/'user_settings.py').write_text(settings)
        replacements['user_settings.py'] = stage/'user_settings.py'
        (stage/'proton').write_text(LAUNCHER)
        (stage/'proton').chmod((root/'proton').stat().st_mode & 0o777)
        replacements['proton'] = stage/'proton'  # Activate only after native files and DLLs.
        for relative in replacements:
            target = root/relative
            assert target.resolve().is_relative_to(root.resolve()), relative
            old[relative] = sha(target) if target.exists() else None
            if target.exists():
                (backup/relative).parent.mkdir(parents=True, exist_ok=True)
                shutil.copy2(target, backup/relative)
        if last_path.exists():
            shutil.copy2(last_path, backup/'native-install-last.json')
        record = {'run': int(run_id), 'commit': revision, 'configuration': CONFIGURATION,
            'backup': str(backup), 'old_files': old, 'prefix_unchanged': prefix_before,
            'files': {name: sha(source) for name, source in replacements.items()}}
        for name, digest in native_manifest['files'].items():
            record['files']['native/'+name] = digest
        (backup/'transaction.json').write_text(json.dumps(record, indent=2))
        closed()
        for relative, digest in old.items():
            assert (sha(root/relative) if (root/relative).exists() else None) == digest, 'File changed during staging'
        moved_native = False
        activated_native = False
        try:
            if (root/'native').exists():
                assert not (root/'native').is_symlink()
                os.replace(root/'native', backup/'native')
                moved_native = True
            os.replace(stage/'native', root/'native')
            activated_native = True
            for relative, source in replacements.items():
                temporary_file = (root/relative).with_name(Path(relative).name+'.native-pending')
                shutil.copy2(source, temporary_file)
                os.replace(temporary_file, root/relative)
            assert all(sha(root/name) == value for name, value in record['files'].items())
            assert all(sha(prefix/name) == value for name, value in prefix_before.items())
            assert sha(root/'zluda-libs/libamd_comgr.so.3') == comgr_before
            last_temp = root/'native-install-last.json.pending'
            last_temp.write_text(json.dumps(record, indent=2)+'\n')
            os.replace(last_temp, last_path)
        except BaseException:
            restore_files(root, backup, old)
            if activated_native:
                shutil.rmtree(root/'native')
            if moved_native:
                os.replace(backup/'native', root/'native')
            raise
        print('Backend Linux installato; kernel PTX verificato prima dell’attivazione.')
        print('Ripristino automatico: python3 "'+str(root/'native/rollback_native.py')+'"')


if __name__ == '__main__':
    main()
