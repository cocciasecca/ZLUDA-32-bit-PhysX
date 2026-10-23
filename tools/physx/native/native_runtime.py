#!/usr/bin/env python3
"""Lifecycle of the native PhysX backend; used by Proton wrapper and installer preflight."""
import os
from pathlib import Path
import queue
import re
import secrets
import signal
import subprocess
import sys
import threading
import uuid


def environment(root, native):
    env = dict(os.environ)
    settings = root/'user_settings.py'
    if settings.exists():
        namespace = {'__file__': str(settings)}
        exec(compile(settings.read_text(), str(settings), 'exec'), namespace)
        for name, value in namespace.get('user_settings', {}).items():
            if name.startswith(('HSA_', 'HIP_', 'ROCR_', 'AMD_', 'ZLUDA_')):
                env[name] = str(value)
    libraries = [str(native), str(root/'zluda-libs')]
    if env.get('LD_LIBRARY_PATH'):
        libraries.append(env['LD_LIBRARY_PATH'])
    env['LD_LIBRARY_PATH'] = ':'.join(libraries)
    env['ZLUDA_NATIVE_LIBRARY'] = str(native/'libnvcuda.so')
    env['ZLUDA_PHYSX_TEXTURE_REUSE'] = '1'
    env['ZLUDA_PHYSX_TEXTURE_REUSE_TRANSFERS'] = '1'
    env['ZLUDA_PHYSX_RANGE_CACHE'] = '1'
    return env


class ServerSession:
    def __init__(self, root, native):
        self.root, self.native = Path(root), Path(native)
        self.process = None
        self.log = None

    def __enter__(self):
        self.env = environment(self.root, self.native)
        self.env['ZLUDA_NATIVE_TOKEN'] = secrets.token_hex(32)
        logs = self.root/'diagnostics'
        logs.mkdir(exist_ok=True)
        identifier = uuid.uuid4().hex
        self.env['ZLUDA_PHYSX_DIAGNOSTICS'] = str(logs/f'native-{identifier}.csv')
        self.log = (logs/f'native-{identifier}.log').open('w')
        try:
            self.process = subprocess.Popen([str(self.native/'zluda64_server'), '--listen', '127.0.0.1:0'],
                env=self.env, stdout=subprocess.PIPE, stderr=self.log, text=True, start_new_session=True)
            startup = queue.Queue()
            threading.Thread(target=lambda: startup.put(self.process.stdout.readline()), daemon=True).start()
            try:
                address = startup.get(timeout=30).strip()
            except queue.Empty:
                raise RuntimeError('Il server Linux non ha completato l’avvio entro 30 secondi.')
            if not re.fullmatch(r'127\.0\.0\.1:[0-9]+', address):
                raise RuntimeError('Avvio server Linux fallito; vedere diagnostics/native-*.log.')
            self.env['ZLUDA_NATIVE_ADDR'] = address
            self.env['ZLUDA_NATIVE_BACKEND'] = '1'
            return self
        except BaseException:
            self.close()
            raise

    def close(self):
        if self.process is not None:
            # Also kill per-client workers; they inherit this dedicated process group.
            try:
                os.killpg(self.process.pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(self.process.pid, signal.SIGKILL)
                self.process.wait()
            if self.process.stdout:
                self.process.stdout.close()
        if self.log:
            self.log.close()

    def __exit__(self, *args):
        self.close()


def base_proton(root):
    path = root/'proton.base'
    return path if path.exists() else root/'proton'


def preflight(root, native, payload32):
    """Use a separate compatdata directory; never touch Batman's prefix or saves."""
    root, native, payload32 = Path(root), Path(native), Path(payload32)
    env = environment(root, native)
    for binary in (native/'zluda64_server', native/'libnvcuda.so'):
        result = subprocess.run(['ldd', str(binary)], env=env, text=True, capture_output=True, check=True)
        if 'not found' in result.stdout:
            raise RuntimeError('Dipendenze Linux mancanti:\n' + result.stdout)
    with ServerSession(root, native) as server:
        env = dict(server.env)
        env['STEAM_COMPAT_DATA_PATH'] = str(root/'native-smoke-compatdata')
        env['STEAM_COMPAT_CLIENT_INSTALL_PATH'] = str(root.parents[1])
        env['SteamAppId'] = env['SteamGameId'] = env['STEAM_COMPAT_APP_ID'] = '0'
        env['ZLUDA_NATIVE_DLL'] = 'Z:' + str(payload32/'nvcuda.dll').replace('/', '\\')
        env['ZLUDA_PROBE_GPU'] = '1'
        env['WINEDLLOVERRIDES'] = 'nvcuda=n'
        log_path = root/'diagnostics/native-preflight.log'
        with log_path.open('w') as log:
            result = subprocess.run([sys.executable, str(base_proton(root)), 'run',
                str(payload32/'native-dll-probe.exe'), 'probe', env['ZLUDA_NATIVE_ADDR']],
                env=env, cwd=root, stdout=log, stderr=subprocess.STDOUT, timeout=300)
        if result.returncode != 0:
            tail = log_path.read_text(errors='replace')[-6000:]
            raise RuntimeError('Prova DLL Windows → ZLUDA Linux fallita; installazione non attivata.\n' + tail)


def run_proton(root, args):
    root = Path(root)
    base = root/'proton.base'
    if not base.is_file():
        raise RuntimeError('proton.base mancante: ripristinare il backup dell’installazione.')
    command = [sys.executable, str(base), *args]
    if not args or args[0] not in ('run', 'waitforexitandrun'):
        return subprocess.call(command)
    with ServerSession(root, root/'native') as server:
        # Keep Proton's own environment; only inject connection information. The
        # native process gets Linux library paths and Linux diagnostic filenames.
        env = dict(os.environ)
        for name in ('ZLUDA_NATIVE_ADDR', 'ZLUDA_NATIVE_TOKEN', 'ZLUDA_NATIVE_BACKEND'):
            env[name] = server.env[name]
        process = subprocess.Popen(command, env=env)
        previous = signal.signal(signal.SIGTERM, lambda *_: (_ for _ in ()).throw(SystemExit(143)))
        try:
            return process.wait()
        finally:
            signal.signal(signal.SIGTERM, previous)
            if process.poll() is None:
                process.terminate()
                try:
                    process.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    process.kill()
                    process.wait()
