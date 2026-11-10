"""Untimed adversarial replays for PR313 head 9323525026d9dcea55cc1e7956a604241c426f9a."""
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import importlib.util

HERE = Path(__file__).parent
text = b'aab'
rotations = sorted((text[i:] + text[:i], i) for i in range(len(text)))
l = bytes(r[-1] for r, _ in rotations)
origin = next(j for j, (_, i) in enumerate(rotations) if i == 0)
f = sorted(l)
psi = sorted(range(len(l)), key=lambda i: l[i])
cell, k = [], -1
for i in range(len(l)):
    if i == 0 or f[i] != f[i-1] or psi[i] != psi[i-1] + 1:
        k += 1
    cell.append(k)
traj, y = [], origin
for _ in l:
    traj.append(y)
    y = psi[y]
q, t = 0, 1
d = traj[t] - traj[q]
m = 0
while t+m < len(l) and traj[t+m] - traj[q+m] == d:
    m += 1
source = bytes(f[traj[q+i]] for i in range(m))
target = bytes(f[traj[t+i]] for i in range(m))
room = 0
while t+room < len(l) and cell[traj[q+room]] == cell[traj[t+room]]:
    room += 1
assert l == b'baa' and origin == 0 and traj == [0,1,2]
assert m == 2 and source == b'aa' and target == b'ab' and room == 1
results = {'census': {'text': text.decode(), 'L': l.decode(), 'origin': origin,
    'F': bytes(f).decode(), 'psi': psi, 'trajectory': traj, 'cells': cell,
    'q': q, 't': t, 'displacement': d, 'coherent_length': m,
    'source': source.decode(), 'target': target.decode(), 'same_cell_length': room,
    'scope': 'Python replay of census displacement predicate; not native Rust execution'}}
spec = importlib.util.spec_from_file_location('fixtures', HERE / 'test_phrase_knee.py')
fixtures = importlib.util.module_from_spec(spec)
spec.loader.exec_module(fixtures)
rows = [json.loads(x) for x in fixtures.lines(3, [1,2]).splitlines()]
results['reporter'] = {}
with tempfile.TemporaryDirectory() as tmp:
    p = Path(tmp)
    for test in (fixtures.test_complete_grid, fixtures.test_missing_cell_refused, fixtures.test_duplicate_refused):
        test(p)
    results['existing_tests'] = '3 passed (direct invocation)'
    for label, mutate in (
        ('valid', lambda r: None),
        ('nan_wall', lambda r: r.update(wall_ns=float('nan'))),
        ('negative_build', lambda r: r.update(build_ns=-1000000)),
        ('wrong_bytes', lambda r: r.update(bytes=1)),
    ):
        sample = [dict(r) for r in rows]
        for r in sample:
            if r['arm'] == 'phrase': mutate(r)
        run = fixtures.run(p, '\n'.join(json.dumps(r) for r in sample)+'\n')
        assert run.returncode == 0, (label, run.stderr)
        results['reporter'][label] = {'exit': run.returncode, 'stdout': run.stdout}
print(json.dumps(results, indent=2))
