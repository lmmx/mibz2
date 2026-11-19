"""Verify the local complete cap-counter score and directly re-pair the opend control."""
import csv
import json
from pathlib import Path
from statistics import median
import sys

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parents[3]/'bench/moonshot'))
from moonshot import paired_ratio

raw = [json.loads(line) for line in (here/'score/runs.jsonl').read_text().splitlines()]
cases = [json.loads(line) for line in (here/'score/check.jsonl').read_text().splitlines()]
assert len([row for row in cases if row.get('pass') is True]) == 338
assert 'PASS (338 cases, 0 failed)' in (here/'score/summary.txt').read_text()
assert len(raw) == 54 and {x['out_bytes'] for x in raw} == {419400392}
by_id = {(x['workers'], x['round'], x['arm']): x for x in raw}
writer = csv.writer(sys.stdout,lineterminator='\n')
writer.writerow(('workers','other','wall','lo','hi','median_peak','max_peak_candidate','max_peak_other'))
for w in (1,4):
    cand = [by_id[w,i,'candidate'] for i in range(9)]
    for name in ('lazy','opend'):
        ref = [by_id[w,i,name] for i in range(9)]
        ratio = paired_ratio([x['wall_s'] for x in cand], [x['wall_s'] for x in ref])
        c = [x['peak_rss_kb'] for x in cand]
        r = [x['peak_rss_kb'] for x in ref]
        writer.writerow((w,name,ratio['geomean'],ratio['lo'],ratio['hi'],median(c)/median(r),max(c),max(r)))
