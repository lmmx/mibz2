"""Synthetic replay probes for PR318 at 795d4d75; no decoder or timing run."""
import contextlib
import io
import json
from pathlib import Path
import sys
import tempfile
sys.path.insert(0, str(Path(__file__).parent / 'bench'))
import two_modes as tm
import test_two_modes as fx

def replay(edit):
    with tempfile.TemporaryDirectory() as d:
        fx.attempt(d, 'H3', gaps=(0, 0))
        edit(Path(d))
        with contextlib.redirect_stdout(io.StringIO()):
            rc = tm.main(['judge', d])
        j = json.loads((Path(d) / 'judgement.json').read_text())
        return {'rc': rc, 'label': j['label'], 'problems': j['problems'],
                'gaps': j['gaps']['outcome'], 'conditions': j['gaps']['conditions']}

def gap_edit(d, f):
    p = d / 'gaps/runs.jsonl'
    rows = [json.loads(x) for x in p.read_text().splitlines()]
    f(rows)
    p.write_text(''.join(json.dumps(r) + '\n' for r in rows))

def identity_edit(d, f):
    p = d / 'identity.json'
    x = json.loads(p.read_text())
    f(x)
    p.write_text(json.dumps(x))

out = {}
out['baseline'] = replay(lambda d: None)
out['missing_G_walls'] = replay(lambda d: gap_edit(d, lambda rows: [r['child'].pop('wall') for r in rows if r['condition']=='G']))
out['first_G_gap_missing'] = {'condition': tm.GAP_ORDER[0], 'gap': fx.gap_records()[0]['gap_s'], 'problems': tm.gaps_problems(fx.gap_records(), 60, False)}
out['negative_Z_gap'] = replay(lambda d: gap_edit(d, lambda rows: [r.update(gap_s=-1.0) for r in rows if r['condition']=='Z']))
out['reverse_gap_timestamps'] = replay(lambda d: gap_edit(d, lambda rows: [r.update(t_spawn_unix_ns=60-i, span_s=-1) for i,r in enumerate(rows)]))
out['zero_rest_and_deadlines'] = replay(lambda d: identity_edit(d, lambda x: (x.update(deadlines={}), x['sustained'].update(rest_s=0))))
out['zero_rest_only'] = replay(lambda d: identity_edit(d, lambda x: x['sustained'].update(rest_s=0)))
out['empty_deadlines_only'] = replay(lambda d: identity_edit(d, lambda x: x.update(deadlines={})))
out['one_missing_G_wall'] = replay(lambda d: gap_edit(d, lambda rows: rows[0]['child'].pop('wall')))
for k,v in out.items():
    if 'label' in v:
        assert v['label'] == 'VALID' and v['rc']==0 and not v['problems'], (k,v)
assert '30/30' in out['missing_G_walls']['gaps'] and '0/30' in out['missing_G_walls']['gaps']
assert out['one_missing_G_wall']['conditions']['G']['slow']==1
Path(__file__).with_name('probe-results.json').write_text(json.dumps(out, indent=2)+'\n')
print(json.dumps(out, indent=2))
