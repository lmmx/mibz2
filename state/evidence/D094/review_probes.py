"""Guardian review witnesses, not the diagnostic's production adjudicator.
Run beside the pinned walk_stats_check.py, its test module and service_tg.py.
"""
import json, re, subprocess, sys
from pathlib import Path
import test_walk_stats_check as fixture
import service_tg as tg

ROOT = Path(__file__).resolve().parent

def specified_accept(exit_code, stdout, stderr='', timeout=False):
    if timeout or exit_code not in (0, 1):
        return False
    lines = stdout.splitlines()
    if stderr or len(lines) != 6 or lines[0] != 'VALID':
        return False
    predicates = []
    for letter, line in zip('abcd', lines[1:5]):
        m = re.fullmatch(r'  \(' + letter + r'\) (holds|fails): .+', line)
        if not m:
            return False
        predicates.append(m[1] == 'holds')
    engaged = all(predicates)
    if lines[-1] != ('ENGAGED' if engaged else 'NOT ENGAGED'):
        return False
    if exit_code != (0 if engaged else 1):
        return False
    return all(predicates[i] for i in (0, 2, 3))

def checker(name, amounts, wrong_walker=False, terminal=True):
    rows = [dict(kind='share', team=0, block=0, thread=h) for h in (1, 2)]
    for t, n in enumerate(amounts):
        rows.append(dict(kind='call', team=0, block=0, thread=t,
            role='leader' if t == 0 else 'helper',
            walker='traverse_staged' if wrong_walker and t == 1 else 'traverse',
            takes=int(n > 0), exhausts=1, bytes=n, tail=0, fragments=0, copied=0))
    rows.append(dict(kind='summary', team=0, block=0, rows=sum(amounts),
        starts=sum(n > 0 for n in amounts), seg_bytes=sum(amounts), entered=7, joined=True, built=6))
    lines = fixture.lines(rows)
    if not terminal:
        lines = lines[:-1]
    path = ROOT / (name + '.jsonl')
    path.write_text('\n'.join(lines) + '\n')
    (ROOT / 'classes.txt').write_text('T\n')
    p = subprocess.run([sys.executable, str(ROOT / 'walk_stats_check.py'), str(path),
        '--walker', 'traverse', '--helpers', '2', '--classes', str(ROOT / 'classes.txt')],
        text=True, capture_output=True)
    return dict(exit=p.returncode, stdout=p.stdout, stderr=p.stderr,
                accepted=specified_accept(p.returncode, p.stdout, p.stderr))

out = {'checker': {}, 'shape_controls': {}, 'confirmation': {}}
for name, amounts, wrong, terminal, expected in [
    ('balanced', (100,100,100), False, True, True),
    ('balance_only', (100,1,99), False, True, True),
    ('zero_helpers', (100,0,0), False, True, True),
    ('wrong_walker', (100,50,50), True, True, False),
    ('missing_terminal', (100,1,99), False, False, False),
]:
    v = checker(name, amounts, wrong, terminal)
    assert v['accepted'] == expected, (name, v)
    out['checker'][name] = v
good = out['checker']['balanced']['stdout']
bad = out['checker']['balance_only']['stdout']
controls = {
    'truncated': (0, '\n'.join(good.splitlines()[:-1])+'\n', '', False),
    'crash': (1, '', 'Traceback: fixture\n', False),
    'timeout': (0, good, '', True),
    'signal': (-9, good, '', False),
    'exit_disagrees_engaged': (1, good, '', False),
    'exit_disagrees_not_engaged': (0, bad, '', False),
    'stderr_warning': (0, good, 'warning\n', False),
    'extra_stdout': (0, good+'reference\n', '', False),
}
for name, args in controls.items():
    out['shape_controls'][name] = specified_accept(*args)
    assert out['shape_controls'][name] is False
for name, a, b in [('empty', [], []), ('duplicate_members', [1]*12, [2]*12),
                    ('single_runs', [1], [2])]:
    v = lambda rs: dict(valid=True, head='a'*40, runs=rs, flags=[])
    out['confirmation'][name] = tg.confirm(v(a), v(b))
    assert out['confirmation'][name]['valid'] is True
(ROOT/'review-probes.json').write_text(json.dumps(out, indent=2)+'\n')
print(json.dumps({'checker': {k: {'exit': v['exit'], 'accepted': v['accepted']} for k,v in out['checker'].items()},
                  'shape_controls_rejected': len(controls), 'confirmation_bad_inputs_accepted': 3}))
