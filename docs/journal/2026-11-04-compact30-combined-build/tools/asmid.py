#!/usr/bin/env python3
"""S62-F01 remedy: the uncounted listing's identity, pinned binary against counted-only change.

    python3 asmid.py PINNED_BINARY NEW_BINARY OUT_DIR ROOT_SYMBOL [ROOT_SYMBOL ...]
        [--archived KEY=FILE ...]

From each ROOT (a demangled symbol, e.g. `<c30walk::ctor2su::Ctor2su>::build::<false>`) the
closure of direct callees is followed transitively in both binaries: every `call`/`jmp` whose
target is outside the function and is a symbol of the binary (GOT calls such as memset are
compared by their symbol, not followed). Every function in the closure is disassembled over
its symbol extent (`nm -C -S`; objdump AT&T, demangled, no raw bytes) and compared after the
normalisation of bench/c30_build_listing.py, with one addition: a comment's symbol (minus
its +offset) is kept, so a GOT callee or a named static that changes is a difference.
- addresses dropped; a branch inside the function becomes its offset from the start;
- a branch or call outside it becomes the callee's symbol;
- RIP-relative displacements masked; sizes must be equal.
The two closures must be the same set of symbols. With --archived KEY=FILE, FILE (an archived
listing with `# size`) is also compared with this NEW binary's function KEY (a symbol).
Writes OUT_DIR/identity.json and OUT_DIR/<n>.diff for any difference; prints `identical`
or `DIFFERENT`. Exit status 0 when identical.
"""
import json, os, re, subprocess, sys

INSN = re.compile(r"^ *([0-9a-f]+):\t(.*)$")
TARGET = re.compile(r"\b([0-9a-f]+) <(.+)>$")
COMMENT = re.compile(r"#\s*[0-9a-f]+ <(.+?)(\+0x[0-9a-f]+)?>\s*$")
SIZE_LINE = re.compile(r"^# size 0x([0-9a-f]+)$", re.M)


def symbols(binary):
    """{start: (key, size)} for every text symbol, and {key: start}. A name held by several
    local symbols is keyed by its name, size and ordinal (by address) among those"""
    out = subprocess.run(["nm", "-C", "-S", binary], capture_output=True, text=True, check=True)
    rows = {}
    for line in out.stdout.splitlines():
        p = line.split(None, 3)
        if len(p) == 4 and p[2] in "tTwW":
            rows.setdefault(p[3], set()).add((int(p[0], 16), int(p[1], 16)))
    by_addr = {}
    for name, hits in rows.items():
        hits = sorted(hits)
        for start, size in hits:
            if len(hits) == 1:
                key = name
            else:
                same = [h for h in hits if h[1] == size]
                key = f"{name} [size 0x{size:x} #{same.index((start, size))}]"
            by_addr[start] = (key, size)
    return by_addr, {k: a for a, (k, _) in by_addr.items()}


def disassemble(binary, start, size):
    args = ["objdump", "-d", "-C", "--no-show-raw-insn", f"--start-address=0x{start:x}",
            f"--stop-address=0x{start + size:x}", binary]
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout


def instructions(text):
    out = []
    for line in text.splitlines():
        m = INSN.match(line)
        if m:
            out.append((int(m.group(1), 16), m.group(2)))
    return out


def normalise(ins, start, size, by_addr=None):
    out, callees = [], set()
    for _, t in ins:
        c = COMMENT.search(t)
        note = f" ;{c.group(1)}" if c else ""
        # objdump's comment starts after whitespace ("{closure#0}" is part of a name)
        t = re.sub(r"\s+#\s.*$", "", t).rstrip()
        m = TARGET.search(t)
        if m:
            a = int(m.group(1), 16)
            if start <= a < start + size:
                rep = "T%+d" % (a - start)
            else:
                name = re.sub(r"\+0x[0-9a-f]+$", "", m.group(2))
                if by_addr is not None and a in by_addr:
                    name = by_addr[a][0]
                    if t.split()[0] == "call" or t.split()[0].startswith("j"):
                        callees.add(a)
                rep = "SYM<%s>" % name
            t = t[: m.start()] + rep
        t = re.sub(r"-?0x[0-9a-f]+\(%rip\)", "DISP(%rip)", t)
        out.append(" ".join(t.split()) + note)
    return out, callees


def closure(binary, roots):
    by_addr, by_key = symbols(binary)
    todo, res = [by_key[r] for r in roots], {}
    seen = set()
    while todo:
        start = todo.pop()
        if start in seen:
            continue
        seen.add(start)
        key, size = by_addr[start]
        text = disassemble(binary, start, size)
        norm, callees = normalise(instructions(text), start, size, by_addr)
        res[key] = {"size": size, "norm": norm, "text": text}
        todo.extend(sorted(callees - seen))
    return res


def main():
    argv = sys.argv[1:]
    archived = []
    while "--archived" in argv:
        i = argv.index("--archived")
        archived.append(argv[i + 1].split("=", 1))
        del argv[i : i + 2]
    pinned, new, out_dir, *roots = argv
    os.makedirs(out_dir, exist_ok=True)
    a, b = closure(pinned, roots), closure(new, roots)
    rep = {"roots": roots, "functions": {}, "identical": True}
    if set(a) != set(b):
        rep["identical"] = False
        rep["closure_only_pinned"] = sorted(set(a) - set(b))
        rep["closure_only_new"] = sorted(set(b) - set(a))
    for k, n in enumerate(sorted(set(a) & set(b))):
        x, y = a[n], b[n]
        if "error" in x or "error" in y:
            rep["functions"][n] = {"error": [x.get("error"), y.get("error")]}
            rep["identical"] = False
            continue
        diff = [] if x["size"] == y["size"] else [f"size 0x{x['size']:x} / 0x{y['size']:x}"]
        diff += [f"{i}: {p!r} / {q!r}" for i, (p, q) in enumerate(zip(x["norm"], y["norm"])) if p != q]
        if len(x["norm"]) != len(y["norm"]):
            diff.append(f"length {len(x['norm'])} / {len(y['norm'])}")
        rep["functions"][n] = {"size": y["size"], "instructions": len(y["norm"]), "identical": not diff}
        if diff:
            rep["identical"] = False
            open(os.path.join(out_dir, f"{k}.diff"), "w").write(n + "\n" + "\n".join(diff) + "\n")
    rep["archived"] = {}
    for n, f in archived:
        t = open(f).read()
        m = SIZE_LINE.search(t)
        ins = instructions(t)
        ref, _ = normalise(ins, ins[0][0], int(m.group(1), 16))
        y = b[n]
        yi = instructions(y["text"])
        got, _ = normalise(yi, yi[0][0], y["size"])  # both by objdump's names
        same = ref == got and int(m.group(1), 16) == y["size"]
        rep["archived"][n] = {"file": os.path.basename(f), "instructions": len(ref), "identical": same}
        rep["identical"] &= same
    rep["closure_size"] = len(b)
    rep["closure_instructions"] = sum(v.get("instructions", 0) for v in rep["functions"].values())
    json.dump(rep, open(os.path.join(out_dir, "identity.json"), "w"), indent=1)
    print("identical" if rep["identical"] else "DIFFERENT")
    return 0 if rep["identical"] else 1


if __name__ == "__main__":
    sys.exit(main())
