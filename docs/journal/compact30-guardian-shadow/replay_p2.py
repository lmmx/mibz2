#!/usr/bin/env python3
"""Independent scanner archive sums and evidence inventory; no constructor runs.

python replay_p2.py --repo /path/to/mibz2 --out /scratch/p2
"""
import argparse,collections,gzip,hashlib,json,pathlib,re,subprocess,sys
ap=argparse.ArgumentParser(description=__doc__)
ap.add_argument('--repo',type=pathlib.Path,required=True)
ap.add_argument('--out',type=pathlib.Path,required=True)
a=ap.parse_args()
if not __debug__: ap.error('Assertions must be enabled')
a.out.mkdir(parents=True,exist_ok=True)
S='7f74e9f30e31af33b0d66fec3fe89af1758307d9'
A='0ac2e67e6826b9f2b1940cb876c3003e7a022144'
G='931640523226053b758ebc0c2c9fd4eac61128b3'
E='docs/journal/2026-11-04-compact30-scanner-prototype/'
manifest={}
def get(path,pin=S):
    b=subprocess.check_output(['git','-C',str(a.repo),'show',pin+':'+path])
    manifest[pin+':'+path]=dict(pin=pin,path=path,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),git_blob=hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest())
    return b
def listing(path):
    return {int(m[1],16):m[2] for l in get(path).decode().splitlines() if (m:=re.match(r'\s*([0-9a-f]+):\s+(.*)',l))}
for f,p in [('ctor2.rs','8beffe28bec9b539517b442350d8a3fda62164c8'),('ctor3.rs','dbf5139e12ae60b5f5bff40059b88d65ddccea41')]:
    assert get('examples/c30walk/'+f)==get('examples/c30walk/'+f,p)
source=get('examples/c30walk/ctor2s.rs').decode()
for path in ['examples/c30walk.rs',E.rstrip('/')+'.md','docs/journal/2026-11-04-compact30-scanner-cleanup-handoff.md']:
    get(path)
# Include every pinned scanner evidence file, not summaries alone.
paths=subprocess.check_output(['git','-C',str(a.repo),'ls-tree','-r','--name-only',S,'--',E]).decode().splitlines()
for path in paths: get(path)
for path in ['docs/journal/compact30-channel/GUARDIAN.md','docs/journal/compact30-channel/PROTOCOL.md','docs/journal/compact30-channel/REVIEWER.md','docs/journal/2026-11-04-compact30-research-charter.md']:
    get(path,G)
get('docs/journal/2026-11-03-compact30-constructor-v2-reassessment.md','d9f153a023b99de46895d049fe296204ff95ac9f')
for path in ['docs/journal/compact30-research/instruction-ledger.md','docs/journal/compact30-research/reviews/scanner-146/report.md','docs/journal/compact30-research/reviews/scanner-146/safety/SAFETY-REVIEW.md','docs/journal/compact30-research/reviews/scanner-146/complete-evidence-manifest.json']:
    get(path,A)

raw=gzip.decompress(get(E+'callgrind/per-instruction.callgrind.gz')).decode()
names={};own=collections.Counter();edges=collections.defaultdict(collections.Counter)
addresses=collections.defaultdict(set);fn=callee=None;edge=False;addr=0
for l in raw.splitlines():
    if l.startswith('positions:'): positions=l.split()[1:]
    elif l.startswith('events:'): events=l.split()[1:]
    elif l.startswith('summary:'): summary=int(l.split()[1])
    elif l.startswith(('fn=','cfn=')):
        key,value=l.split('=',1);m=re.fullmatch(r'\((\d+)\)(?: (.*))?',value)
        if m:
            if m[2] is not None: names[m[1]]=m[2]
            value=names[m[1]]
        if key=='fn': fn=value
        else: callee=value
    elif l.startswith('calls='): edge=True
    elif l and (l[0].isdigit() or l[0] in '+-*'):
        fields=l.split();x=fields[0]
        if x!='*': addr=addr+int(x) if x[0] in '+-' else int(x,0)
        assert events==['Ir']
        assert len(fields)==len(positions)+len(events)
        ir=int(fields[len(positions)])
        if edge: edges[fn][callee]+=ir
        else: own[fn]+=ir;addresses[fn].add(addr)
        edge=False
