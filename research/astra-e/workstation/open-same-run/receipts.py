"""Generate separate evaluator receipts for the frozen scalar input and builtin ungated Open."""
import csv
import json
from pathlib import Path
import subprocess
import tarfile
from audit import ARMS, BUNDLES, CAND, HERE, MASTER, PATH_REV, PRINTED, RECIPE, WORKERS, sha

ROOT = Path(subprocess.check_output(['git','rev-parse','--show-toplevel'],text=True).strip())
SOURCES = ROOT/'research/astra-e/results'

def source(rev,path):
    return subprocess.check_output(['git','show',rev+':'+path],cwd=ROOT)
def receipt():
    with (HERE/'contrasts.csv').open() as f:
        rows=list(csv.DictReader(f))
    with tarfile.open(HERE/'measured.tar.gz') as tar:
        m=next(m for m in tar if m.name.endswith('/scorer-blobs.txt'))
        blobs=tar.extractfile(m).read().decode().splitlines()
    # The workstation projected bench/moonshot from master, leaving the external journal
    # source at the recipe commit. Neither of the two Open controls was the candidate entry.
    selector='bench/moonshot/src/arms.rs'
    main='bench/moonshot/src/main.rs'
    journal='docs/journal/2026-11-18-open-return/open_return.rs'
    for path in (selector,main):
        blob=subprocess.check_output(['git','rev-parse',MASTER+':'+path],cwd=ROOT,text=True).strip()
        assert any(blob in l and l.endswith('\t'+path) for l in blobs)
    selector_text=source(MASTER,selector).decode()
    assert '"open" => $f::<$crate::arms::Open>' in selector_text
    assert 'pub type Open = crate::open_return::Open<16, true, true>;' in selector_text
    assert ('../../../'+journal) in source(MASTER,main).decode()
    old=json.loads((SOURCES/'v1p2-cap-batch-scalar-confirm/i9-receipt.json').read_text())['scalar_i9_run']
    assert old['candidate_source_sha256']==sha(source(RECIPE,CAND))
    for arm,path,record,selector_info in [
        ('candidate',CAND,'v1p2-cap-batch-scalar-confirm',{'head':old['head']}),
        ('open',journal,'builtin-ungated-open',{'head':RECIPE,'builtin_arm':'open',
          'kernel_type':'crate::open_return::Open<16, true, true>',
          'arm_dispatch':selector,'arm_dispatch_sha256':sha(source(MASTER,selector)),
          'module_declaration':main,'module_declaration_sha256':sha(source(MASTER,main))})]:
        target={r['workers']:{'n':int(r['n']), 'geomean':float(r['wall']), 'lo':float(r['wall_lo']),
                           'hi':float(r['wall_hi']),'peak_rss_ratio':float(r['median_peak_ratio']),
                           'peak_rss_geomean':float(r['rss']),'peak_rss_lo':float(r['rss_lo']),
                           'peak_rss_hi':float(r['rss_hi']),'arm_max_kib':int(r['arm_max_kib']),
                           'reference_max_kib':int(r['ref_max_kib'])}
                for r in rows if r['arm']==arm and r['ref']=='lazy'}
        assert list(target)==[str(w) for w in WORKERS]
        obj=dict(id='run-20261119T083507Z',arm=arm,evidence_commit=BUNDLES[1][1],
                 artifact=str((HERE/'measured.tar.gz').relative_to(ROOT)),artifact_sha256=BUNDLES[1][3],
                 recipe_commit=RECIPE,selected=path,candidate_source_sha256=sha(source(RECIPE,path)),
                 machine='Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz',scorer_path_revision=PATH_REV,
                 printed_scorer_version='scorer v1 at '+PRINTED,**selector_info,
                 candidate_over_reference={'master LazyCells':target})
        key='open_same_run_'+('scalar' if arm=='candidate' else 'builtin_ungated_open')+'_i9_run'
        folder=SOURCES/record;folder.mkdir(exist_ok=True)
        (folder/'open-same-run-i9-receipt.json').write_text(json.dumps({key:obj},indent=2)+'\n')
if __name__=='__main__':
    receipt()
