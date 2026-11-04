#!/usr/bin/env python3
"""Derive the report's tables from immutable sources, logs and raw replay.

Run replay.py first. This only reads archives and evaluates small integer models.
It does not execute any Rust source or mutation.
"""
import argparse, collections, json, pathlib, re
from replay import asm, profile, select

ap=argparse.ArgumentParser();ap.add_argument('--evidence-root',type=pathlib.Path,default=pathlib.Path(__file__).parent/'evidence');ap.add_argument('--replay',type=pathlib.Path,default=pathlib.Path(__file__).parent/'replay-results.json');ap.add_argument('--output',type=pathlib.Path,default=pathlib.Path(__file__).parent/'figures.json');args=ap.parse_args()
root=args.evidence_root; data=json.loads(args.replay.read_text())
def folder(label):return next(p for p in (root/label/'docs/journal').iterdir() if p.is_dir())
def primary(rows):return next(r for r in rows if r['input']=='wikidata-20261026.bz2' and r['permille']==0 and not r['extra_starts'])
def normalise(listing):
    result=[]
    for s in listing.values():
        s=re.sub(r'#.*','',s)
        s=re.sub(r'\b[0-9a-f]+ (?=<)', '@ ',s)
        s=re.sub(r'-?0x[0-9a-f]+\(%rip\)','RIP',s)
        result.append(s.strip())
    return result
out={'verified_blobs':data['verified_blobs'],'baseline_listing_equivalence':{},'work':{},'coverage':{},'overlap':{}}
b2=asm(next((root/'b2list').rglob('ctor2-build-false.asm')))
for component in ['scanner','descriptor']:
    f=folder(component);c=data['components'][component];stats=primary(c['counters'])
    baseline=asm(f/'listings/v2-build-false-same-binary.asm')
    assert normalise(b2)==normalise(baseline)
    out['baseline_listing_equivalence'][component]={'instructions':len(baseline),'same_after_relocations':True}
    rows=c['build_classes_run1'];byfreq={r['frequency']:r for r in rows}
    assert rows==c['build_classes_run2']
    assert c['refill_classes_run1']==c['refill_classes_run2']
    own_delta=sum(r['delta'] for r in rows)+sum(r['delta'] for r in c['refill_classes_run1'])
    base=c['baseline'][0]['functions'];cand=c['candidate'][0]['functions']
    inclusive_delta=cand['build']['inclusive']['Ir']-base['build']['inclusive']['Ir']
    if component=='scanner':
        A=stats['fresh_pairs'];J=stats['lookahead_pairs'];H=stats['held_reuses'];Q=stats['runs_no_full_tile'];S=stats['tiles_nonuniform'];X=stats['tiles_second_segment'];R=stats['runs'];E=stats['epochs'];B=stats['blocks'];T=stats['tails']
        assert A==S+Q and R==H+Q and J+Q==stats['run_rows']//16
        assert stats['rows_scalar']==16*S+stats['tail_rows']
        frequencies={A,J,H,S,X,R,E,B,T,J+Q,J-H,S+R}
        known=sum(byfreq[n]['delta'] for n in frequencies)
        unknown=[r for r in rows if r['frequency'] not in frequencies]
        library_charge=0
    else:
        scalar_freqs=[r['frequency'] for r in rows if r['baseline']==12 and r['candidate']==10]
        assert sum(scalar_freqs)==stats['rows_scalar']
        frequencies=set(scalar_freqs)|{stats['outer_iterations'],stats['outer_iterations']-stats['runs'],stats['runs'],stats['epochs'],stats['blocks'],128*stats['epochs'],63*stats['epochs']}
        known=sum(byfreq[n]['delta'] for n in frequencies)+sum(r['delta'] for r in c['refill_classes_run1'])
        unknown=[r for r in rows if r['frequency'] not in frequencies]
        init=[s for s in c['candidate'][0]['build_sites'] if s['calls']==stats['blocks'] and ('memset' in s['callee'] or 'memcpy' in s['callee'])]
        library_charge=sum(s['Ir'] for s in init)
    pos=sum(r['delta'] for r in unknown if r['delta']>0);neg=sum(r['delta'] for r in unknown if r['delta']<0)
    assert known+pos+neg==own_delta
    lower=-known-pos-library_charge;upper=-inclusive_delta
    memory={k:sum(cand[n]['own'][k]-base[n]['own'][k] for n in ['build','refill','finish','canonicalise']) for k in ['Dr','Dw']}
    var='su' if component=='scanner' else 'd'
    p=profile(f/'callgrind/only-v2-run1.callgrind.gz');q=profile(f/f'callgrind/only-{var}-run1.callgrind.gz')
    caller=q['own']['c30walk::main']['Ir']-p['own']['c30walk::main']['Ir']
    assert caller==2*stats['blocks']
    def libsites(run):return [s for s in run['build_sites'] if 'c30walk' not in s['callee']]
    def key(s):return (s['callee'],s['calls'],s.get('Ir',0),s.get('Dr',0),s.get('Dw',0))
    bs=collections.Counter(key(s) for s in libsites(c['baseline'][0]));cs=collections.Counter(key(s) for s in libsites(c['candidate'][0]))
    shared=sum((bs&cs).values())
    assert bs==collections.Counter(key(s) for s in libsites(c['baseline'][1]))
    assert cs==collections.Counter(key(s) for s in libsites(c['candidate'][1]))
    out['work'][component]={'primary_counters':stats,'counter_supported_delta':known,'profile_only_positive':pos,'profile_only_negative':neg,'own_constructor_delta':own_delta,'inclusive_delta':inclusive_delta,'library_charge':library_charge,'interval_replayed':[lower,upper],'margin_to_minimum':lower-(60000000 if component=='scanner' else 40000000),'own_memory_delta':memory,'caller_main_delta':caller,'caller_main_own':[p['own']['c30walk::main']['Ir'],q['own']['c30walk::main']['Ir']],'interval_if_caller_charged':[lower-caller,upper-caller],'shared_library_sites':shared,'baseline_only_library_sites':list((bs-cs).elements()),'candidate_only_library_sites':list((cs-bs).elements())}
    # Coverage is read from every individual console, not from its summary file.
    cases={};miri_source=(root/component/'examples/c30walk'/('ctor2su.rs' if component=='scanner' else 'ctor2d.rs')).read_text()
    class_block=re.search(r'const (?:SCANNER|DESC)_CLASSES:.*?= \[(.*?)\];',miri_source,re.S)[1]
    for name in re.findall(r'"([a-z]+)"',class_block):
        text=(f/'miri'/f'miri-{name}.console').read_text()
        m=re.search(rf'--miri {name}: (\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact',text)
        assert m and re.search(rf'class {name} exit 0\b',text)
        assert 'error: Undefined Behavior' not in text
        cases[name]=dict(zip(['cases','arrays','walks','restorations'],map(int,m.groups())))
    out['coverage'][component]={'miri_classes':cases,'miri_cases':sum(x['cases'] for x in cases.values())}
    for suite in ['corpora','adversarial']:
        records=[json.loads(l) for l in (f/f'{suite}.jsonl').read_text().splitlines()]
        assert 'exit 0' in (f/f'{suite}.console').read_text()
        out['coverage'][component][suite]={'records':len(records),**{k:sum(x[k] for x in records) for k in ['array_checks','walk_checks','restoration_checks']}}
    for suite in ['synthetic','synthetic-quick']:
        text=(f/f'{suite}.console').read_text()
        m=re.search(r': (\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact',text)
        assert m and 'exit 0' in text
        out['coverage'][component][suite]=dict(zip(['cases','arrays','walks','restorations'],map(int,m.groups())))
    # Preserve every frequency class for independent inspection, including small terms.
    out['work'][component]['changed_build_classes']=rows
    out['work'][component]['changed_refill_classes']=c['refill_classes_run1']

