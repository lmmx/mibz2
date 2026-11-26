"""Authenticate timed and dry artifacts and emit the cap-counter retrospective receipt."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import runpy
import subprocess
import sys
import tarfile
from counter_metrics import archive, require

HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[2]
PIN='a7e314a6f433c3ed0b1fb83ec5d9052f56be0f09'
DRY='9eb578225e752d21cc64cb6d17a01d22919bc658'
DRY_PATH='astra-e-cap-counter/run-20261119T060341Z.tar.gz'


def git(*args):
    return subprocess.check_output(['git','-C',ROOT,*args])


def checksum(path,rev,filename):
    require(re.fullmatch(r'[0-9a-f]{40}',rev) and
            re.fullmatch(r'astra-e-cap-counter/run-[0-9]{8}T[0-9]{6}Z\.tar\.gz',filename),'invalid evidence locator')
    data=path.read_bytes()
    digest=hashlib.sha256(data).hexdigest()
    require(data==git('show',rev+':'+filename),'archive differs from published evidence')
    require(git('show',rev+':'+filename+'.sha256').decode().split()[0]==digest,'published checksum disagreement')


def receipt(path,revision,filename):
    sys.path.insert(0,str(HERE/'counter'))
    verify=runpy.run_path(str(HERE/'counter/verify.py'))['verify']
    checksum(path,revision,filename)
    dry_path=HERE/'evidence/cap-counter-dry-9eb5782.tar.gz'
    checksum(dry_path,DRY,DRY_PATH)
    dry=verify(dry_path,PIN)
    timed=verify(path,PIN)
    actual,pairs=archive(path)
    require(timed['target_cpu'] and dry['target_cpu'] and dry['mode']=='dry' and timed['mode']=='measure',
            'incorrect target rehearsal or timing')
    require(dry['scorer_path']==timed['scorer_path'] and dry['reports']==timed['reports'],
            'timed scorer differs from the preregistered target dry')
    require(actual['archive_sha256']==timed['bundle_sha256'],'independent checksum')
    with tarfile.open(path) as z:
        by_name={m.name.split('/',1)[1]:m for m in z if m.isfile()}
        read=lambda name:z.extractfile(by_name[name]).read().decode()
        source=re.search(r'^both ([0-9a-f]{40}) (research/\S+)$',read('sources.txt'),re.M)
        head,entry=source.groups()
        model=re.search(r'^Model name:\s*(.+)$',read('lscpu.txt'),re.M).group(1).strip()
    ratios={}
    for label,ref in [('master LazyCells','lazy'),('master opend','opend'),('frozen edge','edge'),('frozen scalar','scalar')]:
        ratios[label]={}
        for p in pairs:
            if p['reference']==ref and p['treatment']=='both':
                ratios[label][str(p['workers'])]=dict(n=actual['rounds'],geomean=p['wall_s_ratio'],lo=p['wall_s_lo'],
                                                     hi=p['wall_s_hi'],peak_rss_ratio=p['median_rss_ratio'])
    out=dict(id=Path(filename).name.removesuffix('.tar.gz'),evidence_commit=revision,
             artifact=str(path.resolve().relative_to(ROOT)),artifact_sha256=actual['archive_sha256'],
             head=head,selected=entry,candidate_source_sha256=hashlib.sha256(git('show',head+':'+entry)).hexdigest(),
             recipe_commit=PIN,dry_evidence=DRY,scorer_path_revision=timed['scorer_path'],
             printed_scorer_version=timed['reports']['both'],machine=model,
             i9_criterion_met=actual['stretch'],incremental_criterion_met=actual['incremental'],
             candidate_over_reference=ratios)
    return {'cap_counter_i9_run':out}


if __name__=='__main__':
    p=argparse.ArgumentParser(description=__doc__)
    p.add_argument('archive',type=Path)
    p.add_argument('evidence_commit')
    p.add_argument('publication_path')
    a=p.parse_args()
    print(json.dumps(receipt(a.archive,a.evidence_commit,a.publication_path),allow_nan=False,indent=2))
