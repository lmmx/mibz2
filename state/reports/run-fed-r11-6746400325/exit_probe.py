import argparse, contextlib, io, json, pathlib, tempfile
import run_fed_leader_ws as w
from test_run_fed_leader_ws import host_env, snapshot, LSCPU_I9

results=[]
for status in (0,17,124,137):
    with tempfile.TemporaryDirectory() as td:
        d=pathlib.Path(td); p=d/'preflight'; p.mkdir()
        (p/'env.json').write_text(json.dumps(host_env()))
        (p/'lscpu.txt').write_text(LSCPU_I9)
        (p/'sample.jsonl').write_text(''.join(json.dumps({'state':snapshot(4900,t=i)})+'\n' for i in range(3)))
        (p/'exits.txt').write_text(f'ab env --facts: exit {status}\nlscpu: exit 0\nab sample: exit 0\n')
        with contextlib.redirect_stdout(io.StringIO()):
            rc=w.preflight(argparse.Namespace(dir=td,out=str(d/'preflight')))
        result=json.loads((d/'preflight.json').read_text())
        results.append({'collector_exit':status,'checker_exit':rc,'verdict':result['verdict']})
assert all(r['verdict']=='PASS' and r['checker_exit']==0 for r in results)
print(json.dumps(results,indent=2))
