#!/usr/bin/env python3
"""Static replay only: parse archived Callgrind instruction counts and assembly.

No constructor is compiled or run. Paths are relative to the immutable evidence root.
Memory totals classify explicit assembly operands; implicit call/return/push/pop
traffic is reported separately. They are not a cache or performance model.
"""
import argparse, collections, gzip, json, pathlib, re, hashlib

SCAN = pathlib.Path('docs/journal/2026-11-04-compact30-scanner-prototype')
B2 = pathlib.Path('docs/journal/2026-11-03-compact30-constructor-v2-prototype')

def asm(path):
    out = {}
    for ln, line in enumerate(path.read_text().splitlines(), 1):
        m = re.match(r'\s*([0-9a-f]+):\s+(.+)', line)
        if m:
            out[int(m[1],16)] = {'line':ln, 'text':m[2]}
    return out

def cg(path):
    lines = gzip.open(path, 'rt').read().splitlines()
    names = {}
    for line in lines:
        m = re.match(r'(?:c?fn)=\((\d+)\) (.+)', line)
        if m: names[m[1]] = m[2]
    counts = collections.defaultdict(collections.Counter)
    calls = collections.defaultdict(collections.Counter)
    function = None
    target = None
    previous = [0,0]
    pending = False
    positions = events = None
    for line in lines:
        if line.startswith('positions:'): positions = line.split()[1:]
        elif line.startswith('events:'): events = line.split()[1:]
        elif line.startswith('fn='):
            m = re.match(r'fn=\((\d+)\)', line)
            function = names[m[1]] if m else line[3:]
        elif line.startswith('cfn='):
            m = re.match(r'cfn=\((\d+)\)', line)
            target = names[m[1]] if m else line[4:]
        elif line.startswith('calls='): pending = True
        elif line and (line[0].isdigit() or line[0] in '+-*'):
            fields = line.split()
            for i, token in enumerate(fields[:len(positions)]):
                if token != '*':
                    previous[i] = (previous[i]+int(token,0)) if token[0] in '+-' else int(token,0)
            costs = list(map(int,fields[len(positions):]))
            assert events == ['Ir'], events
            if pending: calls[function][target] += costs[0]
            else: counts[function][previous[0]] += costs[0]
            pending = False
    return counts, calls

def operands(s):
    return re.split(r',(?![^()]*\))', s)

def memops(text):
    text = text.split('#')[0].strip()
    parts = text.split(None,1)
    op = parts[0]
    if 'nop' in text or op in ('lea','xchg') and ('%ax,%ax' in text or op=='lea'):
        return 0,0,0,0,0,0
    args = operands(parts[1]) if len(parts)>1 else []
    mem = [i for i,a in enumerate(args) if '(' in a]
    reads = writes = sr = sw = ir = iw = 0
    if op.startswith('push'): ir,iw = 0,1
    elif op.startswith('pop'): ir,iw = 1,0
    elif op.startswith('call'): ir,iw = 0,1
    elif op.startswith('ret'): ir,iw = 1,0
    for i in mem:
        is_dest = i == len(args)-1
        r,w = 1,0
        if is_dest and not op.startswith(('cmp','test','bt','call','jmp','push')):
            if op.startswith(('mov','set','pop')): r,w=0,1
            else: r,w=1,1
        reads += r; writes += w
        if '%rsp' in args[i]: sr+=r; sw+=w
    return reads,writes,sr,sw,ir,iw

def normalize(instructions):
    addresses = {a:i for i,a in enumerate(instructions)}
    result = []
    for rec in instructions.values():
        s = rec['text']
        m = re.match(r'(j\w+|call)\s+([0-9a-f]+)\s+<(.+)>',s)
        if m:
            t=int(m[2],16)
            s=m[1]+' '+('I'+str(addresses[t]) if t in addresses else m[3])
        s = re.sub(r'-?0x[0-9a-f]+\(%rip\).*', 'RIP_RELOCATION', s)
        s = s.replace('ctor2s::','ctor2::')
        result.append(s)
    return result

