from pathlib import Path
import csv,collections
here=Path(__file__).resolve().parent
gaps={int(r['block']):{k:int(v) for k,v in r.items()} for r in csv.DictReader((here/'gaps.csv').open())}
print('split blocks nodes gap_nodes observed_node_saving upper_gap_saving upper_all_fraction candidate_pairs same_candidate_pairs')
for split in ('train','held'):
    rows=collections.defaultdict(dict)
    for r in csv.DictReader((here/(split+'.csv')).open()):
        rows[int(r['block'])][int(r['arm'])]={k:int(v) for k,v in r.items()}
    n=g=actual=bound=pair=same=0
    for block,arms in rows.items():
        a=arms[0];w=arms[2];limit=gaps[block]
        assert limit['old_gaps']==a['nodes']-a['strips']
        gain=a['nodes']-w['nodes'];assert gain<=limit['gap_gain_upper']
        n+=a['nodes'];g+=limit['old_gaps'];actual+=gain;bound+=limit['gap_gain_upper']
        pair+=limit['all_candidate_edges'];same+=limit['same_candidate_edges']
    print(split,len(rows),n,g,actual,bound,f'{bound/n:.8f}',pair,same)
