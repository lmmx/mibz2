"""C6 of docs/journal/2026-11-01-decoder-attribution-protocol.md: the default-feature
decoder is unchanged by the trace's marks. The same example (examples/attribution_decode.rs)
built against the base commit's library and against this branch's, both with default
features: every function whose name holds `mibz2` must be present in both and disassemble
the same, once addresses, jump and call targets' addresses, and rip-relative offsets are
written out of it. Exits 1 on any difference; the report lists them.

    attribution_same_code.py BASE_BINARY HEAD_BINARY --out report.json
"""
import argparse
import json
import re
import subprocess
import sys

FUNC = re.compile(r"^[0-9a-f]+ <(.+)>:$")
LINE = re.compile(r"^\s+[0-9a-f]+:\t(.*)$")


def functions(binary):
    text = subprocess.check_output(["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)
    out, cur = {}, None
    for line in text.splitlines():
        m = FUNC.match(line)
        if m:
            cur = m.group(1) if "mibz2" in m.group(1) else None
            if cur is not None:
                out.setdefault(cur, [])
            continue
        m = LINE.match(line)
        if m and cur is not None:
            ins = m.group(1).split("#")[0].strip()
            ins = re.sub(r"\b[0-9a-f]{4,}\b <([^>+]+)(\+0x[0-9a-f]+)?>", r"<\1\2>", ins)  # call/jmp targets
            ins = re.sub(r"\[rip\+0x[0-9a-f]+\]", "[rip+X]", ins)
            ins = re.sub(r"\b[0-9a-f]{4,}\b", "A", ins) if ins.split()[0].startswith("j") or ins.startswith("call") else ins
            out[cur].append(ins)
    return out


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("base")
    ap.add_argument("head")
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    b, h = functions(a.base), functions(a.head)
    only_b, only_h = sorted(set(b) - set(h)), sorted(set(h) - set(b))
    differ = sorted(n for n in set(b) & set(h) if b[n] != h[n])
    ok = not (only_b or only_h or differ)
    rep = dict(same=ok, functions=len(set(b) & set(h)), instructions=sum(len(v) for v in h.values()),
               only_in_base=only_b, only_in_head=only_h, differ=differ)
    open(a.out, "w").write(json.dumps(rep, indent=1) + "\n")
    print(f"same code: {'yes' if ok else 'NO'}: {rep['functions']} mibz2 functions, {rep['instructions']} instructions"
          + ("" if ok else f"; base only {len(only_b)}, head only {len(only_h)}, differing {len(differ)}"))
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
