"""Check the crossover and summarize nine-block, within-pass paired CPU and wall ratios."""
from collections import Counter, defaultdict
import csv
from pathlib import Path
from statistics import median
import sys

here = Path(__file__).resolve().parent
sys.path.insert(0, str(here.parents[3] / 'bench/moonshot'))
from moonshot import paired_ratio

arms = ['split64', 'packed32', 'packed40', 'packed44', 'wide64']
out = csv.writer(sys.stdout, lineterminator='\n')
out.writerow(['file','metric','arm','reference','ratio','ci_lo','ci_hi','sum_block_medians_ms'])
for file in map(Path, sys.argv[1:]):
    rows = list(csv.DictReader(file.open()))
    assert len(rows) == 9*30*5
    assert len({(r['block'], r['round'], r['arm']) for r in rows}) == len(rows)
    assert set(Counter((r['block'], r['position'], r['arm']) for r in rows).values()) == {6}
    for metric in ['prep_mark_ns','discover_ns','merge_ns','residual_output_ns','inverse_cpu_ns','wall_ns']:
        laps = defaultdict(lambda:[0]*30)
        blocks = defaultdict(list)
        for r in rows:
            t = int(r[metric])
            laps[r['arm']][int(r['round'])-1] += t
            blocks[r['arm'],r['block']].append(t)
        for a in arms:
            ratio = paired_ratio(laps[a],laps['split64'])
            typical = sum(median(samples) for (arm,b),samples in blocks.items() if arm==a)
            out.writerow([file.name,metric,a,'split64',*[f'{ratio[k]:.5f}' for k in ['geomean','lo','hi']],f'{typical/1e6:.5f}'])
