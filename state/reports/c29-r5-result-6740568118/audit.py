"""Untimed independent arithmetic and raw admission checks for artifact 13671533599.

Usage: python audit.py ARTIFACT_ZIP
No reporter imports; no native decoder, filtering, or replacement observations.
"""
import hashlib
import json
import math
from pathlib import Path
import statistics as st
import sys
import zipfile

data = Path(sys.argv[1]).read_bytes()
assert hashlib.sha256(data).hexdigest() == 'b89bdfff7050d6b03678334963cef05c8e31170f1cc081429db159507dd8fc44'
z = zipfile.ZipFile(sys.argv[1])
def read(name):
    return z.read(name).decode()
def obj(name):
    return json.loads(read(name))
ident = obj('c29-identity.json')
assert ident['head'] == 'f693e0d6b7429a8325dbea95979c147778d0c060'
assert ident['schema'] == 'c29_checkpoints/identity/4'
assert ident['tree_clean'] and ident['features'] == [] and ident['profile'] == 'release'
for key, digest in [('binary', 'fdd83fecaed06e60b42bfdc47709464597df11b6226c1cecba85137fe4aeb334'), ('corpus', '82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915')]:
    assert ident[key + '_sha256_before'] == ident[key + '_sha256_after'] == digest
before, after = obj('c29-env.json'), obj('c29-env-after.json')
assert ident['env'] == before and ident['env_after'] == after
configs = []
for env in (before, after):
    assert env['avx2'] is True and env['cpus_allowed'] == env['available_parallelism'] == 4
    assert env['cpus_allowed_ids'] == [0, 1, 2, 3]
    cg = env['cgroup']
    assert cg['ns']['cgroup']['text'] == 'cgroup:[4026531835]'
    assert cg['mount']['root'] == '/' and cg['mount']['fstype'] == 'cgroup2'
    assert cg['membership']['text'].strip() == '0::/system.slice/hosted-compute-agent.service'
    assert [x['path'] for x in cg['levels']] == ['/system.slice/hosted-compute-agent.service', '/system.slice', '/']
    for level in cg['levels'][:-1]:
        assert level['files']['cpu.max']['text'].split() == ['max', '100000']
    assert cg['levels'][-1]['files']['cpu.max'] == {'missing': True}
    assert cg['levels'][-1]['files']['cgroup.type'] == {'missing': True}
    assert cg['gaps'] == [] and cg['coverage'] == 'complete' and cg['state'] == 'unlimited'
    # Compare raw configuration, excluding cumulative accounting and the spin probe.
    config = json.loads(json.dumps(env))
    config.pop('runq')
    for level in config['cgroup']['levels']:
        level['files'].pop('cpu.stat')
    configs.append(config)
assert configs[0] == configs[1]
assert ident['cgroup_stable'] == {'stable': True, 'differences': []}
assert int(read('c29-nproc.txt')) == 4 and ident['ab_run_status'] == 0
rows = [json.loads(s) for s in read('c29-runs.jsonl').splitlines() if s.strip()]
assert len(rows) == 80
assert all(r['exit'] == 0 and r['out'] == 209700196 and r['hash'] == 'a8830caee5c511a6' for r in rows)
assert all(r['avx2'] is True and r['cpus_allowed'] == r['available_parallelism'] == 4 and r['cpus'] == 'all' for r in rows)
out = {}
for n in (9, 3):
    arms = []
    for arm in ('mibz2-pipe3-both-c29', 'mibz2-pipe3-both'):
        subset = [r for r in rows if r['threads'] == n and r['arm'] == arm]
        assert len(subset) == 20 and {r['round'] for r in subset} == set(range(20))
        arms.append({r['round']: r for r in subset})
    cell = {}
    for metric in ('wall', 'cpu'):
        logs = [math.log(arms[0][i][metric] / arms[1][i][metric]) for i in range(20)]
        mu = st.mean(logs)
        half = 2.093024054408263 * st.stdev(logs) / math.sqrt(20)
        cell[metric] = [math.exp(mu), math.exp(mu-half), math.exp(mu+half)]
    cell['rss_median_kb_c29_control'] = [st.median(r['rss_kb'] for r in a.values()) for a in arms]
    wall, cpu = cell['wall'], cell['cpu']
    cell['threshold_reading'] = 'LEAD' if wall[0] <= .98 and wall[2] < 1 and cpu[2] <= 1.02 else 'STOP' if wall[1] > .99 else 'INCONCLUSIVE'
    cell['primary'] = n == 9
    out[n] = cell
print(json.dumps({'raw_admission_checks': 'PASS', 'cells': out}, indent=2))
