"""CPU, wall, occupancy and whole-process peak for the direct seven-arm CI contrasts."""
from pathlib import Path
import csv
import hashlib
import json
import statistics
import sys
import zipfile

root=Path(__file__).resolve().parents[4]
sys.path.insert(0,str(root/'bench/moonshot'))
from moonshot import paired_ratio
z=root/'research/astra-e/ci/evidence/cap-counter-44460943655.zip'
assert hashlib.sha256(z.read_bytes()).hexdigest()=='020be93f298c581c8c6743b0684524401a40ac11b9d85e1a44253062c7f24458'
with zipfile.ZipFile(z) as archive:
    rows=[json.loads(x) for x in archive.read('runs.jsonl').splitlines()]
    assert len(rows)==588
    ratios=json.loads(archive.read('ratios.json'))
by={(r['workers'],r['arm'],r['round']):r for r in rows}
writer=None
for w in [1,2,4]:
    for ref,name in [('lazy','master LazyCells'),('cells','master eager Cells'),('opend','master opend'),
                     ('edge','frozen edge'),('predecessor','frozen predecessor'),('counter','frozen counter')]:
        arms=[[by[w,arm,i] for i in range(28)] for arm in ['candidate',ref]]
        entry=dict(workers=w,reference=ref,rounds=28)
        for label in ['wall_s','cpu_s','peak_rss_kb','cpu_per_wall']:
            a,b=[[r['cpu_s']/r['wall_s'] if label=='cpu_per_wall' else r[label] for r in arm] for arm in arms]
            ratio=paired_ratio(a,b)
            if label=='wall_s':
                assert abs(ratio['geomean']-ratios[name][str(w)]['candidate']['geomean'])<1e-12
            entry.update({f'{label}_{k}':ratio[k] for k in ['geomean','lo','hi']})
            entry[f'{label}_median_combined']=statistics.median(a)
            entry[f'{label}_median_reference']=statistics.median(b)
            if label=='peak_rss_kb':
                entry['maximum_combined_kib']=max(a)
                entry['maximum_reference_kib']=max(b)
        if writer is None:
            writer=csv.DictWriter(sys.stdout,fieldnames=entry,lineterminator='\n')
            writer.writeheader()
        writer.writerow(entry)
