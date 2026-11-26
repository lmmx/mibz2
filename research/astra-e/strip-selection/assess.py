from pathlib import Path
from collections import Counter,defaultdict
import csv,json,sys

mode=sys.argv[1]
path=Path(sys.argv[2])
rows=list(csv.DictReader(path.open()))
groups=defaultdict(Counter)
by_block=defaultdict(dict)
for r in rows:
    stream,block,arm=(int(r[k]) for k in ('stream','block','arm'))
    d={k:int(v) for k,v in r.items() if k not in ('stream','block','arm')}
    groups[stream,arm].update(d)
    by_block[block][arm]=d
streams=sorted({s for s,a in groups})
if mode=='train':
    assert streams==[0,3,6,9]
    candidates=[2,3,4]
else:
    assert streams==[1,2,4,5,7,8,10,11]
    candidates=[int(sys.argv[3])]
for bb in by_block.values():
    assert set(bb)=={0,1,*candidates}
    for a in [1,*candidates]:
        assert bb[a]['retained']==bb[0]['retained']+bb[a]['q'] and bb[a]['strips']==bb[0]['strips'] and bb[a]['q']==bb[1]['q']<=128
        assert bb[a]['edge_bytes']==bb[1]['edge_bytes']
def sumarm(a):
    total=Counter()
    for s in streams:total.update(groups[s,a])
    return total
b=sumarm(0);neutral=sumarm(1)
result={}
for a in candidates:
    x=sumarm(a)
    stream_ratios={s:groups[s,a]['steps']/groups[s,1]['steps'] for s in streams}
    result[a]={'steps_vs_neutral':x['steps']/neutral['steps'],'bytes_vs_neutral':x['dictionary_bytes']/neutral['dictionary_bytes'],'stream_ratio_steps':stream_ratios,
               'geometry_vs_champion':{k:x[k]/b[k] for k in ['nodes','steps','rules','bands','dictionary_bytes','words','word_visits','retained','same_line','same_4k']},
               'geometry_vs_neutral':{k:x[k]/neutral[k] for k in ['nodes','steps','rules','bands','dictionary_bytes','words','word_visits','retained','same_line','same_4k']},
               'max_select_bytes':max(v[a]['selector_cap'] for v in by_block.values()),'blocks_moved_edge_capacity_up':sum(v[a]['edge_cap']>v[0]['edge_cap'] for v in by_block.values())}
    result[a]['qualifies']=(result[a]['steps_vs_neutral']<=.99 and result[a]['bytes_vs_neutral']<=1.01 and sum(v<1 for v in stream_ratios.values())>=3)
winner=min(candidates,key=lambda a:(result[a]['steps_vs_neutral'],a))
summary={'split':mode,'blocks':len(by_block),'streams':streams,'rows':b['n'],'reference':b,'neutral':neutral,'policies':result,'winner':winner,'qualifies':result[winner]['qualifies']}
print(json.dumps(summary,indent=2))
