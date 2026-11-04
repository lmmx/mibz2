#!/usr/bin/env python3
"""Independent archive replay. No candidate execution or compilation.

Use --evidence-root with files materialized by fetch_evidence.py. Outputs
instruction costs in the archived Callgrind convention, not timings.
"""
import argparse, collections, gzip, hashlib, json, pathlib, re

def profile(path):
    names = {}; own = collections.defaultdict(lambda: collections.Counter())
    sites = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
    addresses = collections.defaultdict(lambda: collections.defaultdict(collections.Counter))
    edges = collections.Counter(); current = callee = None; pending = None
    positions = []; previous = []; events = []; summary = None
    def symbol(raw):
        m = re.fullmatch(r'\((\d+)\)(?: (.*))?', raw)
        if not m: return raw
        key = int(m[1])
        if m[2] is not None: names[key] = m[2]
        return key
    for raw in gzip.open(path, 'rt'):
        line = raw.strip()
        if line.startswith('positions:'):
            positions=line.split()[1:]; previous=[0]*len(positions)
        elif line.startswith('events:'): events=line.split()[1:]
        elif line.startswith('summary:'): summary=dict(zip(events,map(int,line.split()[1:])))
        elif line.startswith('fn='): current=symbol(line[3:])
        elif line.startswith('cfn='): callee=symbol(line[4:])
        elif line.startswith('calls='): pending=int(line.split()[0][6:])
        elif line and (line[0].isdigit() or line[0] in '+-*'):
            parts=line.split(); assert len(parts)>=len(positions)
            for i,t in enumerate(parts[:len(positions)]):
                if t=='*': pass
                elif t[0] in '+-': previous[i]+=int(t,0)
                else: previous[i]=int(t,0)
            values=collections.Counter(dict(zip(events,map(int,parts[len(positions):]))))
            addr=previous[positions.index('instr')]
            assert current is not None
            if pending is not None:
                sites[current][(addr,callee)].update(values)
                edges[(current,addr,callee)]+=pending; pending=None
            else:
                own[current].update(values); addresses[current][addr].update(values)
    def resolve(k): return names[k] if isinstance(k,int) else k
    return {
      'own':{resolve(k):dict(v) for k,v in own.items()},
      'sites':{resolve(k):[{'address':a,'callee':resolve(c),'calls':edges[k,a,c],**v} for (a,c),v in s.items()] for k,s in sites.items()},
      'addresses':{resolve(k):{a:dict(v) for a,v in s.items()} for k,s in addresses.items()},
      'summary':summary,
    }

def select(p, pattern):
    keys=[k for k in p['own'] if re.search(pattern,k)]
    assert len(keys)==1,(pattern,keys)
    return keys[0]

def totals(p, name):
    own=collections.Counter(p['own'][name]); inclusive=own.copy()
    for site in p['sites'].get(name,[]): inclusive.update({k:site[k] for k in ('Ir','Dr','Dw') if k in site})
    return {'own':{k:own[k] for k in ('Ir','Dr','Dw')},'inclusive':{k:inclusive[k] for k in ('Ir','Dr','Dw')}}

def asm(path):
    result={}
    for line in path.read_text().splitlines():
        m=re.match(r'^\s*([0-9a-f]+):\s*(.*)',line)
        if not m: continue
        body=m[2]
        body=re.sub(r'^(?:[0-9a-f]{2}\s+)+','',body)
        if not body or re.fullmatch(r'[0-9a-f ]+',body): continue
        result[int(m[1],16)]=body
    return result

