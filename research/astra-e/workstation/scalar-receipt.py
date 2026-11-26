"""Construct the official retrospective input after authenticating the independent target archive."""
from pathlib import Path
import hashlib
import json
import re
import runpy
import subprocess
import tarfile

here=Path(__file__).resolve().parent
root=here.parents[2]
archive=here/'evidence/scalar-i9-1ac706d.tar.gz'
verify=runpy.run_path(str(here/'scalar-verify.py'))['verify']
result=verify(archive)
evidence='1ac706d628cd875092bfcbcf50e72f526018ee82'
path='astra-e-scalar/run-20261119T045900Z.tar.gz'
def git(*args):return subprocess.check_output(['git','-C',root,*args])
assert git('show',f'{evidence}:{path}')==archive.read_bytes()
assert git('show',f'{evidence}:{path}.sha256').decode().split()[0]==result['sha256']
with tarfile.open(archive) as z:
    files={item.name.split('/',1)[1]:item for item in z}
    def read(name):return z.extractfile(files[name]).read().decode()
    inputs=read('matched-sources.txt')
    head,entry=re.search(r'^scalar (\w+) (\S+)$',inputs,re.M).groups()
    previous=re.search(r'^measured edge (\w+)',inputs,re.M).group(1)
    model=re.search(r'^Model name:\s*(.+)$',read('lscpu.txt'),re.M).group(1).strip()
cells=result['decision']['workers']
ratios={}
for label,ref in [('master LazyCells','lazy'),('master opend','opend'),('frozen edge','predecessor')]:
    ratios[label]={str(w):dict(**cell[ref]['wall'],peak_rss_ratio=cell[ref]['median_peak_ratio']) for w,cell in cells.items()}
receipt=dict(id=Path(path).name.removesuffix('.tar.gz'),evidence_commit=evidence,artifact=str(archive.relative_to(root)),
    artifact_sha256=result['sha256'],head=head,recipe_commit=result['recipe'],predecessor_commit=previous,selected=entry,
    candidate_source_sha256=hashlib.sha256(git('show',f'{head}:{entry}')).hexdigest(),machine=model,
    scorer_path_revision=result['scorer_path_revision'],printed_scorer_version=result['scorer_report'],
    i9_criterion_met=result['decision']['i9_criterion_met'],incremental_criterion_met=result['decision']['incremental_primary']['criterion_met'],
    candidate_over_reference=ratios)
print(json.dumps(dict(scalar_i9_run=receipt),indent=2))
