exec(open('/private/tmp/analyze-pool-combined.py').read().split("Path('/private/tmp/pool-combined-summary.json')")[0])
config={};bound={};changes=collections.Counter();known=collections.Counter();orders=collections.Counter();last={};sizes=collections.Counter()
for t,k,args,status in events:
 if status or k=='Reset':config.clear();bound.clear();continue
 if k in ('Flags','Format'):
  h=args[0];v=args[1:];old=config.setdefault(h,{}).get(k)
  if bound.get(h):
   known[k+'_while_bound']+=1
   if old is not None and old!=v:changes[k]+=1
  config[h][k]=v
  last[h]=k
 elif k=='Bind':
  h,p,b=args
  if p:
   orders[last.get(h,'unknown')]+=1;sizes[b]+=1
  bound[h]=bool(p);last[h]='Bind' if p else 'Unbind'
print(json.dumps(dict(setters_while_bound=known,known_changed_while_bound=changes,immediate_pre_bind=orders,sizes=sizes),indent=2))
