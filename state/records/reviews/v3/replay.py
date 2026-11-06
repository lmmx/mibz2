#!/usr/bin/env python3
"""Replay pinned archive evidence only; no constructor execution or compilation.

Use --repo for a checkout containing the recorded evidence. If a working-tree
file is absent or differs, its exact pinned Git object is read with git show.
Use --evidence-dir for downloaded files laid out by manifest path or cache_path.
The options can be combined. Nothing is downloaded and inputs are never edited.
"""
import argparse, hashlib, json, pathlib, re, subprocess

HERE = pathlib.Path(__file__).resolve().parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--repo', type=pathlib.Path, help='Evidence checkout, with pinned history when needed')
parser.add_argument('--evidence-dir', type=pathlib.Path, help='Downloaded evidence root, using manifest path/cache_path')
parser.add_argument('--output', type=pathlib.Path, help='Optional JSON result destination; default is stdout only')
args = parser.parse_args()
if not args.repo and not args.evidence_dir:
    parser.error('Supply --repo and/or --evidence-dir; see report.md for replay examples.')
if not __debug__:
    parser.error('Run without Python -O: archive checks use assertions.')

def gitblob(b):
    return hashlib.sha1(b'blob ' + str(len(b)).encode() + b'\0' + b).hexdigest()

manifest_bytes = (HERE/'evidence-manifest.json').read_bytes()
manifest = json.loads(manifest_bytes)
data = {}
for row in manifest:
    match = re.fullmatch(r'https://github.com/lmmx/mibz2/blob/([0-9a-f]{40})/(.+)', row['url'])
    if not match or match[2] != row['path']:
        parser.error('Manifest entry lacks an exact full commit/path: ' + row['url'])
    commit, repo_path = match.groups()
    candidates = []
    if args.evidence_dir:
        candidates += [args.evidence_dir / row['path'], args.evidence_dir / row['cache_path']]
    if args.repo:
        candidates.append(args.repo / repo_path)
    found = None
    for file in candidates:
        if file.is_file():
            b = file.read_bytes()
            if gitblob(b) == row['git_blob'] and hashlib.sha256(b).hexdigest() == row['sha256']:
                found = b
                break
    if found is None and args.repo:
        result = subprocess.run(['git', '-C', str(args.repo), 'show', f'{commit}:{repo_path}'],
                                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)
        if result.returncode == 0:
            b = result.stdout
            if gitblob(b) == row['git_blob'] and hashlib.sha256(b).hexdigest() == row['sha256']:
                found = b
    if found is None:
        parser.error('Missing or mismatched pinned input: ' + row['url'] +
                     ' (expected Git blob ' + row['git_blob'] + '). ' +
                     'Make its commit available locally, or download it into --evidence-dir.')
    if repo_path in data and data[repo_path] != found:
        parser.error('Conflicting pinned versions for ' + repo_path)
    data[repo_path] = found

def read(path):
    return data[str(path)].decode('utf-8')

E = pathlib.PurePosixPath('docs/journal/2026-11-04-compact30-epoch-tagged-finaliser')
B2 = pathlib.PurePosixPath('docs/journal/2026-11-03-compact30-constructor-v2-prototype')
B3 = E / 'listings'

def instructions(path):
    out=[]
    for l in read(path).splitlines():
        m = re.match(r'^\s*([0-9a-f]+):\s+([^\s]+)\s*(.*)', l)
        if m:
            out.append((int(m[1],16), m[2], m[3].split('#')[0].strip()))
    return out

leaf = instructions(B3/'ctor3-finish-false.asm')
assert len(leaf) == 44
assert [i+1 for i,(_,op,_) in enumerate(leaf) if op=='jne'] == [5,12]
assert all(op not in ('push','pop','call') and '%rsp' not in args
           for _,op,args in leaf)
assert leaf[-1][1] == 'ret'

def memory(path):
    reads=writes=push=pop=0
    for _,op,args in path:
        push += op == 'push'
        pop += op == 'pop'
        if op.startswith(('lea','nop')) or op in ('push','pop','call','ret'):
            continue
        operands = re.split(r',\s*(?![^()]*\))',args)
        for i,a in enumerate(operands):
            if '(' not in a:
                continue
            dst=i==len(operands)-1 and op not in ('cmp','test','jmp')
            writes+=dst
            reads+=not dst or not op.startswith('mov')
    return dict(reads=reads,writes=writes,push=push,pop=pop)

assert memory(leaf) == dict(reads=12,writes=6,push=0,pop=0)
old=instructions(B2/'ctor2-finish-false.asm')
# Explicit successful fast/full branch trace, including branch and ret instructions.
ranges=[(0x61d30,0x61d93),(0x61e08,0x61e17),(0x61f1b,0x61fb8),(0x6231b,0x62329)]
fast=[i for lo,hi in ranges for i in old if lo<=i[0]<=hi]
assert len(fast)==81
assert memory(fast)==dict(reads=17,writes=8,push=6,pop=6)
sizes={pathlib.PurePosixPath(p).name:len(instructions(p)) for p in data if p.startswith(str(B3)+'/') and p.endswith('.asm')}
assert sizes=={'ctor3-build-false.asm':1432,'ctor3-finish-false.asm':44,
               'ctor3-finish-slow-false.asm':378,'ctor3-refill-false.asm':67}

