"""Replay archived elapsed/CPU measurements; never run a decoder or timer."""
import json,math,statistics as s
from pathlib import Path
from team_size import T975
rows=[json.loads(x) for x in Path('rfl-raw/rfl-runs.jsonl').read_text().splitlines()]
def name(a):return a.removeprefix('mibz2-pipe3-runfed-').removeprefix('mibz2-')
by={(x['threads'],name(x['arm']),x['round']):x for x in rows}
def ci(x,log=False):
 if log:x=list(map(math.log,x))
 m=s.mean(x);h=T975[len(x)-2]*s.stdev(x)/len(x)**.5
 return [math.exp(y) for y in (m,m-h,m+h)] if log else [m,m-h,m+h]
out={'contrasts':{},'phases':{},'phase_differences':{},'in_run_rule_counts':{},'wall_cpu':{}}
for t in (9,3):
 pairs=[('reg','ctl'),('dir','reg'),('ls','reg'),('ls','dir'),('ls','hs'),('dir','hs')]
 if t==9:pairs += [('reg','default33'),('dir','default33'),('ls','default33')]
 for a,b in pairs:
  out['contrasts'][f'{t}:{a}/{b}']={k:ci([by[t,a,i][k]/by[t,b,i][k] for i in range(15)],True) for k in ('wall','cpu')}
 p={a:[dict((k,int(v)) for k,v in (x.split('=') for x in by[t,a,i]['phases'].split())) for i in range(15)] for a in ('ctl','reg','ls','hs','dir')}
 keys=['lead_ns','lead_front_ns','lead_sort_ns','lead_rle_ns','lead_small_ns','lead_start_ns','lead_end_ns','build_end_ns','join_wait_ns','run_ns','helper_sort_ns','helper_fill_ns','helper_fused_ns','drained_front_ns','leader_late']
 for a in p:
  out['phases'][f'{t}:{a}']={k:ci([x[k]/x['runs']/(1e6 if k.endswith('_ns') else 1) for x in p[a]]) for k in keys}
  out['phases'][f'{t}:{a}']['all_front_ms']=ci([(x['lead_front_ns']+x['drained_front_ns']+x['lead_sort_ns'])/1e6 for x in p[a]])
  out['in_run_rule_counts'][f'{t}:{a}']=[x['rule_blocks'] for x in p[a]]
  out['wall_cpu'][f'{t}:{a}']={k:ci([by[t,a,i][k] for i in range(15)]) for k in ('wall','cpu','runq_s','rss_kb')}
 for a,b in [('reg','ctl'),('dir','reg'),('ls','reg'),('ls','dir')]:
  out['phase_differences'][f'{t}:{a}-{b}']={k:ci([(p[a][i][k]/p[a][i]['runs']-p[b][i][k]/p[b][i]['runs'])/(1e6 if k.endswith('_ns') else 1) for i in range(15)]) for k in keys}
  if t==3:
   run=[(p[a][i]['run_ns']-p[b][i]['run_ns'])/1e6 for i in range(15)]
   drain=[(p[a][i]['drained_front_ns']-p[b][i]['drained_front_ns'])/1e6 for i in range(15)]
   wall=[(by[t,a,i]['wall']-by[t,b,i]['wall'])*1000 for i in range(15)]
   out.setdefault('single_team_ms_decomposition',{})[a+'-'+b]={'run':ci(run),'drained_front':ci(drain),'wall':ci(wall),'other_residual':ci([w-r-d for w,r,d in zip(wall,run,drain)])}
 # Exact algebra of geometric-mean point ratios, not causal mediation.
 regs=out['contrasts'][f'{t}:reg/ctl']['wall'][0]
 for a in ('dir','ls'):
  inc=out['contrasts'][f'{t}:{a}/reg']['wall'][0]
  out.setdefault('log_share',{})[f'{t}:{a}']=math.log(regs)/math.log(regs*inc)
Path('rfl-analysis.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out['contrasts'],indent=2))
print('LOG-SHARE (descriptive)',out['log_share'])
for t in (9,3):
 print('PHASES',t)
 for a in ('ctl','reg','ls','hs','dir'):
  print(a,{k:round(v[0],4) for k,v in out['phases'][f'{t}:{a}'].items()},'rule_count_range',min(out['in_run_rule_counts'][f'{t}:{a}']),max(out['in_run_rule_counts'][f'{t}:{a}']))
