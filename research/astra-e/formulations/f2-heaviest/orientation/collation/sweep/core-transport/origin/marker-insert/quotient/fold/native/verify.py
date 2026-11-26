"""Check every independently recognized class, both maximizers and native real geometry."""
import csv,gzip,sys
from itertools import groupby
from pathlib import Path

def rows(path):
    reader=gzip.open(path,'rt') if str(path).endswith('.gz') else path.open()
    with reader: return list(csv.DictReader(reader))

def verify(native,reference,ten,published):
    actual,expected=rows(native),rows(reference)
    assert len(actual)==len(expected)==35117
    lookup={int(r['ordinal']):r for r in rows(published)}
    real={int(r['stream']):r for r in rows(ten) if r['arm']=='fold'}
    for r,s in zip(actual,expected):assert all(r[k]==s[k] for k in r)
    print('stream,types,canonical_original_choice_diffs,removed,old_top_base,old_top_frequency,selected_to_old_top,remaining_old_top,new_top_base_old_coord,new_top_frequency')
    for stream,group in groupby(expected,key=lambda r:int(r['stream'])):
        group=list(group);pub=lookup[stream];obs=real[stream]
        for n,p in [('quote','old_rows'),('tokens','new_rows'),('eligible','eligible'),('eligible_mass','eligible_mass'),('chosen','selected'),
                    ('removed','removed_rows'),('empty','empty_buckets'),('old_runs','old_runs'),('new_runs','new_runs'),('moved','mark_moved')]:
            assert int(obs[n])==int(pub[p])
        diff=sum(r['canonical']!=r['original'] for r in group)
        freq={int(r['base']):int(r['mass']) for r in group};after=dict(freq)
        for r in group:
            if r['canonical']=='1':after[int(r['succ'])]-=int(r['mass'])
        old=max(freq,key=lambda k:(freq[k],-k));new=max(after,key=lambda k:(after[k],-k))
        print(f"{stream},{len(group)},{diff},{obs['removed']},{old},{freq[old]},{freq[old]-after[old]},{after[old]},{new},{after[new]}")
if __name__=='__main__':verify(*(Path(x) for x in sys.argv[1:]))
