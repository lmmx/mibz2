"""Offline D008 archive replay. Usage: python replay_d008.py MANIFEST CACHE

Populate CACHE/<key> from each manifest entry's exact repository/ref/path.
No network, constructor execution, compilation, or performance measurements.
"""
import collections
import gzip
import hashlib
import json
from pathlib import Path
import re
import sys
from parser import parse

manifest = json.loads(Path(sys.argv[1]).read_text())
cache = Path(sys.argv[2])
files = {}
for row in manifest['files']:
    data = (cache / row['key']).read_bytes()
    assert hashlib.sha256(data).hexdigest() == row['sha256'], row['key']
    assert hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest() == row['blob'], row['key']
    files[row['key']] = data

def text(key):
    return files[key].decode()

def profile(key):
    pos, events, own, calls, counts = parse(str(cache / key))
    out = {}
    for f, v in own.items():
        if re.search(r'ctor2(?:d|s|su)?::.*(?:build::<false>|W<false>)', f):
            ix = [events.index(e) for e in ('Ir', 'Dr', 'Dw')]
            inc = [a + sum(c[i] for c in calls[f].values()) for i, a in enumerate(v)]
            out[f] = {'self': [v[i] for i in ix], 'inclusive': [inc[i] for i in ix]}
    return out

def instruction_counts(key):
    names, own, sites = {}, collections.defaultdict(collections.Counter), collections.defaultdict(list)
    fn = cfn = None
    pending = None
    last = [0, 0]
    for line in gzip.decompress(files[key]).decode().splitlines():
        if not line: continue
        if line.startswith('positions:'): assert line.split()[1:] == ['instr', 'line']
        if line.startswith('events:'): events = line.split()[1:]
        k, sep, value = line.partition('=')
        if sep and k in ('fn', 'cfn', 'fl', 'fi', 'fe', 'cfl', 'cfi', 'ob', 'cob'):
            kind = 'fn' if k in ('fn','cfn') else 'ob' if k in ('ob','cob') else 'fl'
            m = re.match(r'\((\d+)\)(?: (.*))?$', value)
            if m:
                if m[2] is not None: names[kind,m[1]] = m[2]
                value = names[kind,m[1]]
            if k == 'fn': fn = value
            if k == 'cfn': cfn = value
            continue
        if k == 'calls' and sep:
            pending = int(value.split()[0]); continue
        if line[0].isdigit() or line[0] in '+-*':
            parts = line.split()
            for i in range(2):
                t = parts[i]
                last[i] = last[i] if t == '*' else last[i]+int(t,0) if t[0] in '+-' else int(t,0)
            vals = list(map(int,parts[2:])); vals += [0]*(len(events)-len(vals))
            if pending is None: own[fn][last[0]] += vals[events.index('Ir')]
            else:
                sites[fn].append((last[0], cfn, pending, tuple(vals[events.index(e)] for e in ('Ir','Dr','Dw'))))
                pending = None
    return own, sites

def classes(prof, listing, fn):
    ins = {int(m[1],16) for m in re.finditer(r'^\s*([0-9a-f]+):\s', text(listing), re.M)}
    counts = instruction_counts(prof)[0][fn]
    assert set(counts) <= ins, (prof, set(counts)-ins)
    return collections.Counter(counts.get(a,0) for a in ins)

def delta(a,b):
    return {n:n*(b[n]-a[n]) for n in set(a)|set(b) if n and b[n]!=a[n]}

profiles = {k:profile(k) for k in files if k.endswith('.callgrind.gz')}
for tag, labels in [('d',['d','v2']),('s',['su','v2']),('e',[''])]:
    for label in labels:
        pre = tag+'-'+('only-'+label+'-' if label else '')
        assert profiles[pre+'run1.callgrind.gz'] == profiles[pre+'run2.callgrind.gz']

base = '<c30walk::ctor2::Ctor2>::build::<false>'
dname = '<c30walk::ctor2d::Ctor2d>::build::<false>'
sname = '<c30walk::ctor2su::Ctor2su>::build::<false>'
dcl = delta(classes('d-only-v2-run1.callgrind.gz','d/listings/v2-build-false-same-binary.asm',base),classes('d-only-d-run1.callgrind.gz','d/listings/ctor2d-build-false.asm',dname))
scl = delta(classes('s-only-v2-run1.callgrind.gz','s/listings/v2-build-false-same-binary.asm',base),classes('s-only-su-run1.callgrind.gz','s/listings/ctor2su-build-false.asm',sname))
def primary(tag):
    return next(x for x in map(json.loads,text(tag+'/corpora.jsonl').splitlines()) if x['input']=='wikidata-20261026.bz2' and x['permille']==0 and not x['extra_starts'])
d,s = primary('d'),primary('s')
scalar = [1769,29357924,8935084,106]
assert sum(scalar)==d['rows_scalar']
dm = set(scalar+[d['outer_iterations'],d['runs'],d['epochs'],128*d['epochs'],63*d['epochs'],d['blocks'],d['outer_iterations']-d['runs']])
dmatched = sum(v for k,v in dcl.items() if k in dm)+6*d['refills']+d['refill_finishes']
dp = sum(v for k,v in dcl.items() if k not in dm and v>0)
dn = sum(v for k,v in dcl.items() if k not in dm and v<0)
A,J,H,Q,S,X,R = [s[k] for k in ('fresh_pairs','lookahead_pairs','held_reuses','runs_no_full_tile','tiles_nonuniform','tiles_second_segment','runs')]
sm = {J+Q,J,J-H,A,S+R,X,S,R,H,s['epochs'],s['blocks'],s['tails']}
smatched = sum(v for k,v in scl.items() if k in sm)
sp = sum(v for k,v in scl.items() if k not in sm and v>0)
sn = sum(v for k,v in scl.items() if k not in sm and v<0)
assert (dmatched,dp,dn)==(-56102571,1173383,-3279857)
assert (smatched,sp,sn)==(-88402421,4359203,-17203)

