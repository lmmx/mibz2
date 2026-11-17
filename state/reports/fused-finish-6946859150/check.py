"""Research only: source predicates and exact frozen adjudicator; no timings."""
import contextlib
import io
import json
import math
import sys
from pathlib import Path

root = Path(__file__).parent
sys.path.insert(0, str(root / 'bench'))
import test_two_arm as fixture

fixture.JUDGED, fixture.CONTROLS = [10, 12], []
fixture.ARGS = ['--ref', 'mibz2@build-master', '--cand', 'mibz2-fused',
                '--main', 'wikidata-x4', '--guard', 'text', '--judged', '10,12',
                '--controls', '', '--rounds', '10', '--hold-max-threads', '0']
rows = fixture.runs(main={10: .97}, noise=.01)
for r in rows:
    if r['arm'] == 'mibz2':
        r['arm'] = 'mibz2-fused'
code, output = fixture.run(rows)
assert len(rows) == 80 and code == 0 and 'VERDICT: ADOPTS' in output
assert 'A gain at [10] threads' in output
assert '12 threads: 0.9999' in output
bad = [dict(r) for r in rows]
bad[0]['cpus'] = 'all'
bad_code, bad_out = fixture.run(bad)
assert bad_code == 1 and 'VOID' in bad_out

# Direct algebra of the inspected source predicates, all other i9 checks true.
plans = []
for asked, allowed in [(10,10), (12,20), (10,20)]:
    trimmed = asked >= 10 and allowed == 20
    workers = 7 if trimmed else 10
    slots = 4 * asked * workers // asked if trimmed else 4 * asked
    plans.append(dict(asked=asked, allowed_logical=allowed, trimmed=trimmed,
                      workers=workers, slots=slots))
assert [(p['workers'],p['slots']) for p in plans] == [(10,40),(7,28),(7,28)]

result = dict(source_sha='2bcd82721d637c7e21326006b8475c84a57e9fd1',
    synthetic_only=True, plans=plans, full_roster=80, only_10_gain_adopts=True,
    wrong_pinning_void=True, ratio_of_medians=True,
    naive_serial_fraction=.0154*65/16.85,
    sd_log_needed_at_mean_0985=-math.log(.985)*math.sqrt(10)/2.262
)
(root/'checks.json').write_text(json.dumps(result,indent=2)+'\n')
(root/'synthetic-verdict.txt').write_text(output+'\nInvalid-pinning mutation:\n'+bad_out)
print(json.dumps(result,indent=2))
