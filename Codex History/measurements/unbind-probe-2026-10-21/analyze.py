import json,csv,io,collections
from pathlib import Path
x=json.loads(Path('/private/tmp/unbind-probe-live.json').read_text())
def rows(text): return list(csv.DictReader(io.StringIO('\n'.join(l for l in text.splitlines() if not l.startswith('#')))))
b=rows(x['server_before']);a=rows(x['server_after']);cut=max(int(r['window_end_ns']) for r in b)
w=sorted(set((int(r['window_start_ns']),int(r['window_end_ns'])) for r in a if int(r['window_start_ns'])>=cut))[1:]
selected=[r for r in a if (int(r['window_start_ns']),int(r['window_end_ns'])) in w];agg=collections.defaultdict(lambda:dict(calls=0,errors=0,total_ns=0))
for r in selected:
 for k in ('calls','errors','total_ns'):agg[r['request']][k]+=int(r[k])
service=sum(v['total_ns'] for k,v in agg.items() if not k.startswith(('shadow/', 'probe/')))
result=dict(seconds=sum(e-s for s,e in w)/1e9,windows=len(w),service_seconds=service/1e9,user_fps='26–27',rows=dict(sorted(agg.items(),key=lambda kv:kv[1]['total_ns'],reverse=True)))
Path('/private/tmp/unbind-probe-summary.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
