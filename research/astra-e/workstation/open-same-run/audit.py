"""Validate both official bundles and independently reconstruct the preregistered i9 rule."""
import csv
import hashlib
import io
import itertools
import json
import math
from pathlib import Path
import statistics as st
import subprocess
import sys
import tarfile

HERE = Path(__file__).resolve().parent
RECIPE = '9f867e69e5e7fe33aaae569af94a447eb38cbe26'
MASTER = 'f6cfcb04a53c91d521ddb973427b4d53ad7548da'
RELEASE = '530ed404a014a414ef0b62adfd947fa14d4533e6'
PATH_REV = '75297a1d57b2bdf262c3a19f3064848dedf2c493'
PRINTED = '81585172f5fb9e7a59f330124779ba084fd6bf04'
ARMS = ['lazy','opend','open','candidate']
WORKERS = [7,8,9,10]
BUNDLES = [
    ('dry','f0c26c1b202bf9eeec76528e43731a4a8647c1ea','083215',
     '9e043b67e9e30c8c76237e29f3dd36e9415081e4c40c5703479a1b43ab8f1550'),
    ('measured','f71f153baaae2d28f0a61acc25a2129612eac9cc','083507',
     '083288e4788798e784c39e5a47f95405f1c94b384485fe4c46aaa575c471c30c')]
INPUTS = ['82f65ace0731962d04443bbda4cebfdccd336106049751060f477941c2c29915',
          '0b820d14299af637bf7f644d172f5efe91ad59e169cf2a50bd0962f5c89ce08e']
# Two-sided 95% Student critical value for the 16 matched rounds (df=15).
CRITICAL = 2.131449545559323
HISTORICAL = [0.870,0.880,0.876,0.859]
CAND = 'research/astra-e/front-fusion/cap-batch/worker/candidate.rs'
def git(*args):
    return subprocess.check_output(['git',*args],cwd=HERE)
def sha(data):
    return hashlib.sha256(data).hexdigest()
def plan():
    rows=[]
    for rnd in range(16):
        for w in WORKERS:
            phase=(rnd+w)%8
            cells=ARMS[phase%4:]+ARMS[:phase%4]
            if phase>=4:cells.reverse()
            rows.extend([rnd,w,arm] for arm in cells)
    return rows
def check_bundle(spec):
    label, rev, stamp, digest=spec
    name='run-20261119T'+stamp+'Z'
    path='astra-e/'+name+'.tar.gz'
    raw=(HERE/(label+'.tar.gz')).read_bytes()
    assert git('show',rev+':'+path)==raw
    assert sha(raw)==digest==git('show',rev+':'+path+'.sha256').decode().split()[0]
    with tarfile.open(fileobj=io.BytesIO(raw)) as tar:
        data={m.name.removeprefix(name+'/'):tar.extractfile(m).read() for m in tar if m.isfile()}
    text=lambda k:data[k].decode().strip()
    read=lambda k:json.loads(data[k])
    assert text('commits.txt').splitlines()==[RECIPE,MASTER,RELEASE]
    assert text('mode.txt')==('dry' if label=='dry' else 'measure')
    assert text('cpus-online.txt')=='0-19' and text('affinity.txt').split()[-1]=='0-19'
    assert 'i9-10900K' in text('lscpu.txt') and read('scorer-info.json')['cpus']==20
    assert text('scorer-revision.txt')==PATH_REV
    r=read('scorer-refresh.json')
    assert r['selected_build']=='fresh' and r['selected_projection']==MASTER
    assert r['selected_scorer_tree']==git('rev-parse',MASTER+':bench/moonshot').decode().strip()
    assert 'scorer v1 at '+PRINTED in text('scorer-report.txt')
    assert read('candidate-gate/summary.json')['scorer']==dict(version='v1',commit=PRINTED)
    for l in text('sources.sha256').splitlines():
        expected, file=l.split()
        assert sha(git('show',(MASTER if file.startswith('bench/') else RECIPE)+':'+file))==expected
    assert [l.split()[0] for l in text('inputs.sha256').splitlines()]==INPUTS
    assert git('diff',RECIPE+'^',RECIPE,'--','src',CAND,'research/astra-e/prepare.sh')==b''
    assert read('roster.json')==plan()
    gates={'candidate-gate/check.jsonl':('candidate',338)}
    gates.update({'gate-'+a+'.jsonl':(a,98) for a in ARMS[:-1]})
    if label=='dry':
        cov=read('dry-coverage.json')
        assert cov==dict(target_machine=True,gate_arms=['candidate'],target_worker_counts_tested=WORKERS,
                         all_target_worker_counts_tested=True)
        gates.update({f'gate-candidate-w{w}.jsonl':('candidate',98) for w in WORKERS})
        assert 'runs.jsonl' not in data
    for file,(arm,count) in gates.items():
        lines=[json.loads(l) for l in text(file).splitlines()]
        assert len(lines)==count+1 and lines[-1]=={'arm':arm,'verdict':'PASS','failures':0}
        assert all(l['pass'] and l['failure'] is None for l in lines[:-1])
    return data

