import sys,json,hashlib,base64,gzip
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE))
import replay
x=json.loads((HERE/'inputs.json').read_text())
e={k:base64.b64decode(v['content']) if v['encoding']=='base64' else v['content'].encode() for k,v in x['evidence'].items()}
checks={}
for line in e['SHA256SUMS'].decode().splitlines():
    h,p=line.split(); assert hashlib.sha256(e[p]).hexdigest()==h,p
    checks[p]=h
for line in e['SOURCE-SHA256SUMS'].decode().splitlines():
    h,p=line.split(); assert hashlib.sha256(x['sources'][p].encode()).hexdigest()==h,p
raw=gzip.decompress(e['miri.raw.log.gz'])
normalized=b'\n'.join(line.rstrip() for line in raw.split(b'\n'))
assert normalized==e['miri.log']
src=x['sources']['src/cells.rs'].splitlines()
assert 'assert_eq!(out, want);' in src[976]
assert 'assert_eq!(out, want);' in src[1026]
for row in json.loads(e['mutation-results.json'])['mutants']:
    log=e['mutation-'+row['name']+'.log'].decode()
    assert 'Executable unittests' in log,row
    if row['killed']:
        assert 'test result: FAILED.' in log or ('signal: 6, SIGABRT' in log and row['name']=='uncounted-cell'),row
    else: assert 'test result: ok.' in log,row
word=(b'a'*9+b'b'*9)*3+b'c'
col=bytes(word[(i-1)%len(word)] for i in sorted(range(len(word)),key=lambda i:word[i:]+word[:i]))
out=[]
for a,p in [(bytes([1])*16+bytes([0])*15,16),(col,0),(col,3),(bytes(b for b in col for _ in range(10)),0)]:
    trace=replay.model(a,p)
    row=dict(n=len(a),primary=p,**trace)
    if p==0:
        try: replay.model(a,p,omit_recopy=True)
        except AssertionError as er: row['omitted_recopy_failure']=str(er)
        else: raise AssertionError('mutation escaped')
    out.append(row)
result=dict(evidence_hashes=checks,source_hashes_verified=True,raw_normalizes_exactly=True,witnesses=out)
(HERE/'checks.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps(result,indent=2))
