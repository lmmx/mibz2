"""Aggregate independent per-block bounds without assigning predictive power to entropy."""
import csv
from pathlib import Path
from collections import defaultdict,Counter
import json
here=Path(__file__).resolve().parent
def rows(name):
    with (here/name).open() as f:
        yield from csv.DictReader(f)
mag=defaultdict(Counter)
for x in rows('magnitude.csv'):
    v={k:int(y) for k,y in x.items()}
    mag[v['width']].update(v)
mvalues={}
for w,v in mag.items():
    mvalues[w]={k:v[k]/v['m'] for k in ['plain_centered','optimal_one_base','optimal_per_terminal','optimal_per_label','equal_adjacent_delta','increasing_adjacent_next']}
ent=list(rows('entropy.csv'));mass=sum(int(x['m']) for x in ent)
entropies={k:sum(int(x['m'])*float(x[k]) for x in ent)/mass for k in ['H_label','H_delta','H_delta_given_terminal','H_delta_given_label','H_joint']}
allpub={}
for fname in ['palette.csv','palette-packed.csv']:
    blocks=defaultdict(list);g=defaultdict(Counter)
    for x in rows(fname):
        k=(x.get('variant','index15-fixed8'),int(x['bin_bits']))
        m=int(x['m']);size=int(x['total']);b=int(x['block'])
        blocks[b].append((size,m))
        g[k].update(bytes=size,rows=m,escape=int(x['escaped'] if 'escaped' in x else x['escape']))
    chosen=[min(z) for z in blocks.values()]
    vals={str(k):{'bytes_per_row':c['bytes']/c['rows'],'escape_fraction':c['escape']/c['rows']} for k,c in g.items()}
    allpub[fname]={'fixed':vals,'oracle_per_block':sum(t for t,m in chosen)/mass,
                   'under_4':sum(t<4*m for t,m in chosen),'under_3':sum(t<3*m for t,m in chosen),
                   'minimum_block_bytes_per_row':min(t/m for t,m in chosen)}
print(json.dumps({'rows':mass,'magnitude':mvalues,'zero_order_bits_per_row':entropies,'palette':allpub},indent=2))
