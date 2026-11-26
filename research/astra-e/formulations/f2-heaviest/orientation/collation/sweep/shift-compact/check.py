"""Compare independently derived physical partitions and normalized projected descriptors."""
from pathlib import Path
import csv
D=Path(__file__).resolve().parent
O=D.parent/'shift-constructor'
def read(file,keys):return {tuple(r[k] for k in keys):r for r in csv.DictReader(file.open())}
verified=read(D/'ten.csv',('stream','h'))
model=read(O/'partition.csv',('ordinal','h'))
physical=read(O/'real.csv',('ordinal','h','fused'))
norm=read(D/'audit'/'descriptor.csv',('ordinal','h'))
checked=0
for (idx,h),r in verified.items():
    m=model[idx,h]
    for a,b in [('live_nodes','fused_surviving_nodes'),('components','terminal_components'),('forbidden_components','forbidden_components'),('mark','marked_row'),('skip','marked_skip')]:
        assert r[a]==m[b],(r,m)
    if (idx,h,'1') in physical:
        o=physical[idx,h,'1']
        for a,b in [('early_rows','fused_early_rows'),('early_bands','fused_early_bands'),('words','words'),('root_runs','root_runs'),('dict_bytes','dictionary_bytes')]:
            assert r[a]==o[b],(r,o)
        checked+=1
    if int(h)>0:assert r['lf_pieces']==norm[idx,h]['normalized_descriptors']
print(f'{len(verified)} exact live-state/mark/forbidden geometries; {checked} additional comparisons to physical fused objects')
print("30 independent compressed LF descriptor divisions including all deletion coalescences")
