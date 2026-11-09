"""Untimed admission witnesses against #277 at 7f88388b; no decoder benchmark."""
import copy
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
sys.path.insert(0, str(ROOT / 'bench'))
import test_run_fed_stage as t

v = t.v
ev = v.expected_corpora(str(ROOT))
base = t.base()
# Match source-generated text identity, leaving the fixture's made-up path counts.
base['files'][1].update({k: ev[1][k] for k in ('sha256','bytes','output','blocks')})
assert not v.check(base, t.HEAD, ev)
out = {'source': '7f88388b938528f22088df79387f309eb112fe1c', 'synthetic_text': ev[1], 'cases': []}
cases = [
    ('zero fallback blocks but one fallback row', 3, {'rows': 96531}),
    ('fallback rows exceed 47 block capacity', 0, {'rows': 206000000}),
    ('one run but 256 nonempty byte buckets', 3, {'d_streams': 256}),
    ('B slots below eight per cell', 0, {'cells': 20000000, 'slots': 159999992, 'd_vector': 19000000, 'd_scalar': 0, 'rows_runs': 150000000}),
]
for name, i, changes in cases:
    rec = copy.deepcopy(base)
    rec['files'][i].update(changes)
    p = v.check(rec, t.HEAD, ev)
    out['cases'].append({'name': name, 'file': i, 'set': changes, 'problems': p,
                         'readings': v.readings(rec) if not p else None})
f = base['files'][1]
out['base_fixture_impossibilities'] = {'text_cells': f['cells'], 'B_slots': f['slots'],
    'minimum_B_slots': 8*f['cells'], 'D_store_instructions': f['d_vector']+f['d_scalar'],
    'minimum_D_store_instructions': f['cells']}
Path(ROOT / 'witnesses.json').write_text(json.dumps(out, indent=2)+'\n')
for x in out['cases']:
    print(x['name'], 'VALID' if not x['problems'] else x['problems'])
print(out['base_fixture_impossibilities'])
