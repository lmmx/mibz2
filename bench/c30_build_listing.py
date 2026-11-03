#!/usr/bin/env python3
"""Gate G3's machine-code identity for the construction-cost comparison
(docs/journal/2026-11-03-compact30-construction-protocol.md, §6).

    python3 bench/c30_build_listing.py BINARY OUT_DIR REFERENCE_DIR [--record KEY,...]

Disassembles the timed path's functions from BINARY (objdump, AT&T syntax, demangled, no
raw bytes), each over its own symbol extent [start, start + nm -S size):
- `<c30walk::ctor::Ctor>::build::<false>` and its three out-of-line callees,
  `W<false>::promote`, `W<false>::complete_checked` and `canonicalise`: the accepted
  constructor;
- `c30walk::cbuild::c_build` and `c30walk::cbuild::p_build`: the arms' timed wrappers;
- `mibz2::bwt::build`: production's public builder, with `build_one_pass` (and
  `build_two_pass`) inlined.

Each is compared with REFERENCE_DIR/<key>.asm after normalisation:
- addresses dropped;
- a branch inside the function becomes its offset from the function's start;
- a call or branch outside it becomes the callee's symbol;
- RIP-relative displacements and objdump's comments masked.

The sizes must be equal (the reference's `# size` line). The references are:
- for the constructor's four functions and `mibz2::bwt::build`, the listings of the
  reviewed pin's binary (`e003a32`, built with the same toolchain), recorded with
  `--record` from that binary;
- for the two wrappers, which only the harness has, this harness's own first build,
  recorded for review.

Also checked on the binary itself:
- `p_build` only transfers to `mibz2::bwt::build`;
- `c_build` only calls `Ctor::build::<false>`;
- `mibz2::bwt::build` calls no `build_one_pass` or `build_two_pass` symbol, and no
  out-of-line symbol of either exists (both are inlined).

Writes OUT_DIR/<key>.asm (and <key>-lines.asm, with source lines, for review),
OUT_DIR/<key>.diff (empty when identical) and OUT_DIR/listing.json, and prints `yes` when
every comparison and check passes, otherwise `no`.
"""

import json
import os
import re
import subprocess
import sys

FUNCS = {
    "ctor-build-false": "<c30walk::ctor::Ctor>::build::<false>",
    "ctor-promote": "<c30walk::ctor::W<false>>::promote",
    "ctor-complete-checked": "<c30walk::ctor::W<false>>::complete_checked",
    "ctor-canonicalise": "c30walk::ctor::canonicalise",
    "c-build": "c30walk::cbuild::c_build",
    "p-build": "c30walk::cbuild::p_build",
    "bwt-build": "mibz2::bwt::build",
}

INSN = re.compile(r"^ *([0-9a-f]+):\t(.*)$")
TARGET = re.compile(r"\b([0-9a-f]+) <(.+)>$")
SIZE_LINE = re.compile(r"^# size 0x([0-9a-f]+)$", re.M)


def instructions(text):
    """(address, instruction) for every instruction line; everything else (objdump's
    headers, -l source lines, function labels) dropped; comments removed"""
    out = []
    for line in text.splitlines():
        m = INSN.match(line)
        if m:
            out.append((int(m.group(1), 16), re.sub(r"\s*#.*$", "", m.group(2)).rstrip()))
    return out


def normalise(ins, start, size):
    out = []
    for _, t in ins:
        m = TARGET.search(t)
        if m:
            a = int(m.group(1), 16)
            if start <= a < start + size:
                rep = "T%+d" % (a - start)
            else:
                rep = "SYM<%s>" % re.sub(r"\+0x[0-9a-f]+$", "", m.group(2))
            t = t[: m.start()] + rep
        t = re.sub(r"-?0x[0-9a-f]+\(%rip\)", "DISP(%rip)", t)
        out.append(" ".join(t.split()))
    return out


def compare(got_text, got_size, ref_text, ref_size):
    g, r = instructions(got_text), instructions(ref_text)
    diff = []
    if got_size != ref_size:
        diff.append(f"size: reference 0x{ref_size:x} / this binary 0x{got_size:x}")
    if not g or not r:
        return diff + ["no instructions"]
    gn, rn = normalise(g, g[0][0], got_size), normalise(r, r[0][0], ref_size)
    diff += [
        f"{i}: reference {a!r} / this binary {b!r}"
        for i, (a, b) in enumerate(zip(rn, gn))
        if a != b
    ]
    if len(gn) != len(rn):
        diff.append(f"length: reference {len(rn)} / this binary {len(gn)} instructions")
    return diff


