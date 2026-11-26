"""Third replay of the published cap-counter dry and measured data, independent of scorer code."""
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
import statistics
import subprocess
import tarfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[3]
ARMS = ('lazy', 'opend', 'edge', 'scalar', 'both')
WORKERS = range(7, 11)
PINS = {
    'dry': ('9eb578225e752d21cc64cb6d17a01d22919bc658', '20261119T060341Z', 'cap-counter-dry-9eb5782'),
    'measure': ('1b8d2a33a9976e0f8bc4d7cda2e6188dd6ab8ab5', '20261119T061036Z', 'cap-counter-i9-1b8d2a3'),
}


def read(mode):
    revision, stamp, local = PINS[mode]
    file = HERE.parent / 'evidence' / (local + '.tar.gz')
    published = 'astra-e-cap-counter/run-' + stamp + '.tar.gz'
    payload = file.read_bytes()
    def git_object(path):
        return subprocess.check_output(['git', '-C', ROOT, 'show', revision + ':' + path])
    assert payload == git_object(published)
    digest = hashlib.sha256(payload).hexdigest()
    assert git_object(published + '.sha256').decode().split() == [digest, Path(published).name]
    with tarfile.open(file) as bundle:
        names = [m.name.split('/', 1)[1] for m in bundle.getmembers()]
        assert len(names) == len(set(names))
        contents = {n: bundle.extractfile(m).read() for n, m in zip(names, bundle.getmembers(), strict=True)}
    assert contents['mode.txt'].decode().strip() == mode
    return digest, contents


def ratios(logs, chunk):
    grouped = [statistics.mean(logs[i:i+chunk]) for i in range(0, 40, chunk)]
    # Tabulated two-sided 95 percent critical values for df 39 or 3.
    critical = {1: 2.02269092003676, 10: 3.182446305284263}[chunk]
    center = statistics.mean(grouped)
    margin = critical * statistics.stdev(grouped) / math.sqrt(len(grouped))
    return dict(geomean=math.exp(center), lo=math.exp(center-margin), hi=math.exp(center+margin))


def close(calc, receipt):
    assert all(math.isclose(value, receipt[name], rel_tol=3e-12) for name, value in calc.items())


