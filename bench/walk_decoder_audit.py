"""Structural audit W1-W4 of the walk-unroll full-decoder trial's exact binary
(docs/journal/2026-10-31-walk-unroll-decoder-protocol.md §9), before any timing. No timing,
no execution of the binary.

    walk_decoder_audit.py BINARY --base SHA [--out FILE]

- W1: the arm's amendment-1 gates (bench/walk_unroll_gates.py) on
  room_segments_unrolled::<32, 1> in this binary.
- W2: the shipped room_segments::<32, 1> present, its hot step unchanged (16 instructions,
  3 loads, 2 stores, 1 prefetch a lane-step).
- W3: every direct caller of room_segments_unrolled::<32, 1> is on the arm's side (its name
  names SegmentedUnrolled or the unrolled walk), every caller of room_segments::<32, 1> on
  the shipped side; each has at least one.
- W4: functional equivalence outside the walk, as registered.
  (a) source: `git diff BASE HEAD -- src` changes only src/bwt.rs, by insertions only.
  (b) binary: from each arm's worker thread body, the call-graph closure: the stage
      functions reached (MTF, build, RLE1, CRC, assembly, the walk) must be the same
      functions, the walk's own apart. Which per-block driver functions (decode_block,
      the kernel's invert, the assembly) are out of line in each arm is reported, with the
      worker bodies' sizes. Machine-code identity is not required.
  (c) an inlining difference in the per-block driver is reported as "review required";
      the runner refuses the timed trial unless frozen.json records it as reviewed and
      accepted (`w4_review`), with the same signature.
Exits 1 if a gate fails (W4 (b) "stop for review" included).
"""
import argparse
import collections
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))
import walk_asm_audit as w  # noqa: E402
import walk_unroll_gates as g  # noqa: E402

CALL = re.compile(r"<(.+?)(?:\+0x[0-9a-f]+)?>\s*$")
# the walk's own functions: allowed to differ between the arms' call targets
WALK = re.compile(r"room_segments(_unrolled)?::<|walk_segments_room(_unrolled)?(_parts)?\b|"
                  r"SegmentedUnrolled|bwt::Segmented\b")
ARM_SIDE = re.compile(r"SegmentedUnrolled|room_segments_unrolled|walk_segments_room_unrolled")


