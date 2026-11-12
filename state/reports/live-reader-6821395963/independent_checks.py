"""Review-only checks. No decoder execution or performance clock; fake measurements only."""
import copy
import hashlib
import importlib.util
import json
import math
import statistics
import subprocess
import sys
import tempfile
from pathlib import Path
from types import SimpleNamespace
from unittest import mock

HERE = Path(__file__).resolve().parent
H = HERE / 'input/bench/cells-gap/live'
sys.path.insert(0, str(H))
import driver

result = {}
logs = [-0.1 + (i - 4.5) * 0.02 for i in range(10)]
iv = driver.interval([math.exp(x) for x in logs])
# Sum (i-4.5)^2 = 82.5; sample variance of logs = .0004*82.5/9.
half = 2.2621571628540993 * math.sqrt(0.0004 * 82.5 / 90)
assert math.isclose(iv['ratio'], math.exp(-.1), rel_tol=1e-14)
assert math.isclose(iv['low'], math.exp(-.1-half), rel_tol=1e-14)
assert math.isclose(iv['high'], math.exp(-.1+half), rel_tol=1e-14)
for vals in [[1]*9, [1]*9+[0], [1]*9+[math.inf]]:
    try: driver.interval(vals)
    except ValueError: pass
    else: raise AssertionError('bad ratio accepted')
result['log_t_closed_form'] = iv

rows = []
for j in driver.schedule()[9:]:
    wall = {('baseline', False):1000, ('cells', False):900,
            ('baseline', True):4000, ('cells', True):1800}[j['arm'],j['trace']]
    r = dict(j, timed=True, wall_ns=wall)
    if j['trace']:
        r.update(clock_calls=2, blocks=[], phases={
            'whole_reader_drain':dict(calls=1,inclusive_ns=wall-100,exclusive_ns=wall-100)})
    rows.append(r)
a = driver.analyse(rows)
assert all(x['quantitative_attribution_valid'] for x in a.values())
result['same_category_large_bias'] = {k:a['wikidata'][k] for k in
    ['cells_over_baseline','inflation','quantitative_attribution_valid']}
result['same_category_large_bias']['interaction_ratio'] = .45/.9
for mutation in ['negative_remainder','broken_exclusive','category_flip']:
    changed = copy.deepcopy(rows)
    for r in changed:
        if not r['trace']: continue
        p = r['phases']['whole_reader_drain']
        if mutation == 'negative_remainder':
            p['inclusive_ns'] = p['exclusive_ns'] = r['wall_ns']+1
        elif mutation == 'broken_exclusive': p['exclusive_ns'] += 1
        elif r['arm']=='cells': r['wall_ns'] = 4800
    assert all(not x['quantitative_attribution_valid'] for x in driver.analyse(changed).values())
result['three_invalidation_cases'] = 'rejected'

failures = []
for label,status,body in [('nonzero',7<<8,'{"error":"CRC"}'),
                          ('malformed_json',0,'not JSON'),
                          ('decoder_error',0,'{"error":"CRC"}')]:
    with tempfile.TemporaryDirectory() as d:
        root=Path(d); child=mock.MagicMock(pid=123);child.__enter__.return_value=child
        def launch(command,stdout,stderr,**kwargs):
            stdout.write(body);stdout.flush();stderr.write('failure retained');stderr.flush()
            return child
        usage=SimpleNamespace(ru_utime=.1,ru_stime=.2,ru_maxrss=321)
        with mock.patch.object(driver.subprocess,'Popen',side_effect=launch), \
             mock.patch.object(driver.os,'wait4',return_value=(123,status,usage)), \
             mock.patch.object(driver.time,'perf_counter_ns',side_effect=[100,250]):
            try: driver.checked_child(['fixture'],root/'stdout.json',root/'stderr',measured=True)
            except (ValueError, subprocess.CalledProcessError) as e:
                files=sorted(p.name for p in root.iterdir())
                assert files==['stderr','stdout.json']
                assert not hasattr(e,'resources')
                failures.append(dict(case=label,exception=type(e).__name__,retained=files,
                    collected_but_not_retained=dict(wall_ns=150,user_s=.1,system_s=.2,rss_kib=321)))
            else: raise AssertionError('failure did not propagate')
result['failed_child_retention'] = failures

E = HERE / 'input/docs/journal/2026-11-12-cells-live/evidence'
manifest=json.loads((E/'SHA256.json').read_text())
bad=[p for p,h in manifest.items() if hashlib.sha256((E/p).read_bytes()).hexdigest()!=h]
assert not bad,bad
result['archived_evidence_hashes'] = dict(checked=len(manifest),mismatches=bad)
valid=[]
for name in ['empty','runs','column','repetition']:
    records=[json.loads((E/f'fixtures/{name}-{arm}-{trace}.json').read_text())
             for arm in ['baseline','cells'] for trace in ['off','on']]
    assert len({(r['bytes'],r['hash'],r['sha256'],r['error']) for r in records})==1
    for r in records: driver.reconcile(r)
    valid+=records
for name in ['truncated','block-crc']:
    records=[json.loads((E/f'fixtures/error-{name}-{arm}-{trace}.json').read_text())
             for arm in ['baseline','cells'] for trace in ['off','on']]
    assert len({(r['bytes'],r['hash'],r['error']) for r in records})==1
    assert all(r['error'] for r in records)
result['archived_fixture_json_replay'] = dict(valid=16,errors=8,io_empty_stdout=4,
    limit='replayed reported identities and accounting only; no local decoder or full-byte replay')
spec=importlib.util.spec_from_file_location('coord',HERE/'coord.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
if (HERE/'audit-input.json').exists():
    data=json.loads((HERE/'audit-input.json').read_text())
    c.comments=lambda issue:data[str(issue)]
    audit=c.audit(c.CUTOVER)
    result['routing'] = dict(audit=audit,route=c.route('coordinator','REVIEW'))
else:
    result['routing'] = dict(audit='not replayed: connector snapshot absent',
        route=c.route('coordinator','REVIEW'))
print(json.dumps(result,indent=2))
