#!/usr/bin/env python3
"""Static scanner-event model and archive checks only; no constructor or compilation."""
from pathlib import Path
from itertools import product
from collections import Counter
import json, re, hashlib

HERE = Path(__file__).resolve().parent
EPOCH = 61440

def scanner(v, sliced, mutate=None):
    # Model source consumption and callbacks, never representation/destination writes.
    events, st = [], Counter()
    start = 0
    while start < len(v):
        end = min(len(v), start + EPOCH)
        i, held = start, None
        while end - i >= 16:
            if held is None:
                tile = v[i:i+16]
                st['fresh'] += 1
            else:
                tile, held = held, None
                st['held'] += 1
            if tile == tile[:1] * 16:
                e = i + 16
                while end - e >= 16:
                    nxt = v[e:e+16]
                    st['lookahead'] += 1
                    if nxt == tile:
                        e += 16
                    else:
                        held = nxt
                        break
                st['no-full-tile runs'] += held is None
                # Slice lengths in the new loop; absolute positions in frozen v2.
                m = ((end-i) - (end-e)) if sliced else e-i
                if mutate == 'M2':
                    m -= 16
                events.append(('run', i, m, tile[0]))
                i = e
                if mutate == 'M1' and held is not None:
                    held = None
                    i += 16
            else:
                events.append(('tile', i, 16, bytes(v[i:i+16])))
                st['nonuniform'] += 1
                st['second segments'] += (30-i%30) < 16
                i += 16
        if i < end:
            events.append(('tile', i, end-i, bytes(v[i:end])))
            st['tails'] += 1
        start = end
    return events, st

def filler(n):
    return bytes(2+(k&1) for k in range(n))

