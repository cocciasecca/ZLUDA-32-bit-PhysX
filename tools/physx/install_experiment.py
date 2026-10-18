from pathlib import Path
import hashlib, json, os, shutil, subprocess, tempfile, time
home=Path('/home/deck')
parent=home/'.local/share/Steam/compatibilitytools.d'
source=parent/'ZludaProton'
dest=parent/'ZludaProton-Experiment'
payload=Path(__file__).resolve().parent/'payload'
def sha(p):
    h=hashlib.sha256()
    with p.open('rb') as f:
        for data in iter(lambda:f.read(1024*1024),b''):h.update(data)
    return h.hexdigest()
def snapshot(root):
    result={}
    for p in sorted(root.rglob('*')):
        rel=str(p.relative_to(root))
        if p.is_symlink():result[rel]={'symlink':os.readlink(p)}
        elif p.is_file():result[rel]={'sha256':sha(p)}
    return result
assert source.is_dir() and not dest.exists() and not dest.is_symlink()
assert shutil.disk_usage(parent).free > 4*1024**3
for line in (payload/'SHA256SUMS.txt').read_text(encoding='utf-8-sig').splitlines():
    expected,name=line.split(None,1)
    p=payload/name.strip().replace('\\','/')
    assert sha(p)==expected,(p,'checksum mismatch')
assert sha(source/'zluda-libs/libamd_comgr.so.3')=='e765f0913bc94de361b8e471e22c5085b2e6d967510cb80c2084b031e3f4781c'
print('Recording original tool manifest',flush=True)
before=snapshot(source)
stage=Path(tempfile.mkdtemp(prefix='.ZludaProton-Experiment-stage-',dir=parent))
print('Copying tool to',stage,flush=True)
subprocess.run(['cp','-a','--reflink=auto',str(source)+'/.',str(stage)],check=True)
assert snapshot(stage)==before,'Copied tool differs before patching'
def safe(rel):
    p=stage/rel
    assert p.resolve().is_relative_to(stage.resolve()),('External symlink',p)
    return p
for src,rel in [('diagnostic/zluda64_server.exe','zluda/zluda64_server.exe'),('32/nvcuda.dll','zluda/32/nvcuda.dll'),('32/nvcuda.dll','files/share/default_pfx/drive_c/windows/syswow64/nvcuda.dll')]:
    p=safe(rel)
    if p.exists():p.unlink()
    shutil.copy2(payload/src,p)
control=safe('diagnostic-control');control.mkdir()
shutil.copy2(payload/'control/zluda64_server.exe',control/'zluda64_server.exe')
for name in ['README.txt','SHA256SUMS.txt']:shutil.copy2(payload/name,control/name)
p=safe('compatibilitytool.vdf')
s=p.read_text();assert s.count('"ZludaProton"')==2
p.write_text(s.replace('"ZludaProton"','"ZludaProton-Experiment"'))
logs=safe('diagnostics');logs.mkdir()
p=safe('user_settings.py')
s=p.read_text();assert 'ZLUDA64_PATH' in s and 'ZLUDA_PHYSX_DIAGNOSTICS' not in s
s+='''
# Experimental diagnostics: a unique file for each Proton invocation.
import uuid
_diag_dir = os.path.join(os.path.dirname(os.path.abspath(__file__)), "diagnostics")
user_settings["ZLUDA_PHYSX_DIAGNOSTICS"] = "Z:" + os.path.join(_diag_dir, "requests-" + uuid.uuid4().hex + ".csv").replace("/", "\\\\")
'''
compile(s,str(p),'exec');p.write_text(s)
backup=safe('pre-experiment-prefix-backup');backup.mkdir()
prefix=home/'.local/share/Steam/steamapps/compatdata/35140/pfx'
backup_manifest={}
for rel in ['drive_c/windows/syswow64/nvcuda.dll','drive_c/windows/system32/nvcuda.dll','user.reg','system.reg','userdef.reg']:
    src=prefix/rel
    if src.is_file():
        dst=backup/rel;dst.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(src,dst)
        backup_manifest[rel]={'sha256':sha(src),'symlink':os.readlink(src) if src.is_symlink() else None}
(backup/'manifest.json').write_text(json.dumps(backup_manifest,indent=2)+'\n')
(safe('experiment-manifest.json')).write_text(json.dumps({'run':39160259995,'commit':'866951a53be80c5a8d4f605b9ec72db8f6839817','original_tool':str(source),'original_files':before,'payload':{str(p.relative_to(payload)):sha(p) for p in payload.rglob('*') if p.is_file()}},indent=2)+'\n')
safe('EXPERIMENT-README.txt').write_text('Diagnostic tool, not a performance fix. Original tool preserved.\nUse the experimental zluda-libs path in launch options.\nGame/Steam not started by installer. Existing prefix NOT modified.\nBefore/after launch verify syswow64/nvcuda.dll matches zluda/32/nvcuda.dll.\nRollback: select original ZludaProton and original launch options; verify original DLL restored.\nOriginal prefix DLLs and registry snapshots are in pre-experiment-prefix-backup. Do not reset prefix or overwrite saves.\nLogs: diagnostics/requests-*.csv; five-second aggregates, final partial window may be absent.\n')
assert snapshot(source)==before,'Original tool unexpectedly changed'
for rel,info in backup_manifest.items():assert sha(prefix/rel)==info['sha256'],'Prefix changed concurrently'
assert sha(stage/'zluda-libs/libamd_comgr.so.3')==sha(source/'zluda-libs/libamd_comgr.so.3')
assert not dest.exists()
stage.rename(dest)
print('INSTALLED',dest,flush=True)
print('Original tool and backed-up prefix files unchanged; COMGR preserved.',flush=True)
