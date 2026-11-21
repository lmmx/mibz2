"""Price the two endpoint layouts on the full archived width-16 run-front census."""
import csv
from pathlib import Path
import sys

here = Path(__file__).resolve().parent
source = here.parents[2] / 'docs/journal/2026-11-18-open-return/evidence/blocks.csv'
output = csv.writer(sys.stdout, lineterminator='\n')
output.writerow(['ordinal', 'rows', 'strips', 'node_count_including_sentinel', 'admission_bound',
                 'steps', 'node_bytes_32', 'node_bytes_16'])
with source.open() as file:
    for row in csv.DictReader(file):
        if row['corpus'] != 'wikidata' or row['run_front'] != '1':
            continue
        strips = int(row['H'])
        nodes = 1 + int(row['open_nodes'])
        bound = 3 * strips + 3
        assert nodes <= bound <= 65_535
        output.writerow([row['ordinal'], row['n'], strips, nodes, bound, row['steps'], 24 * nodes, 16 * nodes])
