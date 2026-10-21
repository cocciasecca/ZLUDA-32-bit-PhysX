#!/usr/bin/env python3
"""Commit, push and build the PhysX experimental worktree. Never deploys."""
import argparse
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

BRANCH = 'experiment/physx-diagnostics'
REPO = 'cocciasecca/ZLUDA-32-bit-PhysX'
WORKFLOW = 'push_master.yml'


def run(*args, cwd=None, capture=True):
    return subprocess.run(args, cwd=cwd, check=True, text=True,
                          stdout=subprocess.PIPE if capture else None).stdout


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('-m', '--message', default='Update PhysX diagnostic experiment')
    parser.add_argument('--yes', action='store_true', help='Commit all non-ignored changes without confirmation')
    parser.add_argument('--no-wait', action='store_true', help='Exit after starting the build')
    args = parser.parse_args()
    for tool in ('git', 'gh'):
        if not shutil.which(tool):
            raise RuntimeError(f'Manca {tool}: installalo prima di eseguire lo script.')
    here = Path(__file__).resolve().parent
    listing = run('git', 'worktree', 'list', '--porcelain', cwd=here)
    matches = []
    for block in listing.strip().split('\n\n'):
        lines = block.splitlines()
        if f'branch refs/heads/{BRANCH}' in lines:
            matches.append(Path(lines[0].removeprefix('worktree ')))
    if len(matches) != 1:
        raise RuntimeError(f'Serve un worktree esistente sul ramo {BRANCH}; nessun cambio di ramo automatico.')
    root = matches[0]
    if run('git', 'branch', '--show-current', cwd=root).strip() != BRANCH:
        raise RuntimeError('Il ramo del worktree è cambiato.')
    origin = run('git', 'remote', 'get-url', '--push', 'origin', cwd=root).strip()
    if origin not in (f'https://github.com/{REPO}.git', f'https://github.com/{REPO}',
                       f'git@github.com:{REPO}.git', f'ssh://git@github.com/{REPO}.git'):
        raise RuntimeError(f'Origin inatteso: {origin}')
    run('gh', 'auth', 'status', capture=False)
    status = run('git', 'status', '--short', cwd=root)
    if status:
        print(f'\nWorktree: {root}\nModifiche da includere:\n{status}', flush=True)
        run('git', 'diff', '--stat', 'HEAD', cwd=root, capture=False)
        if not args.yes and input('Includere TUTTE queste modifiche nel commit e avviare la build? [s/N] ').strip().lower() not in ('s', 'si', 'sì', 'y', 'yes'):
            print('Annullato: nessun commit o push eseguito.')
            return
        run('git', 'add', '--all', cwd=root)
        run('git', 'diff', '--cached', '--check', cwd=root, capture=False)
        run('git', 'commit', '-m', args.message, cwd=root, capture=False)
    sha = run('git', 'rev-parse', 'HEAD', cwd=root).strip()
    run('git', 'push', 'origin', f'HEAD:refs/heads/{BRANCH}', cwd=root, capture=False)
    # Explicit repository prevents dispatch to the upstream fork parent.
    output = run('gh', 'workflow', 'run', WORKFLOW, '--repo', REPO, '--ref', BRANCH, cwd=root)
    print(output, end='', flush=True)
    match = re.search(r'https://github\.com/' + re.escape(REPO) + r'/actions/runs/(\d+)', output)
    if not match:
        raise RuntimeError('Build avviata, ma gh non ha restituito il link. Controlla GitHub Actions; non rilanciare per errore una seconda build.')
    run_id = match.group(1)
    info = json.loads(run('gh', 'run', 'view', run_id, '--repo', REPO, '--json', 'headSha,headBranch'))
    if info['headSha'] != sha or info['headBranch'] != BRANCH:
        raise RuntimeError('La build non corrisponde al commit inviato: controlla il link prima di usare i binari.')
    print(f'Commit: {sha}\nNessuna installazione sulla Legion.', flush=True)
    if not args.no_wait:
        run('gh', 'run', 'watch', run_id, '--repo', REPO, '--exit-status', cwd=root, capture=False)
        print('Build riuscita. I binari sono negli artifact del link sopra.')


if __name__ == '__main__':
    try:
        main()
    except (RuntimeError, subprocess.CalledProcessError, EOFError) as error:
        print(f'\nInterrotto: {error}', file=sys.stderr)
        sys.exit(1)
    except KeyboardInterrupt:
        print('\nMonitoraggio interrotto; una build già avviata continua su GitHub.', file=sys.stderr)
        sys.exit(130)