def cases():
    out = {}
    out['tiny'] = [bytes(((k//3)%s)*5 for k in range(n)) for n in [0,1,2,15,16,17,31,32,33] for s in [1,2]]
    out['tails'] = [v for n in range(1,16) for v in [bytes(k&1 for k in range(n)), filler(16)+bytes(k%3 for k in range(n)), bytes([7])*32+bytes(7+k%2 for k in range(n))]]
    # Alignment affects pointer offset only, not the event trace.
    out['align'] = [v for _ in range(1,8) for v in [bytes([3,1,2,1,3]),bytes([4])*32+filler(16)+bytes([4])*48+filler(21)]]
    out['phase'] = [bytes([9])*(16*k)+filler(16)+filler(5) for k in range(15)]
    out['held'] = [bytes([6])*48+nxt+filler(19) for nxt in [bytes([8])*16, filler(16), bytes([6])*8+bytes([8])*8,bytes([8])*8+bytes([6])*8]]
    out['held'].append(b''.join(bytes([6 if k%2==0 else 8])*16 for k in range(8))+filler(3))
    out['endrun'] = [filler(16)+bytes([5])*64+filler(n) for n in [0,5]]
    a=bytearray(filler(EPOCH+64));a[EPOCH-48:EPOCH+48]=bytes([4])*96
    out['epoch'] = [bytes(a),bytes([4])*(EPOCH-16)+filler(16)+bytes([4])*64]
    return out

def log_count(class_):
    t=(HERE/f'miri-{class_}.console').read_text()
    assert re.search(r'^exit 0$',t,re.M)
    m=re.search(r'--miri \w+: (\d+) cases, .*?every check exact; (.*)',t)
    return int(m[1]), {label:int(n) for label,n in re.findall(r'([a-z-]+(?: [a-z-]+)*) (\d+)(?:,|$)',m[2])}

def main():
    report={'miri':{},'source_pins':{},'archive_totals':{},'mutations':{}}
    pins=json.loads((HERE/'fetched-pins.json').read_text())
    for name, info in pins.items():
        d=(HERE/name).read_bytes()
        assert hashlib.sha1(b'blob '+str(len(d)).encode()+b'\0'+d).hexdigest()==info['git_blob'],name
    report['pinned_files_verified']=len(pins)
    # Compare unchanged destination functions verbatim at source level. This
    # parser is intentionally limited to these pinned bodies (balanced braces).
    def body(src, name):
        m=re.search(r'\bfn '+name+r'(?:<|\()', src)
        start=src.index('{',m.start()); depth=1;end=start+1
        while depth:
            depth+=(src[end]=='{')-(src[end]=='}');end+=1
        return src[start:end]
    old=(HERE/'ctor2.rs').read_text();new=(HERE/'ctor2s.rs').read_text()
    unchanged=['field','advance','create_raw','snapshot','epoch_of','open','close','refill','finish','write_entries','one','fill','complete','run']
    for name in unchanged:
        assert body(old,name)==body(new,name),name
    assert body(old,'build')==body(new,'build').replace('Ctor2s','Ctor2')
    report['unchanged_function_bodies']=unchanged+['build (type name only)']
    w0=(HERE/'walk-0.rs').read_text();w1=(HERE/'walk-1.rs').read_text()
    w0=w0.replace('/// Charter prototype 1: v2 with its scanner\'s source bounds re-expressed in safe Rust\n#[path = "c30walk/ctor2s.rs"]\nmod ctor2s;\n\n','')
    w0=w0.replace('    if let Some(m) = args[1].strip_prefix("ctor2s-") {\n        return ctor2s::main(m, &args[2..]);\n    }\n','')
    assert w0==w1
    report['immutable_walker_oracle_types']='byte-identical to PR base after removing only new module and dispatch'
    for name,vs in cases().items():
        total=Counter()
        for v in vs:
            new,stats=scanner(v,True)
            old,_=scanner(v,False)
            assert old==new
            assert sum(e[2] for e in new)==len(v)
            total.update(stats)
        count,logged=log_count(name)
        assert count==len(vs)
        assert {k:total[k] for k in logged}==logged,(name,total,logged)
        report['miri'][name]={'cases':count,'counts':logged}
    count,logged=log_count('malformed')
    assert count==4 and all(v==0 for v in logged.values())
    report['miri']['malformed']={'cases':count,'counts':logged}
    assert sum(x['cases'] for x in report['miri'].values())==105
    phases=sorted({16*k%30 for k in range(15)})
    assert phases==list(range(0,30,2)) and EPOCH%30==EPOCH%16==0
    report['reachable_phases']=phases
    # Each symbol denotes a full tile's equality class. Full binary/ternary row
    # enumeration is unnecessary: scanner control depends only on these classes.
    alphabet=[bytes([0])*16,bytes([1])*16,bytes([0])*8+bytes([1])*8,bytes([1])*8+bytes([0])*8,filler(16)]
    n=0
    for k in range(7):
        for seq in product(alphabet,repeat=k):
            v=b''.join(seq)+filler(k%16)
            a,_=scanner(v,False);b,_=scanner(v,True)
            assert a==b and sum(x[2] for x in a)==len(v)
            n+=1
    report['symbolic_event_cases']=n
    for m,v in [('M1',bytes([0])*16+filler(16)),('M2',bytes([0])*16)]:
        original,_=scanner(v,True);mutant,_=scanner(v,True,m)
        assert original!=mutant
        consumed=sum(x[2] for x in mutant)
        assert consumed<len(v)
        console=(HERE/f'mutation-{m}.console').read_text()
        assert 'counted build rejected valid counts' in console and 'exit 101' in console
        report['mutations'][m]={'input_rows':len(v),'processed_rows':consumed,'recorded_exit':101}
    for suite in ['corpora','adversarial']:
        records=[json.loads(x) for x in (HERE/f'{suite}.jsonl').read_text().splitlines() if x]
        for x in records:
            assert x['fresh_pairs']==x['tiles_nonuniform']+x['runs_no_full_tile']
            assert x['runs']==x['held_reuses']+x['runs_no_full_tile']
            assert x['rows_scalar']==16*x['tiles_nonuniform']+x['tail_rows']
            assert 16*(x['fresh_pairs']+x['lookahead_pairs'])+x['tail_rows']==x['rows']
            assert 16*(x['fresh_pairs']+x['lookahead_pairs']-x['tiles_nonuniform'])==x['run_rows']
            assert x['array_checks']==2*x['blocks'] and x['walk_checks']==3*x['blocks'] and x['restoration_checks']==2*x['blocks']
        report['archive_totals'][suite]={'records':len(records),**{k:sum(x[k] for x in records) for k in ['array_checks','walk_checks','restoration_checks','blocks','rows']}}
    for name in sorted(pins):
        p=HERE/name
        data=p.read_bytes()
        report['source_pins'][p.name]={'git_blob':hashlib.sha1(b'blob '+str(len(data)).encode()+b'\0'+data).hexdigest(),'sha256':hashlib.sha256(data).hexdigest()}
    digest_input=''.join(name+'\t'+report['source_pins'][name]['git_blob']+'\t'+report['source_pins'][name]['sha256']+'\n' for name in sorted(report['source_pins']))
    report['evidence_digest_sha256']=hashlib.sha256(digest_input.encode()).hexdigest()
    print(json.dumps(report,indent=2,sort_keys=True))

if __name__=='__main__':
    main()