def main():
    p=argparse.ArgumentParser();p.add_argument('evidence_root',type=pathlib.Path,nargs='?');p.add_argument('--root',type=pathlib.Path);p.add_argument('--output',type=pathlib.Path)
    args=p.parse_args(); root=args.root or args.evidence_root
    if root is None:p.error('provide --root EVIDENCE_ROOT')
    counts,calls=cg(root/SCAN/'callgrind/per-instruction.callgrind.gz')
    report={'conventions':[
      'Archived Ir only; no constructor execution or new capture.',
      'Self excludes call-edge inclusive costs; inclusive adds each direct call edge once.',
      'Explicit memory counts are architectural operands: RMW is one read plus one write, LEA and NOP are no memory access.',
      'Direct rsp counts require an explicit %rsp-based operand; stack accesses through aliased registers are not classified as rsp.',
      'push/call implicitly write, pop/ret implicitly read the stack; reported separately from explicit rsp operands.',
      'These absolute memory counts are not Callgrind Dr/Dw semantics. Their scanner-v2 differences match the saved self Dr/Dw differences, but missing raw Dr/Dw cannot be reproduced.',
      'Normalization erases instruction addresses, branch destinations to instruction ordinals, and RIP-relative displacements/comments. It does not verify constant bytes or external callee binaries.'
    ]}; listing_map={
      'v2':('ctor2','Ctor2','v2-build-false-same-binary.asm'),
      'scanner':('ctor2s','Ctor2s','ctor2s-build-false.asm')}
    rows=[json.loads(l) for l in (root/SCAN/'corpora.jsonl').read_text().splitlines()]
    primary=[r for r in rows if r['input']=='wikidata-20261026.bz2' and r['permille']==0 and not r['extra_starts']]
    assert len(primary)==1
    report['primary_counters']=primary[0]
    report['functions']={}
    for short,(mod,cls,file) in listing_map.items():
        name=f'<c30walk::{mod}::{cls}>::build::<false>'
        listing=asm(root/SCAN/'listings'/file)
        missing=sorted(set(counts[name])-set(listing)); assert not missing,missing
        totals=collections.Counter(); hot=[]
        for addr,n in counts[name].items():
            r,w,sr,sw,ir,iw=memops(listing[addr]['text'])
            totals.update({'Ir':n,'explicit_reads':r*n,'explicit_writes':w*n,'rsp_reads':sr*n,'rsp_writes':sw*n,'implicit_reads':ir*n,'implicit_writes':iw*n})
            if sr or sw:hot.append({'address':hex(addr),'line':listing[addr]['line'],'count':n,'text':listing[addr]['text'],'read':sr,'write':sw})
        report['functions'][short]={'self':dict(totals),'inclusive_Ir':sum(counts[name].values())+sum(calls[name].values()),'calls_inclusive_Ir':dict(calls[name]),'hot_rsp':sorted(hot,key=lambda x:x['count'],reverse=True)}
    old=asm(root/B2/'ctor2-build-false.asm'); new=asm(root/SCAN/'listings/v2-build-false-same-binary.asm')
    report['B2_bridge']={'old_instruction_count':len(old),'new_instruction_count':len(new),'equal_after_address_and_rip_relocation_normalization':normalize(old)==normalize(new),'limitation':'RIP data values and external callee binaries are not proven equal by normalization.'}
    report['self_delta_saved']={k:report['functions']['v2']['self'][k]-report['functions']['scanner']['self'][k] for k in report['functions']['v2']['self']}
    report['inclusive_delta_saved']=report['functions']['v2']['inclusive_Ir']-report['functions']['scanner']['inclusive_Ir']
    def region(file,name,lo,hi):
        a=asm(root/SCAN/'listings'/file)
        seq=[(ad,x) for ad,x in a.items() if lo<=ad<=hi]
        return {'first':hex(lo),'last':hex(hi),'instructions':len(seq),
                'weighted_Ir':sum(counts[name][ad] for ad,x in seq),
                'frequencies':sorted({counts[name][ad] for ad,x in seq}),
                'lines':[x['line'] for ad,x in seq]}
    vn='<c30walk::ctor2::Ctor2>::build::<false>';sn='<c30walk::ctor2s::Ctor2s>::build::<false>'
    vf='v2-build-false-same-binary.asm';sf='ctor2s-build-false.asm'
    report['fulltile_refill_wrappers']={
      'first':{'v2':region(vf,vn,0x3babe,0x3baec),'scanner':region(sf,sn,0x41bae,0x41bf2)},
      'second':{'v2':region(vf,vn,0x3bb4d,0x3bb77),'scanner':region(sf,sn,0x41c6e,0x41ca0)}}
    report['fulltile_normal_row_loops']={
      'first':{'v2':region(vf,vn,0x3ba90,0x3baba),'scanner':region(sf,sn,0x41b80,0x41baa)},
      'second':{'v2':region(vf,vn,0x3bb20,0x3bb46),'scanner':region(sf,sn,0x41c40,0x41c67)}}
    a=asm(root/SCAN/'listings'/sf)
    report['proposal_existing_site_exposure']={}
    for key,sites in {'U1_U2':[0x41b0c,0x41b12,0x41b15,0x41c00,0x41c04,0x41c07,0x41c0d,0x41c15], 'U3':[0x41b23,0x41b27]}.items():
        report['proposal_existing_site_exposure'][key]={'weighted_Ir':sum(counts[sn][ad] for ad in sites),'sites':[{'address':hex(ad),'line':a[ad]['line'],'count':counts[sn][ad],'text':a[ad]['text']} for ad in sites]}
    report['callees_B2_bridge']={}
    for callee in ['finish','refill']:
        old=asm(root/B2/f'ctor2-{callee}-false.asm');new=asm(root/SCAN/'listings'/f'ctor2s-{callee}-false.asm')
        report['callees_B2_bridge'][callee]={'old_count':len(old),'new_count':len(new),'normalized_equal':normalize(old)==normalize(new)}
    report['earlier_attempts']={}
    for attempt in ['attempt-1','attempt-2']:
        aa=asm(root/SCAN/attempt/'ctor2s-build-false.asm')
        report['earlier_attempts'][attempt]={'instruction_count':len(aa),'fulltile_first_loop':sum(0x41910<=ad<=0x41942 for ad in aa),'fulltile_second_loop':sum(0x419a0<=ad<=0x419d3 for ad in aa)}
    consumed=[SCAN/'callgrind/per-instruction.callgrind.gz',SCAN/'corpora.jsonl',B2/'ctor2-build-false.asm']
    consumed += [SCAN/'listings'/v[2] for v in listing_map.values()]
    consumed += [base/file for base,file in [(B2,'ctor2-finish-false.asm'),(B2,'ctor2-refill-false.asm'),(SCAN/'listings','ctor2s-finish-false.asm'),(SCAN/'listings','ctor2s-refill-false.asm')]]
    consumed += [SCAN/attempt/'ctor2s-build-false.asm' for attempt in ['attempt-1','attempt-2']]
    report['inputs_sha256']={str(path):hashlib.sha256((root/path).read_bytes()).hexdigest() for path in consumed}
    text=json.dumps(report,indent=2,sort_keys=True)+'\n'
    if args.output:args.output.write_text(text)
    else:print(text,end='')

if __name__=='__main__':main()
