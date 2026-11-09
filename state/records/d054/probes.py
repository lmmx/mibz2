import json,re,hashlib
from pathlib import Path
root=Path(__file__).parent
p=root/'docs/journal/2026-11-09-run-fed-leader'
out={}
# Independent finite-domain ownership check: allocation entry and u64 bit access.
cases=0
for shares in range(1,65):
    for helpers in range(shares,81):
        access=[j for j in range(helpers) if j<shares]
        assert access==list(range(shares))
        assert all(j<64 for j in access)
        assert sum(1<<j for j in access)==(1<<shares)-1
        cases+=1
out['ownership_domains']=cases
out['old_witness']={'team_k':66,'helpers':65,'shares':64,'j':64,'entry_size':3072,'allocation_bytes':64*3072,'reference_offset':64*3072}
# C29 two 32-byte stores: test physical, not logical, footprint.
good=bad=0
first=None
for cap in range(1,130):
    for n in range(cap):
        for length in range(1,min(64,cap-n)+1):
            if n+64<=cap:
                assert n+32<=cap and n+32+32<=cap
                good+=1
            elif n<cap:
                bad+=1
                if first is None:first={'cap':cap,'n':n,'length':length,'physical_end':n+64}
out['vector_valid_cases']=good
out['mutant_unsafe_cases']=bad
out['mutant_first_witness']=first
out['fuzz']=[]
for f in sorted((p/'fuzz').glob('*.txt')):
    s=f.read_text()
    head=re.search(r'(\d+) cases in (\d+) s \(seed (\d+), mode run-fed\), (\d+) failures',s)
    rows=re.findall(r'^\s*(\d+)  mibz2 run-fed (.+): rule (\d+), team records (\d+) column (\d+), small records (\d+) column (\d+), cases with team records (\d+), rebuilt (\d+)$',s,re.M)
    assert len(rows)==11 and sum(int(r[0]) for r in rows)==int(head[1])
    assert int(head[4])==0 and all(int(r[8])==0 for r in rows)
    for r in rows:
        if not r[1].startswith(('Control','RegControl')): assert int(r[3])>0 and int(r[5])>0
    out['fuzz'].append({'file':f.name,'cases':int(head[1]),'seed':int(head[3]),'candidate_rows':len(rows),'team_records':sum(int(r[3]) for r in rows),'small_records':sum(int(r[5]) for r in rows),'sha256':hashlib.sha256(f.read_bytes()).hexdigest()})
out['m12_native']={}
for f in sorted((p/'mutants/logs').glob('M12r-*.log')):
    m=re.search(r'test result: FAILED\. (\d+) passed; (\d+) failed;',f.read_text())
    out['m12_native'][f.name]={'passed':int(m[1]),'failed':int(m[2])}
out['scope']='Python arithmetic and archived-log reconciliation only; no Rust/Miri execution or timing here.'
(root/'results.json').write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps(out,indent=2))
