"""Summarise exact edit, span-move and fallback counters from the one-worker probe."""
import csv
import json
import sys
from collections import Counter

with open(sys.argv[1]) as f:
    rows = []
    for row in csv.reader(f):
        assert row[0]=='slot' and len(row)==12
        rows.append(dict(zip(('n','primary','strategy','edits','scanned','left','right','filled',
                              'output','directory_bytes','fallback_capacity'),
                             [int(row[1]),int(row[2]),row[3], *map(int,row[4:])])))
def report(group):
    total = lambda k: sum(row[k] for row in group)
    peak = lambda k: max(row[k] for row in group)
    return dict(blocks=len(group), strategy=dict(Counter(row['strategy'] for row in group)),
                encoded_bytes=total('n'), output_bytes=total('output'), recorded_edits=total('edits'),
                scanned=total('scanned'), literal_bytes_moved_left=total('left'),
                literal_bytes_moved_right=total('right'), run_bytes_filled=total('filled'),
                initialized_new_tail_bytes=sum(max(0,r['output']-r['n']) for r in group if r['strategy']=='Applied'),
                fallback_reparse_bytes=sum(r['n'] for r in group if r['strategy']!='Applied'),
                maximum_directory_bytes=peak('directory_bytes'),
                maximum_fallback_capacity=peak('fallback_capacity'))
print(json.dumps(dict(frozen=report(rows[:1]), all_gated=report(rows)), indent=2))
