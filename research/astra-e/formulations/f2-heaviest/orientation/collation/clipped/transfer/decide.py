"""Replay the preregistered matched-mass transfer after excluding discovery tape 0."""
import csv
from pathlib import Path
import sys

rows = {}
with Path(sys.argv[1]).open() as source:
    for row in csv.DictReader(source):
        ordinal = int(row['ordinal'])
        if ordinal:
            rows.setdefault(ordinal, {})[row['policy']] = row
assert sorted(rows) == list(range(24, 217, 24))
for policy in ('colon', 'quantile'):
    both = 0
    for arms in rows.values():
        ref, test = arms['named'], arms[policy]
        assert test['mass'] == ref['mass']
        both += all(int(test[key]) < int(ref[key]) for key in ('dictionary_bytes', 'lookups'))
    print(policy, 'both_strict', both, 'of', len(rows))
    totals = {}
    for key in ('dictionary_bytes', 'lookups', 'types', 'token_runs', 'intermediate_run_entries', 'run_pieces'):
        baseline = sum(int(arms['named'][key]) for arms in rows.values())
        candidate = sum(int(arms[policy][key]) for arms in rows.values())
        totals[key] = candidate / baseline
        print(key, baseline, candidate, f'{totals[key]:.7f}')
    if policy == 'colon':
        print('advance', both >= 8 and totals['token_runs'] <= 1.10)