# Exact common B2 reload, not an intersection of aggregate class counts.
assert 'mov    0x8(%rsp),%r13'==b2[min(b2)+0x12b5]
overlap={'baseline_offset':'0x12b5','baseline_address':hex(min(b2)+0x12b5),'instruction':b2[min(b2)+0x12b5]}
for component,mod,var,klass in [('scanner','ctor2su','su','Ctor2su'),('descriptor','ctor2d','d','Ctor2d')]:
    f=folder(component);p=profile(f/'callgrind/only-v2-run1.callgrind.gz');n=select(p,r'::Ctor2>::build::<false>$');a=asm(f/'listings/v2-build-false-same-binary.asm');address=min(a)+0x12b5
    assert a[address]==overlap['instruction'];freq=p['addresses'][n][address]['Ir']
    assert freq==out['work']['scanner']['primary_counters']['tiles_nonuniform']
    q=profile(f/f'callgrind/only-{var}-run1.callgrind.gz');nn=select(q,rf'::{klass}>::build::<false>$');aa=asm(f/f'listings/{mod}-build-false.asm')
    base=min(aa);lo,hi=(0xa80,0xa89) if component=='scanner' else (0x13e0,0x13ef)
    overlap[component+'_successor_block']=[{'offset':hex(ad-base),'instruction':aa[ad],'executions':q['addresses'][nn].get(ad,{}).get('Ir',0)} for ad in aa if lo<=ad-base<=hi]
overlap['demonstrated_lower_bound']=freq;out['overlap']=overlap

# Source-independent integer checks of the scanner range and v3 tag arguments.
scanner=(root/'scanner/examples/c30walk/ctor2su.rs').read_text()
assert scanner.count('ll.get_unchecked(')==1
assert '16usize.min(ROWS.wrapping_sub(self.p))' in scanner
assert 'w.tile16(ll, end - rest.len(), t)' in scanner
phases=sorted({(16*k)%30 for k in range(15)})
for i in range(0,983010-15,16):
    for p in phases:
        n1=min(16,30-p);assert i<=i+n1<=i+16<=983010