def classify(p, name):
    result=collections.Counter()
    for values in p['addresses'][name].values():
        if values.get('Ir',0): result[values['Ir']]+=1
    return result

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--evidence-root',type=pathlib.Path,default=pathlib.Path(__file__).parent/'evidence');ap.add_argument('--output',type=pathlib.Path,default=pathlib.Path('replay-results.json'));args=ap.parse_args()
    root=args.evidence_root
    manifest=json.loads((pathlib.Path(__file__).parent/'manifest.json').read_text())
    for e in manifest:
        p=root/e['local'].removeprefix('evidence/'); b=p.read_bytes()
        assert hashlib.sha1(b'blob '+str(len(b)).encode()+b'\0'+b).hexdigest()==e['blob'],str(p)
    out={'verified_blobs':len(manifest),'components':{}}
    for component,mod,cls in [('scanner','ctor2su','Ctor2su'),('descriptor','ctor2d','Ctor2d')]:
        folder=next(p for p in (root/component/'docs/journal').iterdir() if p.is_dir()); cg=folder/'callgrind'
        c={}; allprofiles={}
        for variant,filename,module,klass in [('baseline','v2','ctor2','Ctor2'),('candidate','su' if component=='scanner' else 'd',mod,cls)]:
            runs=[]
            for run in (1,2):
                p=profile(cg/f'only-{filename}-run{run}.callgrind.gz'); allprofiles[variant,run]=p
                build=select(p,rf'::{klass}>::build::<false>$'); refill=select(p,rf'::{module}::W<false>>::refill$'); finish=select(p,rf'::{module}::W<false>>::finish$')
                canon=select(p,r'::canonicalise$')
                runs.append({'functions':{k:totals(p,n) for k,n in [('build',build),('refill',refill),('finish',finish),('canonicalise',canon)]},'build_sites':p['sites'][build],'process':{k:p['summary'][k] for k in ('Ir','Dr','Dw')}})
            c[variant]=runs
        for fn in ['build','refill','finish','canonicalise']:
            for variant in ['baseline','candidate']:
                assert c[variant][0]['functions'][fn]==c[variant][1]['functions'][fn],(component,variant,fn)
        for run in (1,2):
            pb=allprofiles['baseline',run]; pc=allprofiles['candidate',run]
            nb=select(pb,r'::Ctor2>::build::<false>$');nc=select(pc,rf'::{cls}>::build::<false>$')
            rb=select(pb,r'::ctor2::W<false>>::refill$');rc=select(pc,rf'::{mod}::W<false>>::refill$')
            for key,bn,cn in [('build',nb,nc),('refill',rb,rc)]:
                b=classify(pb,bn);d=classify(pc,cn)
                c[f'{key}_classes_run{run}']=[{'frequency':n,'baseline':b[n],'candidate':d[n],'delta':(d[n]-b[n])*n} for n in sorted(b.keys()|d.keys(),reverse=True) if b[n]!=d[n]]
        c['counters']=[json.loads(l) for l in (folder/'corpora.jsonl').read_text().splitlines() if l.strip()]
        # Every profiled address belongs to the archived disassembly, and its row
        # contributes exactly once; call costs are kept separately above.
        for variant,prefix in [('baseline','v2'),('candidate',mod)]:
            file=folder/'listings'/('v2-build-false-same-binary.asm' if variant=='baseline' else f'{mod}-build-false.asm')
            listing=asm(file);p=allprofiles[variant,1];n=select(p,rf'::{"Ctor2" if variant=="baseline" else cls}>::build::<false>$')
            missing=set(p['addresses'][n])-set(listing)
            assert not missing,(component,variant,missing)
            c[f'{variant}_static_build_instructions']=len(listing)
        out['components'][component]=c
    args.output.write_text(json.dumps(out,indent=2)+'\n')
    print('Verified',len(manifest),'pinned blobs; replayed eight profiles.')
    for component,c in out['components'].items():
        print(component)
        for fn in ['build','refill','finish','canonicalise']:
            a=c['baseline'][0]['functions'][fn];b=c['candidate'][0]['functions'][fn]
            print(fn,'own',a['own'],'->',b['own'],'inclusive Ir',a['inclusive']['Ir'],'->',b['inclusive']['Ir'])
        for fn in ['build','refill']:
            rows=c[f'{fn}_classes_run1'];print(fn,'class delta',sum(r['delta'] for r in rows))
if __name__=='__main__':main()
