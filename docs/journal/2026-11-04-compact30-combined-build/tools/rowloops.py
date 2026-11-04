"""The scalar row loops of a build::<false> (backward-branch bodies with a movzbl and an
`addq $0x2`), from the binary; the scanner trial's rowloops.py, with the binary as an
argument.   python3 rowloops.py BINARY SYMBOL [SYMBOL ...]   (run from the worktree root)"""
import sys
sys.path.insert(0, 'bench')
import c30_build_listing as L
B = sys.argv[1]
syms = L.symbols(B)
for name in sys.argv[2:]:
    st, sz, _ = [s for s in syms if s[2] == name][0]
    ins = L.instructions(L.disassemble(B, st, sz))
    print(name, hex(st), hex(sz), len(ins))
    for k, (a, t) in enumerate(ins):
        m = L.TARGET.search(t)
        if t.startswith(('jne', 'jb ', 'ja ', 'jae', 'jbe')) and m:
            tgt = int(m.group(1), 16)
            if tgt < a:
                body = [x for x in ins if tgt <= x[0] <= a]
                txt = ' | '.join(x[1] for x in body)
                if ('addq   $0x2' in txt or 'add    $0x2' in txt) and 'movzbl' in txt and len(body) < 20:
                    print(f"  loop +{hex(tgt-st)}..+{hex(a-st)}: {len(body)} instr: {txt}")
