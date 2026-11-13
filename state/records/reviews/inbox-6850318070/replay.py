"""Saved JSON/hash replay only. Does not execute decoders or decompress inputs."""
import importlib.util,json,pathlib,hashlib,collections,statistics
p=pathlib.Path('live-raw/20261113T101816Z')
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
spec=importlib.util.spec_from_file_location('frozen','live-driver.py')
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
m=json.loads((p/'attempt/manifest.json').read_text())
assert m['schedule']==d.schedule()
assert m['build']==json.loads((p/'bin/build.json').read_text())
assert sha(pathlib.Path('live.tar.gz'))=='23c8712e1d1f2a64bbbb2f10232db7f21a8a8499826ac8f9ac42a2bd27374414'
for k,h in m['build']['binaries'].items(): assert sha(p/'bin'/k)==h
for k,h in m['input_sha256'].items(): assert sha(p/'inputs'/(k+'.bz2'))==h
for local,key in [('live-driver.py','driver.py'),('live-prepare.py','prepare.py'),('live-trace.rs','src/trace.rs')]:
    assert sha(pathlib.Path(local))==m['build']['harness'][key]
assert sha(pathlib.Path('live-source.rs'))==m['build']['source']['original']['src/cells.rs']
rows=[json.loads(l) for l in (p/'attempt/runs.jsonl').read_text().splitlines()]
analysis=d.analyse(rows)
assert json.dumps(analysis,indent=2)+'\n'==(p/'attempt/analysis.json').read_text()
checks={}
missing=[]
for i,job in enumerate(d.schedule()):
    q=p/'attempt'/f"{i:03d}-{job['corpus']}-{job['arm']}.json"
    r=json.loads(q.read_text())
    if job['mode']=='check':
        checks[(job['corpus'],job['arm'])]=r
        if job['arm']!='oracle': d.reconcile(r); assert r['error'] is None and not r['timed']
        b=q.with_suffix('.bytes')
        if b.exists(): assert sha(b)==r['sha256'] and b.stat().st_size==r['bytes']
        else: missing.append(b.name)
    else:
        s=json.loads(q.with_suffix('.resources').read_text())
        assert s['returncode']==0 and s['validation']=='accepted' and s['status']=='exited'
        row=rows[i-9]
        assert all(row[k]==v for k,v in r.items())
        assert all(row[k]==v for k,v in job.items())
        assert s['job']==dict(index=i,**job)
        assert all(row[k]==v for k,v in s['resources'].items())
        d.reconcile(row)
for corpus in d.CORPORA:
    oracle=checks[corpus,'oracle']
    for arm in ['baseline','cells']:
        check=checks[corpus,arm]
        assert check['sha256']==oracle['sha256'] and check['bytes']==oracle['bytes']
        assert all(r['hash']==check['hash'] and r['bytes']==check['bytes'] and r['error'] is None for r in rows if r['corpus']==corpus and r['arm']==arm)
out={'verified':{'measured':len(rows),'checks':len(checks),'binaries':4,'inputs':3,'analysis_byte_identical':True,'missing_byte_files':missing,'oracle_decompression_executed':False},'analysis':analysis,'census':{},'wall_ms':{}}
for corpus in d.CORPORA:
    traced=[r for r in rows if r['corpus']==corpus and r['arm']=='cells' and r['trace']]
    assert all(r['blocks']==traced[0]['blocks'] and r['counters']==traced[0]['counters'] for r in traced)
    r=traced[0]; counts=collections.Counter()
    for b in r['blocks']: counts.update(b['counters'])
    outcomes=collections.Counter(b['outcome'].split('(')[0] for b in r['blocks'])
    out['census'][corpus]={'blocks':len(r['blocks']),'outcomes':dict(outcomes),'counter_sums':dict(counts),'global':r['counters']}
    out['wall_ms'][corpus]={f'{arm}-{trace}':[r['wall_ns']/1e6 for r in rows if r['corpus']==corpus and r['arm']==arm and r['trace']==trace] for arm in ['baseline','cells'] for trace in [False,True]}
ex=analysis['pattern']['cost_reconciliation']['exclusive']
out['pattern_static_ceiling']={'rules':out['census']['pattern']['counter_sums']['pos_initialized_bytes']//4,'whole_baseline_on_ms':sum(x['baseline_mean_ns'] for x in ex.values())/1e6,'whole_cells_on_ms':sum(x['cells_mean_ns'] for x in ex.values())/1e6,'inversion_new_writer_break_even_ms':(ex['inversion']['baseline_mean_ns']-sum(ex[k]['cells_mean_ns'] for k in ['inversion','attempt','induction','order']))/1e6}
pathlib.Path('live-replay.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({'verified':out['verified'],'pattern_static_ceiling':out['pattern_static_ceiling']},indent=2))