def main():
    dry_hash, dry = read('dry')
    run_hash, run = read('measure')
    for name in ('registration.txt', 'scorer-refresh.json', 'scorer-revision.txt', 'scorer-tree.txt',
                 'protocol-blobs.txt', 'sources.txt', 'roster.json', 'affinity.txt', 'toolchain.txt'):
        assert run[name] == dry[name], name
    assert run['registration.txt'].decode().strip() == 'a7e314a6f433c3ed0b1fb83ec5d9052f56be0f09'
    assert b'i9-10900K' in run['lscpu.txt'] and b'i9-10900K' in dry['lscpu.txt']
    assert run['affinity.txt'].decode().split()[-1] == '0-19'
    assert [l.split()[0] for l in dry['inputs.sha256'].splitlines()] == [l.split()[0] for l in run['inputs.sha256'].splitlines()]
    deltas = {}
    for a in ('edge', 'scalar', 'both'):
        fn = a+'-compiled.json'
        old, new = json.loads(dry[fn]), json.loads(run[fn])
        assert len(new)==33 and old.keys()==new.keys()
        deltas[a] = [k for k in old if old[k]!=new[k]]
        assert deltas[a] == (['research/astra-e/front-fusion/cap-batch/worker/kernel.rs'] if a=='scalar' else [])
        for data in (dry,run):
            *cases, verdict = map(json.loads,data['gate-'+a+'/check.jsonl'].splitlines())
            assert len(cases)==338 and all(t['pass'] and t['failure'] is None for t in cases)
            assert verdict==dict(arm='candidate',verdict='PASS',failures=0)
    for data in (dry,run):
        for label in ARMS:
            names=[f'gate-{label}-w{w}.jsonl' for w in WORKERS]
            if label in ('lazy','opend'):
                names.append(f'gate-{label}.jsonl')
            for name in names:
                *cases, verdict=map(json.loads,data[name].splitlines())
                assert len(cases)==98 and all(t['pass'] and t['failure'] is None for t in cases)
                assert verdict==dict(arm=label if label in ('lazy','opend') else 'candidate',verdict='PASS',failures=0)
    rows=list(map(json.loads,run['runs.jsonl'].splitlines()))
    expected=[]
    for r in range(40):
        for w in WORKERS:
            start=(r+w)%5
            cycle=ARMS[start:]+ARMS[:start]
            if (r+w)%10>=5:
                cycle=cycle[::-1]
            expected.extend((r,w,arm)for arm in cycle)
    assert [(row['round'],row['workers'],row['arm'])for row in rows]==expected
    for row in rows:
        arm=row['arm']
        assert row['out_bytes']==838800784
        assert (row['build'],row['compiled_arm']) == (('edge',arm) if arm in ('lazy','opend') else (arm,'candidate'))
        assert all(math.isfinite(row[k]) and row[k]>0 for k in ('wall_s','cpu_s','peak_rss_kb'))
    assert len(set(expected))==800
    positions=Counter((w,i%5,arm)for i,(_,w,arm)in enumerate(expected))
    assert len(positions)==100 and set(positions.values())=={8}
    index=dict(zip(expected,rows,strict=True))
    def values(w,arm,key):
        return [index[r,w,arm][key] for r in range(40)]
    def logs(w,reference,key):
        return [math.log(a)-math.log(b) for a,b in zip(values(w,'both',key),values(w,reference,key),strict=True)]
    published=json.loads(run['decision.json'])
    direct={}
    by_w={}
    for w in WORKERS:
        cell={}
        for ref in ARMS[:-1]:
            wall=ratios(logs(w,ref,'wall_s'),1)
            rss=ratios(logs(w,ref,'peak_rss_kb'),1)
            base=values(w,ref,'peak_rss_kb')
            candidate=values(w,'both','peak_rss_kb')
            out=published['workers'][str(w)][ref]
            close(wall,out['wall'])
            close(rss,out['rss'])
            assert (statistics.median(candidate)/statistics.median(base),max(candidate),max(base))==(
                out['median_peak_ratio'],out['max_peak_kb'],out['ref_max_peak_kb'])
            cell[ref]=wall
            if ref=='lazy':
                peak=dict(paired_upper=rss['hi']<=1,median=statistics.median(candidate)<=statistics.median(base),maximum=max(candidate)<=max(base))
                assert peak==published['peak_vs_lazy'][str(w)]
                cell['peak_ok']=all(peak.values())
                cell['stretch_ok']=wall['hi']<=.8 and cell['peak_ok']
        by_w[w]=cell
    for ref in ('edge','scalar'):
        series=[statistics.mean(group)for group in zip(*(logs(w,ref,'wall_s') for w in WORKERS),strict=True)]
        official=ratios(series,1)
        close(official,published['direct'][ref]['wall'])
        guards={str(w):by_w[w][ref]['hi']<=1.02 for w in WORKERS}
        assert guards==published['direct'][ref]['per_worker_guard']
        passed=official['hi']<1 and all(guards.values())
        assert passed==published['direct'][ref]['wall_screen_met']
        direct[ref]=dict(registered=official,whole_rotor_sensitivity=ratios(series,10),per_w_guards=guards,passed=passed)
    joint=all(x['passed'] for x in direct.values()) and all(c['peak_ok'] for c in by_w.values())
    stretch=all(c['stretch_ok'] for c in by_w.values())
    assert (joint,stretch)==(published['joint_incremental_met'],published['i9_criterion_met'])
    print(json.dumps(dict(dry_sha256=dry_hash,measure_sha256=run_hash,scorer=json.loads(run['scorer-refresh.json']),
                          compiled_differences=deltas,direct=direct,workers=by_w,
                          incremental=joint,stretch=stretch),indent=2))


if __name__=='__main__':
    main()
