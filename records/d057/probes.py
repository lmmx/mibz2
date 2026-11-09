import importlib.util,itertools,json,re,subprocess
from pathlib import Path
p=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('coord',p/'coord.py')
c=importlib.util.module_from_spec(spec);spec.loader.exec_module(c)
snap=json.loads((p/'inboxes.json').read_text())
c.comments=lambda n:snap[str(n)]
audit=c.audit(c.CUTOVER)
assert not [x for x in audit if not x['resolved']]
routing={'audit':audit,'route':c.route('coordinator','REVIEW')}
(p/'routing.json').write_text(json.dumps(routing,indent=2)+'\n')
def code(s):
    return '\n'.join(l for l in s.splitlines() if not l.lstrip().startswith('//') and l.strip())
assert code((p/'read-base.rs').read_text())==code((p/'read-final.rs').read_text())
note=(p/'note.md').read_text().split('## D053 repairs',1)[1]
totals={}
for label,end in [('A','Corrected reach, pass B'),('B','The admission bound')]:
    block=note.split('Corrected reach, pass '+label,1)[1].split(end,1)[0]
    rows=[]
    for l in block.splitlines():
        if l.startswith('| ') and not any(x in l for x in ['candidate','**','---']):
            cols=[x.strip() for x in l.strip('|').split('|')]
            if len(cols)==11: rows.append([int(x) for x in cols[1:]])
    sums=[sum(row[i] for row in rows) for i in range(9)]
    # Joint build/walk lower bound, from per-candidate marginals (no pairing assumed).
    joint=sum(max(0,r[2]+r[3]-r[0]) for r in rows)
    totals[label]={'candidates':len(rows),'sums_excluding_maximum':sums,'joint_build_walk_lower_bound':joint,'max_queue_by_candidate':[r[-1] for r in rows]}
assert totals['A']['sums_excluding_maximum'][0]==7010
assert totals['A']['sums_excluding_maximum'][2:4]==[4909,4909]
assert totals['B']['sums_excluding_maximum'][0]==3694
assert totals['B']['sums_excluding_maximum'][2:4]==[2603,2603]
check=subprocess.run(['patch','--dry-run','--batch','-p1','-i','../admission.patch'],cwd=p/'a',text=True,capture_output=True)
assert check.returncode==0,check.stdout+check.stderr
valid=bad=fenced=fenced_bad=0
for seq in itertools.permutations(['markA','returnA','exitA','takeA','resetB','takeB']):
    at={v:i for i,v in enumerate(seq)}
    if not (at['markA']<at['returnA']<at['takeA']<at['resetB']<at['takeB'] and at['markA']<at['exitA']):continue
    valid+=1
    wrong=at['takeA']<at['exitA'] # lost to A, or published into B
    bad+=wrong
    if at['exitA']<at['takeA']:
        fenced+=1;fenced_bad+=wrong
assert bad>0 and fenced>0 and fenced_bad==0
results={'scope':'independent source comparison, table arithmetic, overlay dry-run, finite publication-order model; no Rust decoder execution','reader_runtime_code_identical_to_3e201246':True,'tables':totals,'admission_overlay_dry_run':check.stdout,'publication_model':{'orders':valid,'unfenced_incomplete_or_misattributed':bad,'orders_with_true_completion_fence':fenced,'fenced_bad':fenced_bad,'premise':'exit publishes TLS; fence observes completion. This model does not prove procfs enumeration.'},'native_probe':{'compiler':'gcc -std=c11 -O2 -Wall -Wextra -Werror -pthread','built':True,'status':'unavailable, not PASS','exit':77,'stderr':'opendir /proc/self/task: No such file or directory','note':'First attempt asserted on missing procfs; rerun changed diagnostic only. No control result obtained.'}}
(p/'results.json').write_text(json.dumps(results,indent=2)+'\n')
print(json.dumps(results))
