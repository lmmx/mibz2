"""Untimed admission probes against unmodified 7ed5b29 reporter and fixtures."""
import json
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent / 'bench'))
import test_c29_checkpoints as t

results = []
def check(name, recs, ident, expected):
    code, out = t.report(recs, ident)
    assert code == expected, (name, code, out)
    results.append({'case': name, 'exit': code, 'output': out})

for ratio in (1.0, 0.97):
    check('positive_'+str(ratio), t.runs(ratio), t.identity(), 0)
for field in ('avx2', 'cpus_allowed', 'available_parallelism', 'cpus'):
    for position in (0, 39, 79):
        recs = t.runs(1.0)
        del recs[position][field]
        check(f'missing_{field}_record_{position}', recs, t.identity(), 3)
for value in (False, 2.0, '4', float('nan'), float('inf')):
    check('bad_quota_'+repr(value), t.runs(), t.identity(cpu_quota=value), 3)
ident = t.identity()
del ident['cpu_quota']
check('missing_quota_key', t.runs(), ident, 3)
# Exact workflow mapping: env.get("cpu_quota") maps an absent fact to None.
env = {k: v for k, v in t.IDENTITY_FACTS.items() if k != 'cpu_quota'}
check('workflow_absent_quota_mapped_to_null', t.runs(1.0),
      t.identity(cpu_quota=env.get('cpu_quota'), env=env), 0)
# ab env's match quota { Some(Some(q)) => number, _ => null } also merges
# a failed detector (None) and known unlimited (Some(None)); this is an
# evidence-encoding counterexample, not a claim about any actual CI machine.
check('unreadable_quota_serialized_as_null', t.runs(1.0),
      t.identity(cpu_quota=None), 0)
check('failed_parent_with_complete_records', t.runs(1.0),
      t.identity(ab_run_status=101), 0)
recs = t.runs(facts={})
recs[0]['exit'] = 1
check('invalid_precedes_fallback', recs, t.identity(avx2=False), 2)
Path(__file__).with_name('probe-results.json').write_text(json.dumps(results, indent=2)+'\n')
print(json.dumps([{'case': r['case'], 'exit': r['exit'], 'first_line': r['output'].splitlines()[0]} for r in results], indent=2))
