"""Replay geometric hop ceilings and byte-exact half-reservation movements."""
from collections import Counter
import csv
import json
from pathlib import Path

D=Path(__file__).resolve().parent
def read(path):
    with path.open() as src:
        for r in csv.DictReader(src):
            yield {k:(v if k in ('relation','method','stop') else int(v)) for k,v in r.items()}
base=list(read(D/'panel.csv'))
bound={(r['block'],r['method']):r for r in read(D/'bounded.csv')}
for r in base:
    r.update(bound[r['block'],r['method']])
    assert r['primary_bytes']+r['secondary_bytes']==r['n'] and r['primary_hops']+r['secondary_hops']==r['m']
    assert r['paired_before_boundary']<=r['paired_rounds']<=r['ideal_paired_rounds']
extra={41,56,165,270,278}
def summarize(selected):
    s=Counter()
    for r in selected:s.update({k:v for k,v in r.items() if isinstance(v,int)})
    frac=lambda a,b:s[a]/s[b]
    order=sorted(selected,key=lambda r:r['paired_rounds']/r['ideal_paired_rounds'])
    q=lambda k:order[round(k*(len(order)-1))]
    return dict(blocks=len(order),n=s['n'],m=s['m'],unbounded_paired_hops=s['paired_rounds'],
                ideal_paired_hops=s['ideal_paired_rounds'],frac_of_ideal_unbounded=frac('paired_rounds','ideal_paired_rounds'),
                unbounded_critical_hop_fraction=frac('critical_rounds','m'),idealized_unit_latency_speed=s['m']/s['critical_rounds'],
                actual_bounded_paired_hops=s['paired_before_boundary'],
                frac_of_ideal_bounded=frac('paired_before_boundary','ideal_paired_rounds'),
                fraction_bounded_vs_unbounded=frac('paired_before_boundary','paired_rounds'),
                primary_words=s['primary_words'],secondary_words=s['secondary_words'],
                secondary_words_payload=s['secondary_spelling_bytes'],secondary_all_bytes=s['secondary_bytes'],
                secondary_byte_fraction=frac('secondary_bytes','n'),word_word_rounds=s['both_nonliteral_rounds'],
                paired_min_payload=s['paired_shorter_payload_sum'],
                modeled_same_edge_lines=s['same_edge_line_rounds'],modeled_same_edge_pages=s['same_edge_page_rounds'],
                unbounded_peak_scratch_if_separate_vector=max(r['persistent_secondary_capacity'] for r in selected),
                normal_carried=s['normal_carried'],normal_moved=s['normal_moved'],
                directed_carried=s['directed_carried'],directed_moved=s['directed_moved'],
                stop=dict(Counter(r['stop'] for r in selected)),seed_relation=dict(Counter(r['relation'] for r in selected)),
                quantiles_unbounded=[dict(p=p,block=q(p)['block'],frac=q(p)['paired_rounds']/q(p)['ideal_paired_rounds'])
                                     for p in (0,.1,.25,.5,.75,.9,1)])
out={}
for method in ('half_shift','xor1','zero','midpoint'):
    rows=[r for r in base if r['method']==method];assert len(rows)==233
    out[method]={k:summarize([r for r in rows if f(r)]) for k,f in [('all',lambda r:True),('guarded',lambda r:r['block'] in extra),('ordinary',lambda r:r['block'] not in extra)]}
halves={r['block']:r for r in base if r['method']=='half_shift'}
examples={1,*extra,*(q['block'] for q in out['half_shift']['all']['quantiles_unbounded'] if q['p'] in (0,.5,1))}
out['examples']={b:halves[b] for b in sorted(examples)}
# The independent native geometry run is generated and archived by the parent stage.
# Compare every item when present. Core summaries can be rebuilt solely from these tapes.
native=D.parent/'all.csv'
if native.exists():
    rows=list(csv.DictReader(native.open()));assert len(rows)==466
    for r in rows:
        v=halves[int(r['block'])];prefix='normal' if r['arm']=='0' else 'directed'
        assert int(r['paired'])==v['paired_before_boundary'] and int(r['carried'])==v[prefix+'_carried'] and int(r['moved'])==v[prefix+'_moved']
        assert (r['boundary']=='true')==(v['stop'] in ('primary_boundary','secondary_boundary'))
print(json.dumps(out,indent=2))