def all_functions(binary):
    text = subprocess.check_output(
        ["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)
    funcs, cur = {}, None
    for line in text.splitlines():
        m = w.FUNC.match(line)
        if m:
            cur = funcs.setdefault(f"{m.group(2)}@{m.group(1).lstrip('0')}", [])
            continue
        m = w.LINE.match(line)
        if m and cur is not None:
            cur.append((int(m.group(1), 16), m.group(2), m.group(3).split("#")[0].strip()))
    return funcs


def callees(ins):
    out = collections.Counter()
    for _, mn, op in ins:
        if mn.startswith("call") or mn == "jmp":
            m = CALL.search(op)
            if m and mn.startswith("call"):
                out[m.group(1)] += 1
    return out


def base(name):
    return name.split("@")[0]


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("binary")
    ap.add_argument("--base", required=True, help="the source base (master) commit")
    ap.add_argument("--out")
    a = ap.parse_args()
    funcs = all_functions(a.binary)
    report, gates = {}, {}

    # W1
    gate = subprocess.run([sys.executable, "-I", str(Path(__file__).parent / "walk_unroll_gates.py"), a.binary],
                          capture_output=True, text=True)
    w1 = json.loads(gate.stdout)
    report["W1"] = w1
    gates["W1_arm_gates"] = gate.returncode == 0 and w1["pass_"]

    # W2
    shipped = [v for k, v in funcs.items() if base(k) == "mibz2::bwt::room_segments::<32, 1>"]
    arm = [v for k, v in funcs.items() if base(k) == "mibz2::bwt::room_segments_unrolled::<32, 1>"]
    ok2 = len(shipped) == 1 and len(arm) == 1
    if ok2:
        c, per = w.census(w.hot_loop(shipped[0]))
        report["W2"] = dict(per_lane_step=per)
        ok2 = (per["instructions"], per["loads"], per["stores"], per["prefetches"]) == (16.0, 3.0, 2.0, 1.0)
    gates["W2_shipped_walk_unchanged"] = ok2

    # W3
    callers = {"mibz2::bwt::room_segments::<32, 1>": [], "mibz2::bwt::room_segments_unrolled::<32, 1>": []}
    for k, ins in funcs.items():
        for t in callees(ins):
            if t in callers:
                callers[t].append(base(k))
    s_callers, u_callers = callers["mibz2::bwt::room_segments::<32, 1>"], callers["mibz2::bwt::room_segments_unrolled::<32, 1>"]
    report["W3"] = dict(shipped_walk_callers=sorted(set(s_callers)), arm_walk_callers=sorted(set(u_callers)))
    gates["W3_each_arm_its_walk"] = (bool(s_callers) and bool(u_callers)
                                     and all(ARM_SIDE.search(c) for c in u_callers)
                                     and not any(ARM_SIDE.search(c) for c in s_callers))

    # W4 (a) source
    numstat = subprocess.check_output(["git", "diff", "--numstat", a.base, "HEAD", "--", "src"], text=True).split("\n")
    changed = [l.split("\t") for l in numstat if l.strip()]
    report["W4a_source_diff"] = [dict(file=f, added=int(x), deleted=int(y)) for x, y, f in changed]
    gates["W4a_src_insertions_only_in_bwt"] = all(f == "src/bwt.rs" and y == "0" for x, y, f in changed)

    # W4 (b) binary: what each arm's worker thread reaches (call-graph closure from its
    # thread body), the stage functions compared; how the per-block driver is laid out
    by_base = {base(k): v for k, v in funcs.items()}

    def reach(root):
        seen, stack = set(), [root]
        while stack:
            n = stack.pop()
            if n in seen or n not in by_base:
                continue
            seen.add(n)
            stack.extend(callees(by_base[n]))
        return seen

    def root(kernel):
        want = f"std::sys::backtrace::__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::{kernel}>::{{closure#0}}, ()>"
        assert want in by_base, f"the {kernel} worker's thread body not found"
        return want

    # the stages that must be the same functions; the walk (room_segments, its unrolled
    # twin, and the segments' assembly, which `walk_segments_room` and its twin both end
    # with) is the change under test, so it is listed but not compared
    stage = re.compile(r"mibz2::(mtf::|bwt::build\b|rle::|crc::)")
    walkfns = re.compile(r"SegScratch>::assemble|room_segments(_unrolled)?::<")
    driver = re.compile(r"mibz2::decode_block::<|InverseBwt>::invert$|SegScratch>::assemble")
    norm = lambda n: n.replace("SegmentedUnrolled", "Segmented").replace("room_segments_unrolled", "room_segments")  # noqa: E731
    ra, rb = reach(root("Segmented")), reach(root("SegmentedUnrolled"))
    sa = sorted(n for n in ra if stage.search(n))
    sb = sorted(n for n in rb if stage.search(n))
    stage_diff = sorted({norm(n) for n in sa} ^ {norm(n) for n in sb})
    da = sorted(n for n in ra if driver.search(n))
    db = sorted(n for n in rb if driver.search(n))
    layout = sorted({norm(n) for n in da} ^ {norm(n) for n in db})
    wa = len(by_base[root("Segmented")])
    wb = len(by_base[root("SegmentedUnrolled")])
    report["W4b"] = dict(
        stage_functions=dict(A=sa, B=sb), stage_functions_differ=stage_diff,
        walk_functions=dict(A=sorted(n for n in ra if walkfns.search(n)), B=sorted(n for n in rb if walkfns.search(n))),
        driver_out_of_line=dict(A=da, B=db),
        worker_body_instructions=dict(A=wa, B=wb),
        inlining_differences=layout)
    gates["W4b_same_stage_functions"] = not stage_diff
    # W4 (c): a difference in how the per-block driver is inlined is reported for review;
    # the timed run requires it to be recorded as accepted (frozen.json "w4_review")
    report["W4c_review_required"] = dict(required=bool(layout), signature=dict(inlining_differences=layout))

    ok = all(gates.values())
    out = dict(gates=gates, pass_=ok, report=report)
    text = json.dumps(out, indent=1)
    if a.out:
        Path(a.out).write_text(text + "\n")
    print(json.dumps(dict(gates=gates, pass_=ok, w3=report["W3"], w4a=report["W4a_source_diff"],
                          w4b=report["W4b"], w4c=report["W4c_review_required"]), indent=1))
    sys.exit(0 if ok else 1)


if __name__ == "__main__":
    main()
