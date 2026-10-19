from pathlib import Path
import hashlib,json,shutil,subprocess,os
root=Path('/home/deck/.local/share/Steam/compatibilitytools.d/ZludaProton-Experiment')
original=root.parent/'ZludaProton'
payload=Path(__file__).resolve().parent/'payload'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert subprocess.run(['pgrep','-f','[z]luda64_server|[S]hippingPC-BmGame'],capture_output=True).returncode==1,'Game/server still running'
manifest=json.loads((root/'experiment-manifest.json').read_text())
for rel,info in manifest['original_files'].items():
 p=original/rel
 if 'sha256' in info:assert sha(p)==info['sha256'],('Original changed',rel)
 else:assert os.readlink(p)==info['symlink']
for row in (payload/'SHA256SUMS.txt').read_text(encoding='utf-8-sig').splitlines():
 h,n=row.split(None,1);assert sha(payload/n.replace('\\','/'))==h
prefix=Path('/home/deck/.local/share/Steam/steamapps/compatdata/35140/pfx')
prefix_before={rel:sha(prefix/rel) for rel in ['drive_c/windows/syswow64/nvcuda.dll','drive_c/windows/system32/nvcuda.dll']}
backup=root/'update-backup-39219218455'
backup.mkdir()
pairs=[('reuse/zluda64_server.exe','zluda/zluda64_server.exe'),('32/nvcuda.dll','zluda/32/nvcuda.dll'),('32/nvcuda.dll','files/share/default_pfx/drive_c/windows/syswow64/nvcuda.dll'),('control/zluda64_server.exe','diagnostic-control/zluda64_server.exe'),('README.txt','diagnostic-control/README.txt'),('SHA256SUMS.txt','diagnostic-control/SHA256SUMS.txt')]
staged=[]
for src,rel in pairs:
 target=root/rel
 assert target.resolve().is_relative_to(root.resolve())
 saved=backup/rel;saved.parent.mkdir(parents=True,exist_ok=True);shutil.copy2(target,saved)
 temp=target.with_name(target.name+'.pending-39219218455')
 assert not temp.exists()
 shutil.copy2(payload/src,temp)
 assert sha(temp)==sha(payload/src)
 staged.append((temp,target,saved))
settings=root/'user_settings.py'
saved_settings=backup/'user_settings.py'
shutil.copy2(settings,saved_settings)
new_settings=settings.read_text()+'\n# Opt-in texture reuse trial, run 39219218455.\nuser_settings["ZLUDA_PHYSX_TEXTURE_REUSE"] = "1"\n'
compile(new_settings,str(settings),'exec')
temp_settings=settings.with_name('user_settings.py.pending-39219218455')
temp_settings.write_text(new_settings)
staged.append((temp_settings,settings,saved_settings))
assert subprocess.run(['pgrep','-f','[z]luda64_server|[S]hippingPC-BmGame'],capture_output=True).returncode==1,'Game started during staging'
updated=[]
try:
 for temp,target,saved in staged:
  os.replace(temp,target);updated.append((target,saved))
 for src,rel in pairs:assert sha(root/rel)==sha(payload/src)
except Exception:
 for target,saved in reversed(updated):shutil.copy2(saved,target)
 raise
assert all(sha(prefix/rel)==h for rel,h in prefix_before.items())
assert sha(root/'zluda-libs/libamd_comgr.so.3')==sha(original/'zluda-libs/libamd_comgr.so.3')
record={'run':39219218455,'commit':'af21634e4379681fb26567fad458b15c304a47d8','files':{rel:sha(root/rel) for _,rel in pairs},'prefix_unchanged':prefix_before,'previous_payload_backup':str(backup),'csv_suffix':'.pid-N.csv','texture_reuse_enabled':True,'settings_sha256':sha(settings)}
(root/'update-manifest-39219218455.json').write_text(json.dumps(record,indent=2)+'\n')
print(json.dumps(record,indent=2))
