"""Untimed R12 replay. Argument: exact-head repository checkout containing bench/."""
import argparse, contextlib, io, json, pathlib, sys, tempfile
sys.path.insert(0, str(pathlib.Path(sys.argv[1]).resolve() / 'bench'))
import run_fed_leader_ws as w
from test_run_fed_leader_ws import host_env, snapshot, LSCPU_I9

names = ('ab env --facts', 'lscpu', 'ab sample')
good = ''.join(f'{n}: exit 0\n' for n in names)
cases = [('all-zero', good, 'PASS'), ('reordered', '\n'.join(reversed(good.splitlines())), 'PASS')]
for name in names:
    for status in (17, 124, 137):
        cases.append((f'{name}-exit-{status}', good.replace(f'{name}: exit 0', f'{name}: exit {status}'), 'REFUSED'))
    cases += [(f'{name}-missing', good.replace(f'{name}: exit 0\n', ''), 'REFUSED'),
              (f'{name}-duplicate', good + f'{name}: exit 0\n', 'REFUSED'),
              (f'{name}-failure-then-success', f'{name}: exit 17\n' + good, 'REFUSED')]
cases += [('missing-file', None, 'REFUSED'), ('empty-file', '', 'REFUSED'),
          ('unknown', good + 'other: exit 0\n', 'REFUSED'),
          ('malformed', good.replace('lscpu: exit 0', 'lscpu: exit 0junk'), 'REFUSED')]
results = []
for label, text, expected in cases:
    with tempfile.TemporaryDirectory() as td:
        d = pathlib.Path(td); p = d / 'preflight'; p.mkdir()
        (p / 'env.json').write_text(json.dumps(host_env()))
        (p / 'lscpu.txt').write_text(LSCPU_I9)
        (p / 'sample.jsonl').write_text(''.join(json.dumps({'state': snapshot(4900, t=i)}) + '\n' for i in range(3)))
        if text is not None:
            (p / 'exits.txt').write_text(text)
        with contextlib.redirect_stdout(io.StringIO()):
            rc = w.preflight(argparse.Namespace(dir=td, out=str(p)))
        verdict = json.loads((d / 'preflight.json').read_text())['verdict']
        results.append(dict(case=label, checker_exit=rc, verdict=verdict, expected=expected))
        assert (rc, verdict) == ((0 if expected == 'PASS' else 1), expected), results[-1]
print(json.dumps(results, indent=2))