assert sum(own.values())==summary
b2='<c30walk::ctor2::Ctor2>::build::<false>'
scanner='<c30walk::ctor2s::Ctor2s>::build::<false>'
code=listing(E+'listings/ctor2s-build-false.asm')
baseline=listing(E+'listings/v2-build-false-same-binary.asm')
assert addresses[scanner]<=code.keys() and addresses[b2]<=baseline.keys()
path_counts={name:sum(lo<=x<=hi for x in code) for name,lo,hi in [('fresh',0x41a58,0x41a60),('lookahead',0x41aa0,0x41ac9),('row1',0x41b80,0x41baa),('row2',0x41c40,0x41c67),('tail1',0x42520,0x4254a),('tail2',0x425a0,0x425c4)]}
assert list(path_counts.values())==[4,11,12,12,12,11]
attempts=[len(listing(E+f'attempt-{n}/ctor2s-build-false.asm')) for n in [1,2,3]]
miri={}
for name in ['tiny','tails','align','phase','held','endrun','epoch','malformed']:
    log=get(E+'miri/miri-'+name+'.console').decode()
    assert re.search(r'^exit 0$',log,re.M)
    miri[name]=int(re.search(r'--miri \w+: (\d+) cases,',log)[1])
assert sum(miri.values())==105
mutations={}
for name,n in [('M1',32),('M2',16)]:
    log=get(E+'mutation-'+name+'.console').decode()
    assert f'small n {n} sigma' in log and 'counted build rejected valid counts' in log and 'exit 101' in log
    mutations[name]=dict(rejects_valid_input_rows=n,exit=101)
synthetic=get(E+'synthetic.console').decode()
matched=re.search(r'(\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact \(full\)',synthetic)
assert matched
full=dict(zip(['cases','arrays','walks','restorations'],map(int,matched.groups())))
assert 'malformed counts: 3995 rejected, 7 accepted' in synthetic
phases=sorted({16*k%30 for k in range(15)})
assert phases==list(range(0,30,2)) and 61440%30==0
# Reproduce the packaged parser defect in scratch using the uncompressed input.
(a.out/'raw-profile.txt').write_text(raw)
(a.out/'original-cgparse.py').write_bytes(get(E+'callgrind/cgparse.py'))
bad=subprocess.run([sys.executable,str(a.out/'original-cgparse.py'),str(a.out/'raw-profile.txt')],capture_output=True,text=True)
assert bad.returncode!=0 and "invalid literal for int() with base 10: '*'" in bad.stderr
(a.out/'parser-failure.txt').write_text(bad.stderr)
delta={f:edges[scanner][f]-edges[b2][f] for f in sorted(edges[b2].keys()|edges[scanner].keys())}
result=dict(pin=S,baseline_pin='8beffe28bec9b539517b442350d8a3fda62164c8',
    profile_positions=positions,profile_events=events,profile_global_Ir=summary,
    own_B2=own[b2],own_scanner=own[scanner],own_saving=own[b2]-own[scanner],
    inclusive_B2_observed=own[b2]+sum(edges[b2].values()),inclusive_scanner_observed=own[scanner]+sum(edges[scanner].values()),
    inclusive_saving_observed=own[b2]-own[scanner]+sum(edges[b2].values())-sum(edges[scanner].values()),
    callee_delta_scanner_minus_B2=delta,minimum=60000000,own_body_gap_to_minimum=60000000-(own[b2]-own[scanner]),
    full_listing_instructions=dict(B2=len(baseline),scanner=len(code)),attempt_listing_instructions=attempts,
    diagnostic_path_counts=path_counts,miri_cases=miri,miri_total=sum(miri.values()),reachable_phases=phases,
    mutations=mutations,full_synthetic=full,packaged_parser_fails=True,
    gate1='PASS on archived evidence; no fresh execution',gate2='diagnostic partial miss',gate3='REJECT qualification; HOLD conservative whole-constructor interval',gate4='outside charter',
    limits=['Only Ir exists in the raw supplied profile; Dr/Dw and run2 cannot be reproduced from it.','No bounded callee/caller cancellation follows from equal call counts or two observations.'])
mb=(json.dumps(sorted(manifest.values(),key=lambda x:(x['pin'],x['path'])),indent=2,sort_keys=True)+'\n').encode()
(a.out/'p2-manifest.json').write_bytes(mb)
result['evidence_digest']=hashlib.sha256(mb).hexdigest()
(a.out/'p2-results.json').write_text(json.dumps(result,indent=2,sort_keys=True)+'\n')
print(json.dumps(result,indent=2,sort_keys=True))
