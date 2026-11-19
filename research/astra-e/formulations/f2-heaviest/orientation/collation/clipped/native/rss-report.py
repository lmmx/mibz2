"""Combine six isolated sequential corpus runs and check the verified output reference."""
from pathlib import Path
import csv
import hashlib
import sys
scratch=Path(sys.argv[1])
rows={}
for arm in ('original','quote','frequent','raw_quote','fifth','half'):
    r=list(csv.DictReader((scratch/f'rss-{arm}.csv').open()))
    assert len(r)==228 and all(x['arm']==arm for x in r)
    rows[arm]=r
with (scratch/'rss.csv').open('w') as out:
    w=csv.DictWriter(out,fieldnames=rows['original'][0].keys(),lineterminator='\n')
    w.writeheader()
    for arm,r in rows.items():
        w.writerows(r)
        print(arm,max(int(x['high_water_kib']) for x in r),max(int(x['retained_bytes_including_output']) for x in r))
total=sum(x['retained_bytes_including_output']==y['retained_bytes_including_output'] for x,y in zip(rows['quote'],rows['original']))
print('quote vs original equal live capacities',total,'of 228')
assert hashlib.sha256((scratch/'expected.csv').read_bytes()).hexdigest()=='371c94a260c2f9a01cca5580cbee43ac687cc7b8d1a2b1a3d3e8b295c4131c10'
