"""P87's machine-code audit (docs/journal/2026-11-01-p87-four-core-protocol.md §5): before
any timing, of the exact binaries the run times. Exits 1 if any registered check fails; the
report lists what it found.

- E1, one compiled path for both arms. The trial's decoder runs its blocks in the
  worker's thread body (`with_inverse<Segmented>`'s closure, `decode_block` inlined into
  it). That body must hold exactly one call to `mibz2::p87::arm` and exactly two call sites
  of each kernel (the shipped path's, which neither arm takes, and the diagnostic path's,
  which both take), and no out-of-line `mibz2::p87::decode_block` other than at most one.
  A third call site of a kernel would mean the compiler made a copy per arm: E1 fails.
- E2, the shipped kernels. Each kernel (and the Huffman reader) in the P87 binary must be
  present once and disassemble the same, addresses written out, as in the default-feature
  build of examples/attribution_decode at the same commit.
- E3 (the attribution diagnostic's C6), the production default unchanged: the default-
  feature examples/attribution_decode at this commit against the base commit's
  (bench/attribution_same_code.py, run by the runner; its report is read here).

    p87_code_audit.py P87_BINARY DEFAULT_BINARY --same-code SAME_CODE_JSON --out report.json
"""
import argparse
import json
import re
import subprocess
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from attribution_same_code import functions  # noqa: E402

FUNC = re.compile(r"^[0-9a-f]+ <(.+)>:$")
CALL = re.compile(r"\bcall\s+[0-9a-f]+ <(.+?)(\+0x[0-9a-f]+)?>\s*$")
KERNELS = [
    "mibz2::bwt::build",
    "mibz2::bwt::room_segments::<32, 1>",
    "<mibz2::bwt::SegScratch>::assemble",
    "mibz2::mtf::mtf_avx2",
    "mibz2::rle::unrle_avx2",
]
# E2's: the kernels, their scalar twins, the CRC and the Huffman reader (the scanner's)
SHARED = KERNELS + [
    "mibz2::mtf::mtf_scalar", "mibz2::rle::unrle_scalar", "mibz2::crc::crc32", "mibz2::crc::clmul_update",
    "mibz2::block::read_item", "<mibz2::block::Huff>::new", "<mibz2::block::Huff>::decode_slow",
]
WORKER = re.compile(r"__rust_begin_short_backtrace::<<mibz2::parallel::StreamDecoder>::with_inverse<mibz2::bwt::Segmented>::\{closure#0\}")


def disassembly(binary):
    return subprocess.check_output(["objdump", "-d", "-C", "-Mintel", "--no-show-raw-insn", "-w", binary], text=True)


def occurrences(text):
    """every function's name, in order, with its calls (a name can occur more than once:
    distinct monomorphisations or copies)"""
    out, cur = [], None
    for line in text.splitlines():
        m = FUNC.match(line)
        if m:
            cur = dict(name=m.group(1), calls=[], instructions=0)
            out.append(cur)
            continue
        if cur is not None and line.startswith(" "):
            cur["instructions"] += 1
            c = CALL.search(line.split("#")[0])
            if c:
                cur["calls"].append(c.group(1))
    return out


def audit(p87_text, p87_funcs, default_funcs, same_code):
    occ = occurrences(p87_text)
    workers = [o for o in occ if WORKER.search(o["name"])]
    e1 = dict(worker_bodies=len(workers))
    ok1 = len(workers) == 1
    if workers:
        w = workers[0]
        e1["worker_instructions"] = w["instructions"]
        e1["arm_calls"] = w["calls"].count("mibz2::p87::arm")
        e1["kernel_call_sites"] = {k: w["calls"].count(k) for k in KERNELS}
        ok1 &= e1["arm_calls"] == 1 and all(n == 2 for n in e1["kernel_call_sites"].values())
    outline = [o for o in occ if o["name"].startswith("mibz2::p87::decode_block")]
    e1["out_of_line_p87_decode_block"] = len(outline)
    ok1 &= len(outline) <= 1
    e1["ok"] = ok1
    e2 = {}
    for k in SHARED:
        n = sum(1 for o in occ if o["name"] == k)
        same = k in p87_funcs and k in default_funcs and p87_funcs[k] == default_funcs[k]
        e2[k] = dict(copies=n, same_as_default_build=same, instructions=len(p87_funcs.get(k, [])))
    ok2 = all(v["copies"] == 1 and v["same_as_default_build"] for v in e2.values())
    e3 = dict(same=bool(same_code.get("same")), functions=same_code.get("functions"),
              differ=same_code.get("differ"), only_in_base=same_code.get("only_in_base"),
              only_in_head=same_code.get("only_in_head"))
    ok = ok1 and ok2 and e3["same"]
    return dict(ok=ok, E1_one_compiled_path=e1, E2_shipped_kernels=dict(ok=ok2, functions=e2),
                E3_production_default=dict(ok=e3["same"], **e3))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("p87")
    ap.add_argument("default")
    ap.add_argument("--same-code", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    rep = audit(disassembly(a.p87), functions(a.p87), functions(a.default), json.loads(Path(a.same_code).read_text()))
    Path(a.out).write_text(json.dumps(rep, indent=1) + "\n")
    e1 = rep["E1_one_compiled_path"]
    print(f"p87 audit: {'pass' if rep['ok'] else 'FAIL'}: E1 {'ok' if e1['ok'] else 'FAIL'} "
          f"(arm calls {e1.get('arm_calls')}, kernel call sites {e1.get('kernel_call_sites')}); "
          f"E2 {'ok' if rep['E2_shipped_kernels']['ok'] else 'FAIL'}; E3 {'ok' if rep['E3_production_default']['ok'] else 'FAIL'}")
    sys.exit(0 if rep["ok"] else 1)


if __name__ == "__main__":
    main()