def build(tag,variant,name): return profiles[tag+'-only-'+variant+'-run1.callgrind.gz'][name]
db,dc = build('d','v2',base),build('d','d',dname)
sb,sc = build('s','v2',base),build('s','su',sname)
def sub(a,b): return [x-y for x,y in zip(a,b)]
refbase='<c30walk::ctor2::W<false>>::refill'
refnew='<c30walk::ctor2d::W<false>>::refill'
refdelta=sub(profiles['d-only-d-run1.callgrind.gz'][refnew]['self'],profiles['d-only-v2-run1.callgrind.gz'][refbase]['self'])
ddelta=[a+b for a,b in zip(sub(dc['self'],db['self']),refdelta)]
assert ddelta==[-58209045,-1096093,5004156]
assert sub(sc['self'],sb['self'])==[-84060421,-505885,3391432]

# Compare archived call-site costs from the raw records, not the printed summaries.
def library_sites(key,fn):
    combined = {}
    for addr,callee,n,vals in instruction_counts(key)[1][fn]:
        if 'c30walk::' in callee: continue
        row = combined.setdefault((addr,callee),[0,0,0,0])
        for i,v in enumerate((n,*vals)): row[i] += v
    return [(callee,row[0],tuple(row[1:])) for (addr,callee),row in combined.items()]
for run in (1,2):
    assert sorted(library_sites(f's-only-su-run{run}.callgrind.gz',sname))==sorted(library_sites(f's-only-v2-run{run}.callgrind.gz',base))
    ds=library_sites(f'd-only-d-run{run}.callgrind.gz',dname)
    bs=library_sites(f'd-only-v2-run{run}.callgrind.gz',base)
    assert sorted(x for x in ds if x[1]!=434)==sorted(x for x in bs if x[1]!=434)
    assert sum(x[2][0] for x in ds if x[1]==434)==378882
    assert sum(x[2][0] for x in bs if x[1]==434)==4447632

for tag in ('d','s'):
    miri=[k for k in files if k.startswith(tag+'/miri/miri-') and '-mu' not in k]
    assert len(miri)==9
    for k in miri: assert 'every check exact' in text(k) and 'exit 0' in text(k)
    for name in ('corpora.console','adversarial.console','synthetic.console'): assert text(tag+'/'+name).rstrip().endswith('exit 0')

old='<c30walk::ctor2s::Ctor2s>::build::<false>'
eb,ec=profiles['e-run1.callgrind.gz'][base],profiles['e-run1.callgrind.gz'][old]
results={'descriptor':{'counter_matched_delta':dmatched,'counterless_positive':dp,'counterless_negative':dn,'lower':-dmatched-dp-378882,'upper_observed':db['inclusive'][0]-dc['inclusive'][0],'own_delta_Ir_Dr_Dw':ddelta,'refill_delta_Ir_Dr_Dw':refdelta},'scanner_u2':{'counter_matched_delta':smatched,'counterless_positive':sp,'counterless_negative':sn,'lower':-smatched-sp,'upper_observed':sb['inclusive'][0]-sc['inclusive'][0],'own_delta_Ir_Dr_Dw':sub(sc['self'],sb['self'])},'correction':{'own_delta_Ir_Dr_Dw':sub(ec['self'],eb['self']),'inclusive_observed_delta_Ir_Dr_Dw':sub(ec['inclusive'],eb['inclusive']),'whole_constructor_bound':'not established by these measurements'},'raw_profiles':len(profiles),'completed_registered_miri_classes':{'descriptor':9,'scanner_u2':9},'scope':'offline archive replay; no rerun of constructors or Miri; no general timing or arbitrary-environment claim'}
assert results['descriptor']['lower']==54550306
assert results['scanner_u2']['lower']==84043218
for listing, ranges in [
    ('d/listings/ctor2d-build-false.asm',[(0x9c0,0x9ed,10),(0x1370,0x139d,10),(0x1410,0x143a,10),(0x14d0,0x14fd,10)]),
    ('s/listings/ctor2su-build-false.asm',[(0xa10,0xa39,12),(0xaa0,0xaca,12),(0x13a0,0x13ca,12),(0x1420,0x1444,11)])
]:
    addresses=[int(m[1],16) for m in re.finditer(r'^\s*([0-9a-f]+):\s',text(listing),re.M)]
    start=min(addresses)
    for lo,hi,n in ranges: assert sum(start+lo<=a<=start+hi for a in addresses)==n
for tag,names in [('d',['mutation-M1.console','mutation-M2.console']),('s',['mutations/m1.console','mutations/m2.console'])]:
    for name in names: assert 'panicked' in text(tag+'/'+name) and 'exit 101' in text(tag+'/'+name)
for mu in ('mu1','mu2','mu3'):
    for mode in ('default','noubchecks'):
        body=text(f's/miri/miri-{mu}-{mode}.console')
        assert 'exit 1' in body and ('precondition' in body if mode=='default' else 'Undefined Behavior' in body)
results['row_loop_counts']={'descriptor':[10,10,10,10],'scanner_u2':[12,12,12,11]}
results['mutation_logs_verified']={'descriptor_release':2,'scanner_release':2,'scanner_miri_controls':6}
print(json.dumps(results,indent=2))