def symbols(binary):
    out = subprocess.run(["nm", "-C", "-S", binary], capture_output=True, text=True, check=True)
    rows = []
    for line in out.stdout.splitlines():
        p = line.split(None, 3)
        if len(p) == 4:
            rows.append((int(p[0], 16), int(p[1], 16), p[3]))
    return rows


def disassemble(binary, start, size, lines=False):
    args = ["objdump", "-d", "-C", "--no-show-raw-insn"] + (["-l"] if lines else [])
    args += [f"--start-address=0x{start:x}", f"--stop-address=0x{start + size:x}", binary]
    return subprocess.run(args, capture_output=True, text=True, check=True).stdout


def main():
    argv = sys.argv[1:]
    record = set()
    if "--record" in argv:
        i = argv.index("--record")
        record = set(argv[i + 1].split(","))
        del argv[i : i + 2]
    binary, out_dir, ref_dir = argv[:3]
    syms = symbols(binary)
    res = {"functions": {}, "checks": {}}
    ok = True
    texts = {}
    for key, name in FUNCS.items():
        hits = [s for s in syms if s[2] == name]
        if len(hits) != 1:
            if record:
                continue
            print(f"c30_build_listing: {len(hits)} symbols named {name}", file=sys.stderr)
            return 2
        start, size, _ = hits[0]
        text = disassemble(binary, start, size)
        texts[key] = (start, size, text)
        with open(os.path.join(out_dir, f"{key}.asm"), "w") as f:
            f.write(f"# {name}\n# size 0x{size:x}\n" + text)
        with open(os.path.join(out_dir, f"{key}-lines.asm"), "w") as f:
            f.write(disassemble(binary, start, size, lines=True))
        info = {"symbol": name, "size": size, "instructions": len(instructions(text))}
        ref = os.path.join(ref_dir, f"{key}.asm")
        if key in record:
            with open(ref, "w") as f:
                f.write(f"# {name}\n# size 0x{size:x}\n" + text)
        if os.path.exists(ref):
            rtext = open(ref).read()
            m = SIZE_LINE.search(rtext)
            diff = compare(text, size, rtext, int(m.group(1), 16) if m else -1)
            info["reference"] = os.path.relpath(ref)
            info["identical"] = not diff
            ok &= not diff
        else:
            diff = [f"no reference {ref}"]
            info["identical"] = False
            ok = False
        with open(os.path.join(out_dir, f"{key}.diff"), "w") as f:
            f.write("".join(d + "\n" for d in diff))
        res["functions"][key] = info
    if record and len(texts) < len(FUNCS):
        print("recorded", ",".join(sorted(k for k in record if k in texts)))
        return 0

    def calls(key):
        return [
            re.sub(r"\+0x[0-9a-f]+$", "", m.group(2))
            for _, t in instructions(texts[key][2])
            if (t.startswith("call") or t.startswith("jmp"))
            and (m := TARGET.search(t))
            and not (texts[key][0] <= int(m.group(1), 16) < texts[key][0] + texts[key][1])
        ]

    c = res["checks"]
    c["p_build_transfers_only_to_bwt_build"] = set(calls("p-build")) == {"mibz2::bwt::build"}
    c["c_build_calls_only_ctor_build_false"] = set(calls("c-build")) == {FUNCS["ctor-build-false"]}
    bwt_calls = calls("bwt-build")
    c["bwt_build_callees"] = sorted(set(bwt_calls))
    c["bwt_build_inlines_one_and_two_pass"] = not any(
        "build_one_pass" in s or "build_two_pass" in s for s in bwt_calls
    ) and not any("build_one_pass" in s[2] or "build_two_pass" in s[2] for s in syms)
    for k in ("p_build_transfers_only_to_bwt_build", "c_build_calls_only_ctor_build_false",
              "bwt_build_inlines_one_and_two_pass"):  # fmt: skip
        ok &= c[k]
    res["identical"] = ok
    with open(os.path.join(out_dir, "listing.json"), "w") as f:
        json.dump(res, f, indent=1)
    print("yes" if ok else "no")
    return 0


if __name__ == "__main__":
    sys.exit(main())
