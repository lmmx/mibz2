import csv,sys
from collections import Counter,defaultdict
from pathlib import Path
from statistics import median
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[2]/'bench/moonshot'))
from moonshot import paired_ratio
dir=Path(sys.argv[1]); out=Path(sys.argv[2])
bs=list(csv.DictReader((dir/'blocks.csv').open())); assert len(bs)==228
ge=list(csv.DictReader((dir/'geometry.csv').open())); assert len(ge)==228*8
data=list(csv.DictReader((dir/'times.csv').open()));assert len(data)==9*9*54
assert all(v==6 for v in Counter((r['block'],r['cut'],r['fresh'],r['position']) for r in data).values())
def read(path,rows,keys):
    accum=Counter()
    for r in rows:accum.update({k:int(r[k]) for k in keys})
    return accum
bkeys=[k for k in bs[0] if k!='block']
b=read(dir,bs,bkeys)
lines=['block totals '+str(dict(b)), 'max words '+str(max(int(r['words']) for r in bs)),'cut visits covered_pct digest_bytes unique gaps short_gaps short_gap_bytes max_plan_bytes']
for cut in sorted({int(r['cut']) for r in ge}):
    rows=[r for r in ge if int(r['cut'])==cut]
    c=read(dir,rows,list(rows[0])[2:])
    lines.append(f"{cut} {c['visits']} {100*c['covered']/b['decoded']:.3f} {c['digest_bytes']} {c['unique']} {c['gaps']} {c['short_gaps']} {c['short_gap_bytes']} {max(int(r['visits']) for r in rows)*24}")
(out/'coverage.txt').write_text('\n'.join(lines)+'\n')
groups=defaultdict(lambda:[0]*54)
for r in data:groups[(r['cut'],r['fresh'])][int(r['round'])-1]+=int(r['cpu_ns'])
lines=['cut fresh ratio ci_lo ci_hi median_nine_ms']
for key,vals in sorted(groups.items(),key=lambda t:(int(t[0][0]),t[0][1])):
    ci=paired_ratio(vals,groups['0','false'])
    lines.append(f'{key[0]} {key[1]} {ci["geomean"]:.5f} {ci["lo"]:.5f} {ci["hi"]:.5f} {median(vals)/1e6:.5f}')
(out/'ratios.txt').write_text('\n'.join(lines)+'\n')
print((out/'coverage.txt').read_text(),(out/'ratios.txt').read_text())