def paired(a,b):
    logs=[math.log(x/y) for x,y in zip(a,b,strict=True)]
    assert len(logs)==16
    m=st.mean(logs); half=CRITICAL*st.stdev(logs)/4
    return dict(n=len(logs),geomean=math.exp(m),lo=math.exp(m-half),hi=math.exp(m+half))
def close(actual,expected):
    assert math.isclose(actual,expected,rel_tol=2e-12,abs_tol=2e-12),(actual,expected)
def classify(interval,low,high):
    if interval['hi']<1:return low
    if interval['lo']>1:return high
    return 'unresolved'
def run():
    dry,measured=map(check_bundle,BUNDLES)
    dry_binary=dry['binaries.sha256'].decode().split()[0]
    measured_binary=measured['binaries.sha256'].decode().split()[0]
    assert dry_binary=='ab810879bf036236d23e1ddcc5d2ca05e4d0a253d6650b7427430e438e63a8bc'
    assert measured_binary=='e9bee08282f3ced5ba61e2ee6ca0417e9f4db52c5852bb30bfdd38b7136a8293'
    rows=[json.loads(l) for l in measured['runs.jsonl'].splitlines()]
    assert [[r['round'],r['workers'],r['arm']] for r in rows]==plan()
    assert all(r['out_bytes']==838800784 and all(math.isfinite(r[k]) and r[k]>0 for k in
               ['wall_s','peak_rss_kb','cpu_s']) for r in rows)
    official=json.loads(measured['decision.json'])
    comparisons=[]; armstats=[];size_verdicts=[];wall_verdicts=[];checks=[];test=[]
    for i,w in enumerate(WORKERS):
        per={a:[r for r in rows if r['workers']==w and r['arm']==a] for a in ARMS}
        def values(a,metric):return [r[metric] for r in per[a]]
        for arm in ARMS:
            peaks=values(arm,'peak_rss_kb');walls=values(arm,'wall_s')
            armstats.append([w,arm,len(peaks),st.median(walls),st.median(peaks),max(peaks)])
        for ref,arm in itertools.combinations(ARMS,2):
            wall=paired(values(arm,'wall_s'),values(ref,'wall_s'))
            rss=paired(values(arm,'peak_rss_kb'),values(ref,'peak_rss_kb'))
            cpu=paired(values(arm,'cpu_s'),values(ref,'cpu_s'))
            med=st.median(values(arm,'peak_rss_kb'))/st.median(values(ref,'peak_rss_kb'))
            maxima=[max(values(a,'peak_rss_kb')) for a in (arm,ref)]
            comparisons.append([w,arm,ref,wall['n'],*(wall[k] for k in ('geomean','lo','hi')),
                                *(rss[k] for k in ('geomean','lo','hi')),med,*maxima,
                                *(cpu[k] for k in ('geomean','lo','hi'))])
            if arm=='candidate':
                expected=official['workers'][str(w)][ref]
                for metric,series in [('wall',wall),('rss',rss)]:
                    for k,v in series.items():close(v,expected[metric][k])
                close(med,expected['median_peak_ratio'])
                assert maxima==[expected['max_peak_kb'],expected['ref_max_peak_kb']]
            if ref=='open':
                wall_verdicts.append(classify(wall,'faster','slower'))
                size_verdicts.append(classify(rss,'smaller','larger'))
            if ref=='lazy' and arm in ('open','candidate'):
                old=0.919 if arm=='open' else HISTORICAL[i]
                checks.append(dict(workers=w,arm=arm,historical=old,observed=wall['geomean'],
                                   within=abs(wall['geomean']-old)<=0.03))
            if ref=='lazy' and arm=='candidate':
                decision=wall['hi']<=0.8 and rss['hi']<=1 and med<=1 and maxima[0]<=maxima[1]
                test.append(decision)
                assert official['workers'][str(w)]['target_met']==decision
    assert all(test)==official['i9_criterion_met']
    names=['workers','arm','ref','n','wall','wall_lo','wall_hi','rss','rss_lo','rss_hi',
           'median_peak_ratio','arm_max_kib','ref_max_kib','cpu','cpu_lo','cpu_hi']
    for file,header,table in [('contrasts.csv',names,comparisons),('arms.csv',
            ['workers','arm','n','median_wall_s','median_peak_kib','max_peak_kib'],armstats)]:
        with (HERE/file).open('w') as f:
            wr=csv.writer(f,lineterminator='\n');wr.writerow(header);wr.writerows(table)
    verdict=dict(dry_sha256=BUNDLES[0][3],measured_sha256=BUNDLES[1][3],workers=WORKERS,rounds=16,
                 runs=len(rows),candidate_over_open_wall=wall_verdicts,candidate_over_open_rss=size_verdicts,
                 reproduction=checks,stretch_by_worker=test)
    print(json.dumps(verdict,indent=2))
if __name__=='__main__':run()
