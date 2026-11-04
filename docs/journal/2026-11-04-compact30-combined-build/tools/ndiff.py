#!/usr/bin/env python3
"""Gate 2: a normalised instruction diff of two archived listings (`# size` format, objdump
AT&T), from different binaries.

    python3 ndiff.py A.asm B.asm [--label A,B] > OUT.diff

Normalisation (bench/c30_build_listing.py's, as asmid.py applies it, plus two masks so that
listings from different binaries align): addresses dropped; a branch inside the function
becomes `T` (its offset dropped, since code moves); a branch or call outside it becomes the
callee's symbol with the module path masked (`ctor2*` → `ctorX`); RIP-relative
displacements masked; objdump's comments dropped. Registers and stack offsets are kept, so
register-allocation and stack-slot changes show. Prints a unified diff, then counts:
instructions in A and B, equal, removed (A only) and added (B only), and the stack loads and
stores (`(%rsp)` operands) in each.
"""
import difflib, re, sys
INSN = re.compile(r"^ *([0-9a-f]+):\t(.*)$")
TARGET = re.compile(r"\b([0-9a-f]+) <(.+)>$")
SIZE = re.compile(r"^# size 0x([0-9a-f]+)$", re.M)

def load(path):
    t = open(path).read()
    ins = [(int(m.group(1), 16), m.group(2)) for l in t.splitlines() if (m := INSN.match(l))]
    start = ins[0][0]
    m = SIZE.search(t)
    # a listing without `# size` (B2's at cd14fa9): its extent is through its last instruction
    size = int(m.group(1), 16) if m else ins[-1][0] - start + 16
    out = []
    for a, x in ins:
        x = re.sub(r"\s+#\s.*$", "", x).rstrip()
        m = TARGET.search(x)
        if m:
            tgt = int(m.group(1), 16)
            if start <= tgt < start + size:
                rep = "T"
            else:
                rep = "SYM<" + re.sub(r"ctor[0-9a-z]*\b", "ctorX", re.sub(r"Ctor[0-9a-z]*\b", "CtorX", re.sub(r"\+0x[0-9a-f]+$", "", m.group(2)))) + ">"
            x = x[: m.start()] + rep
        x = re.sub(r"-?0x[0-9a-f]+\(%rip\)", "DISP(%rip)", x)
        out.append((a - start, " ".join(x.split())))
    return out

def stack(ins):
    ld = st = 0
    for _, x in ins:
        if "(%rsp" not in x:
            continue
        op, _, args = x.partition(" ")
        if op.startswith(("lea", "push", "pop", "call")):
            continue
        dst = args.rsplit(",", 1)[-1]
        if "(%rsp" in dst and "," in args:
            st += 1
            if op.startswith(("add", "sub", "inc", "dec", "or", "and", "xor")):
                ld += 1
        else:
            ld += 1
    return ld, st

a_path, b_path = sys.argv[1:3]
labels = sys.argv[sys.argv.index("--label") + 1].split(",") if "--label" in sys.argv else [a_path, b_path]
A, B = load(a_path), load(b_path)
sm = difflib.SequenceMatcher(None, [x for _, x in A], [x for _, x in B], autojunk=False)
eq = rem = add = 0
for op, i1, i2, j1, j2 in sm.get_opcodes():
    if op == "equal":
        eq += i2 - i1
        continue
    rem += i2 - i1; add += j2 - j1
    print(f"@@ {op} {labels[0]}[{i1}:{i2}] +{A[i1][0] if i1 < len(A) else 0:#x} | {labels[1]}[{j1}:{j2}] +{B[j1][0] if j1 < len(B) else 0:#x}")
    for o, x in A[i1:i2]:
        print(f"- +{o:#06x} {x}")
    for o, x in B[j1:j2]:
        print(f"+ +{o:#06x} {x}")
la, sa = stack(A); lb, sb = stack(B)
print(f"# {labels[0]}: {len(A)} instructions, {la} stack loads, {sa} stack stores; "
      f"{labels[1]}: {len(B)} instructions, {lb} stack loads, {sb} stack stores; "
      f"equal {eq}, removed {rem}, added {add}")