rows=[json.loads(l) for l in read(E/'corpora.jsonl').splitlines() if l]
assert len(rows)==44
for r in rows:
    assert r['leaf_fast']==r['v2_fast_full']
    assert r['fin_fast']==r['leaf_fast']+r['slow_ordinary_partial_same_epoch']
    assert r['slow_entries']==r['slow_ordinary_old_epoch']+r['slow_ordinary_partial_same_epoch']+r['slow_raw_preset']
    assert r['slow_raw_preset']==r['fin_crossing']+r['fin_forced']
    assert sum(r[x] for x in ['fin_fast','fin_ordinary','fin_wide','fin_crossing','fin_forced'])==r['leaf_fast']+r['slow_entries']
primary=[r for r in rows if r['input']=='wikidata-20261026.bz2' and r['permille']==0 and not r['extra_starts']]
assert len(primary)==1
p=primary[0]
assert (p['blocks'],p['rows'])==(434,207006515)
F=p['leaf_fast']; S=p['slow_entries']; P=p['slow_ordinary_partial_same_epoch']; O=p['opens_ordinary']; epochs=p['epochs']
assert (F,S,P,O,epochs)==(1647375,148107,41,1754362,3654)
gross=(len(fast)-len(leaf))*F
center=gross-5*(S-P)-12*P-4*epochs
assert (gross,center,center-S,center+S)==(60952875,60197437,60049330,60345544)
slow=instructions(B3/'ctor3-finish-slow-false.asm')
old_by_addr={a:(op,args) for a,op,args in old}
slow_by_addr={a:(op,args) for a,op,args in slow}
assert [old_by_addr[a][0] for a in [0x62312,0x62316,0x62319,0x6230d,0x62310]]==['sub','inc','jmp','cmp','je']
assert [slow_by_addr[a][0] for a in [0x8106d,0x81071,0x81074,0x81077]]==['sub','inc','cmp','jne']
assert p['norm_partial']==42
# Independent conservative envelope from complete-build/slow-path audit. This
# does not replace the checkpoint's authoritative interval.
cost_envelope=(center-3*p['blocks'],center+3*p['norm_partial']+2*p['blocks'])
assert center-S <= cost_envelope[0] <= cost_envelope[1] <= center+S

mc=read(E/'miri.console')
classes=[int(n) for n in re.findall(r'^ctor3-synthetic: .*: (\d+) cases exact$',mc,re.M)]
assert classes==[127,4,2,1] and sum(classes)==134
assert '134 cases, 262 array comparisons, 393 walk outputs, 262 restorations: every check exact' in mc
assert 'leaf 6162, slow 189 (old epoch 8, partial 105, preset RAW 76), wide 1' in mc
assert mc.endswith('exit 0\n')
assert 2*(134-3)==262 and 3*(134-3)==393 # Three empty small-block cases.
mut=read(E/'mutation.diff')
assert mut.count('-            if d.raw & ORD != 0 {')==1
assert mut.count('-        if d.raw & ORD != 0 {')==1
assert mut.count('d.raw == u32::MAX')==2
mutlog=read(E/'mutation.console')
assert 'assertion failed: (d.raw as usize) < self.raw.len()' in mutlog
assert 'finish_slow<true>' in mutlog and mutlog.endswith('exit 101\n')

def normalized_console(p):
    return [s for s in read(p).splitlines() if s!='exit 0']
rebuild={}
for n in ['corpora.console','adversarial.console','synthetic.console','synthetic-quick.console']:
    rebuild[n]=normalized_console(E/n)==normalized_console(E/'rebuild'/n)
    assert rebuild[n]
full=read(E/'synthetic.console')
assert '53960 cases, 99840 array comparisons, 149760 walk outputs, 99840 restorations: every check exact (full)' in full
assert 'malformed counts: 3995 rejected, 7 accepted' in full
full_counts=json.loads(re.search(r'ctor3-synthetic counters: (\{.*\})',full)[1])
assert full_counts['max_ordinary_span']==65533 and full_counts['min_wide_span']==65534

result=dict(archived_ledger=dict(gross=gross,center=center,lower=center-S,upper=center+S),
    supporting_cost_envelope=dict(lower=cost_envelope[0],upper=cost_envelope[1],
         residual_field_saving_upper=3*p['norm_partial'],reserve_move_saving_upper=2*p['blocks'],
         conservative_empty_build_setup_charge_upper=3*p['blocks'],
         canonical_ledger_replaced=False),
    primary_counts={k:p[k] for k in ['blocks','rows','leaf_fast','slow_entries','slow_ordinary_old_epoch','slow_ordinary_partial_same_epoch','slow_raw_preset','opens_ordinary','epochs']},
    explicit_memory=dict(v2_fast=memory(fast),v3_leaf=memory(leaf),
         journal_reads_delta=-5*F+O+2*S+2*P,
         journal_writes_delta=-2*F,journal_push_pop_delta=-12*F,
         corrected_reads_delta_nonempty=-5*F+O+2*S+2*P+2*p['blocks'],
         corrected_writes_delta_nonempty=-2*F+p['blocks']+epochs),
    whole_listing_sizes=sizes,miri=dict(cases=134,arrays=262,walks=393,restorations=262,leaf=6162,slow=189),
    rebuild_console_equivalent=rebuild,verified_blobs=len(manifest),
    evidence_manifest_sha256=hashlib.sha256(manifest_bytes).hexdigest(),
    limits=['Archived evidence replay only; no fresh Rust/Miri/corpus execution.',
            'Archived primary counters are per-corpus totals; not an independent per-block collection.',
            'Ledger interval is the checkpoint claim; whole-build/fallback review is recorded separately.',
            'No completed sequence closeout index or combined build; this is not charter section 6.2.'])
rendered=json.dumps(result,indent=2,sort_keys=True)+'\n'
if args.output:
    args.output.write_text(rendered)
print(rendered,end='')
