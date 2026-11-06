#!/usr/bin/env python3
"""Read-only synthetic probes of compact30 analyser b12b49b.

No benchmark, decoder, hardware counter or clock capture is executed.
The two imported repository files are checked against their Git blob identities.
Run in this directory: python review_gate_checks.py
"""
from pathlib import Path
import hashlib
import json
import sys

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
EXPECTED = {
    'c30_traversal.py': '3c560a96f0f6a807f45aa1c55805bb260a85572f',
    'test_c30_traversal.py': 'a0fb65cebc59028073ad05d97670ac11cac7906f',
}
for name, expected in EXPECTED.items():
    b = (HERE / name).read_bytes()
    actual = hashlib.sha1(f'blob {len(b)}\0'.encode() + b).hexdigest()
    if actual != expected:
        raise RuntimeError(f'{name}: wrong source blob {actual}')

import c30_traversal as analyser
import test_c30_traversal as fixture

passed = []
for name, fn in sorted(vars(fixture).items()):
    if name.startswith('test_') and callable(fn):
        fn()
        passed.append(name)

def report(name, m, w, note):
    r = analyser.analyse(m, w, True)
    return dict(case=name, note=note, valid=r['valid'], decision=r['decision'],
                ratios=r['ratios'], gates=r['gates'], path=r['path'])

cases = []
m,w = fixture.make(rounds=1, c_cost=0.8)
cases.append(report('one_pair_accepted',m,w,
                    'Non-dry run contains only one measured pair, although the protocol requires 20.'))

m,w = fixture.make(c_cost=0.8)
for x in w:
    if x['mode']=='w4':
        x['worker']=(x['worker']+1)%4
        x['cpu']=m['cpus'][x['worker']]
cases.append(report('assignment_mismatch_accepted',m,w,
                    'Every W4 walk is attributed to a worker other than its declared list owner.'))

m,w = fixture.make(c_cost=0.95)
base = analyser.analyse(m,w,True)
for x in w:
    if x['mode']=='w4' and x['arm']=='C':
        x['rows']*=2
q = report('row_denominator_corruption_accepted',m,w,
           'Candidate W4 row counts are doubled; block_rows and allocation metadata are unchanged.')
q['uncorrupted_decision']=base['decision']
q['uncorrupted_ratios']=base['ratios']
cases.append(q)

m,w = fixture.make(c_cost=0.6)
for x in w:
    if x['mode']=='w4' and x['arm']=='C' and x['step']==0 and x['worker']==0:
        x['step']=99
cases.append(report('split_full_step_accepted',m,w,
                    'Each candidate round has one scheduled full quartet relabelled as three workers plus a singleton step; another valid quartet remains.'))

m,w = fixture.make(c_cost=0.8)
previous = next(x for x in w if x['round']==0 and x['arm']=='C' and x['mode']=='w4' and x['step']==0 and x['worker']==0)
following = next(x for x in w if x['round']==0 and x['arm']=='C' and x['mode']=='w4' and x['step']==1 and x['worker']==0)
following['prep_start']=previous['start']+1
following['prep_end']=previous['end']-1
cases.append(report('next_preparation_overlaps_prior_walk_accepted',m,w,
                    'One next-step preparation occurs during a prior timed walk; per-step checks do not detect cross-step overlap.'))

out = dict(source_pin='b12b49b44e27b96c8e16e0ccab4e348c1ba56bba',
           scope='Synthetic analysis records only; no decoder or timing execution',
           verified_git_blobs=EXPECTED, existing_tests_passed=passed,
           cases=cases,
           exact_screen_for_b_equal_one=1-0.25/1.847,
           b_max_at_rounded_screen=analyser.b_max(0.865),
           saving_at_rounded_screen_and_b_one=1.847*(1-0.865))
path=HERE/'gate-review-results.json'
path.write_text(json.dumps(out,indent=2)+'\n')
print('Original tests passed:',len(passed))
for c in cases:
    print(c['case'], 'valid=',c['valid'], 'decision=',c['decision'],
          'w4=',round(c['ratios']['w4']['point'],6),
          'e4=',round(c['ratios']['e4']['point'],6))
print('Exact threshold:',out['exact_screen_for_b_equal_one'])
print('Budget at rounded threshold:',out['b_max_at_rounded_screen'])
