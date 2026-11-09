"""Research-only producer/judge counterexample; no decoding or timing benchmark.

Run beside the exact PR302 bench/*.py files or with them on PYTHONPATH.
This models ab's stdout projection, not an execution of the Rust harness.
"""
import json
import subprocess
import sys
import test_service_guarded as fixture

results = []
for ending in ('sys.exit(0)', 'sys.exit(17)', 'os.kill(os.getpid(), signal.SIGTERM)'):
    rows = fixture.rows()
    payload = json.dumps(rows[0])
    code = 'import sys,os,signal;print(' + repr(payload) + ',flush=True);' + ending
    child = subprocess.run([sys.executable, '-c', code], capture_output=True, text=True)
    # ab uses o.stdout and does not persist o.status. Its parent metadata does
    # not repair the missing status. Retain that projection in this witness.
    rows[0] = json.loads(child.stdout.strip())
    projected = fixture.judge(r=rows)
    rows[0]['exit'] = child.returncode
    status_present = fixture.judge(r=rows)
    results.append({'ending': ending, 'actual_exit': child.returncode,
                    'stdout_projection_valid': projected['valid'],
                    'status_present_valid': status_present['valid'],
                    'status_present_failure': status_present['failure']})
assert [(x['actual_exit'], x['stdout_projection_valid'], x['status_present_valid'])
        for x in results] == [(0, True, True), (17, True, False), (-15, True, False)]
print(json.dumps(results, indent=2))
