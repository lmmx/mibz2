import sys,re
sys.path.insert(0,'bench')
import c30_build_listing as L
B='target/rust-1.97.0/release/examples/c30walk'
name=sys.argv[1]
syms=L.symbols(B)
st,sz,_=[s for s in syms if s[2]==name][0]
ins=L.instructions(L.disassemble(B,st,sz))
print(name, hex(st), hex(sz), len(ins))
for k,(a,t) in enumerate(ins):
    m=L.TARGET.search(t)
    if t.startswith(('jne','jb ','ja ','jae','jbe')) and m:
        tgt=int(m.group(1),16)
        if tgt<a:
            body=[x for x in ins if tgt<=x[0]<=a]
            txt=' | '.join(x[1] for x in body)
            if ('addq   $0x2' in txt or 'add    $0x2' in txt) and 'movzbl' in txt and len(body)<20:
                print(f"  loop +{hex(tgt-st)}..+{hex(a-st)}: {len(body)} instr: {txt}")