out['scanner_range_model']={'reachable_phases':phases,'full_tile_offsets_checked':len(range(0,983010-15,16)),'unchecked_operations_in_final_source':scanner.count('ll.get_unchecked(')}
mutants={}
f=folder('scanner')
for mut,line,operation,diagnostic in [('mu1',667,'U2','arithmetic overflow in `unchecked_sub`'),('mu2',666,'U1','dangling reference'),('mu3',666,'U2','dangling reference')]:
    for mode in ['default','noubchecks']:
        text=(f/'miri'/f'miri-{mut}-{mode}.console').read_text()
        assert f'ctor2su.rs:{line}:' in text and f'mutant {mut} {mode} exit 1' in text
        assert ('slice::get_unchecked requires' if mode=='default' else diagnostic) in text
    mutants[mut]={'intended_operation':operation,'diagnostic_line':line,'both_modes_caught':True}
for comp in ['scanner','descriptor']:
    f=folder(comp)
    for number in [1,2]:
        path=f/'mutations'/f'm{number}.console' if comp=='scanner' else f/f'mutation-M{number}.console'
        assert 'exit 101' in path.read_text() and 'panicked' in path.read_text()
out['mutants']=mutants

f=folder('finaliser');text=(f/'miri.console').read_text()
m=re.search(r'--miri: (\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact; leaf (\d+), slow (\d+) \(old epoch (\d+), partial (\d+), preset RAW (\d+)\), wide (\d+)',text)
assert m and 'exit 0' in text and 'error: Undefined Behavior' not in text
numbers=list(map(int,m.groups()));assert numbers[0]==41*3+4+4+2+1 and numbers[5]==sum(numbers[6:9])
out['coverage']['finaliser']=dict(zip(['miri_cases','arrays','walks','restorations','leaf','slow','old_epoch','partial','preset_raw','wide'],numbers))
for suite in ['corpora','adversarial']:
    rs=[json.loads(l) for l in (f/f'{suite}.jsonl').read_text().splitlines()]
    out['coverage']['finaliser'][suite]={'records':len(rs),**{k:sum(x[k] for x in rs) for k in ['array_checks','walk_checks','restoration_checks']}}
for suite in ['corpora','adversarial','synthetic','synthetic-quick']:
    a=(f/f'{suite}.console').read_text().removesuffix('exit 0\n');b=(f/'rebuild'/f'{suite}.console').read_text()
    assert a==b,(suite,'rebuild console mismatch')
for suite in ['synthetic','synthetic-quick']:
    m=re.search(r': (\d+) cases, (\d+) array comparisons, (\d+) walk outputs, (\d+) restorations: every check exact',(f/f'{suite}.console').read_text())
    out['coverage']['finaliser'][suite]=dict(zip(['cases','arrays','walks','restorations'],map(int,m.groups())))
assert 'exit 101' in (f/'mutation.console').read_text()
assert 'd.raw as usize) < self.raw.len()' in (f/'mutation.console').read_text()
leaf=asm(f/'listings/ctor3-finish-false.asm')
reads=writes=0
for s in leaf.values():
    op,*rest=s.split(None,1)
    if op=='lea' or not rest:continue
    operands=re.split(r',\s*(?![^()]*\))',rest[0].split('#')[0])
    if len(operands)==2:
        reads+='(' in operands[0]
        writes+='(' in operands[1] and op.startswith('mov')
out['finaliser_leaf']={'instructions':len(leaf),'explicit_reads':reads,'explicit_writes':writes}
assert len(leaf)==44 and reads==12 and writes==6
tags=[0x80000000|(e<<24) for e in range(16)]
assert len(set(tags))==16 and all(t!=0xffffffff and t>0xfffff for t in tags)
phi=lambda i:i+2*(i//30)
assert phi(61439)==65533 and phi(61440)-phi(3)==65533 and phi(61440)-phi(2)==65534
for e in range(16):
    for u in [0,1,65533]:
        for ch in [0,1,255]:assert ((tags[e]&0x7fffffff)|(u<<8)|ch)==((65536*e+u)<<8)|ch
out['finaliser_integer_model']={'epochs':len(tags),'max_same_epoch_span':phi(61439),'boundary_spans':[phi(61440)-phi(3),phi(61440)-phi(2)]}
manifest=json.loads((pathlib.Path(__file__).parent/'manifest.json').read_text())
out['frozen_sources']={}
for filename,baseline in [('ctor2.rs','b2'),('ctor3.rs','b3')]:
    original=(root/baseline/'examples/c30walk'/filename).read_bytes()
    for comp in ['scanner','descriptor','finaliser']:assert original==(root/comp/'examples/c30walk'/filename).read_bytes()
    out['frozen_sources'][filename]=next(e['blob'] for e in manifest if e['local']==f'evidence/{baseline}/examples/c30walk/{filename}')
args.output.write_text(json.dumps(out,indent=2)+'\n')
print(json.dumps({k:v for k,v in out.items() if k not in ['work','overlap']},indent=2))
for comp,w in out['work'].items():print(comp,{k:w[k] for k in ['counter_supported_delta','profile_only_positive','profile_only_negative','own_constructor_delta','inclusive_delta','library_charge','interval_replayed','own_memory_delta','caller_main_delta']})
print('demonstrated overlap',out['overlap']['demonstrated_lower_bound'])
