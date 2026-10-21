from pathlib import Path
import json,re,csv,io,collections
x=json.loads(Path('/private/tmp/pool-combined-live.json').read_text());lines=x['pool'].splitlines();assert lines[-1]=='complete'
coverage=[l.split(',') for l in lines if l.startswith('coverage,')];lru={};names={};events=[]
for l in lines:
 if l.startswith('lru,'):
  _,t,cap,h,m,e=l.split(',');h,m,e=map(int,(h,m,e));lru[int(cap)]=dict(hits=h,misses=m,evictions=e,hit_percent=100*h/(h+m) if h+m else None)
 elif l.startswith('name,'):
  _,k,h,n=l.split(',');names[(k,int(h))]=bytes.fromhex(n).decode(errors='replace').rstrip('\0')
 elif l.startswith('event,'):
  match=re.fullmatch(r'event,(\d+),(\w+)(?:\((.*?)\))?,(\d+)',l);assert match,l
  t,kind,args,status=match.groups();events.append((int(t),kind,tuple(map(int,args.split(', '))) if args else (),int(status)))
c=coverage[-1];keys=['milliseconds','binds','unbinds','launches','unknown','errors','resets','distinct'];summary={k:int(v) for k,v in zip(keys,c[1:9])};summary['distinct_truncated']=c[9];summary['lru']=lru;summary['nonzero_offsets']=int([l for l in lines if l.startswith('offset_nonzero,')][-1].split(',')[1]);summary['ring_counts']=dict(collections.Counter(e[1] for e in events));summary['ring_seconds']=(events[-1][0]-events[0][0])/1e6;summary['texrefs_in_ring']=len({e[2][0] for e in events if e[1]=='Bind'});summary['names']={f'{k}:{h}':n for (k,h),n in names.items()}
assert all(v['hits']+v['misses']==summary['binds']-summary['unknown'] for v in lru.values())
start=int(next(l.split(',')[1] for l in lines if l.startswith('started_unix_ns,')));origin=int(re.search(r'unix_ns=(\d+)',x['service']).group(1));end=start+summary['milliseconds']*1000000
rows=list(csv.DictReader(io.StringIO('\n'.join(l for l in x['service'].splitlines() if not l.startswith('#')))))
selected=[r for r in rows if origin+int(r['window_start_ns'])>=start and origin+int(r['window_end_ns'])<=end];agg=collections.defaultdict(lambda:dict(calls=0,errors=0,total_ns=0))
for r in selected:
 for key in ('calls','errors','total_ns'):agg[r['request']][key]+=int(r[key])
summary['service_complete_windows']=dict(agg)
Path('/private/tmp/pool-combined-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='names'},indent=2))
print('SEQUENCE SAMPLE')
for t,k,args,status in events[:45]:
 print(t,k,args,status,names.get(('false' if k in ('Bind','Format','Flags') else 'true',args[0]),'') if args else '')
