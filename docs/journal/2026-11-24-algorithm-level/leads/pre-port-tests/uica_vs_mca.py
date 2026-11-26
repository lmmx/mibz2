#!/usr/bin/env python3
"""Pre-port test 2 (Deep Dive #2): Stock uiCA (-arch CFL) vs llvm-mca (-mcpu=skylake)
on the four hot loops of mibz2, extracted directly from release `target-cpu=skylake`
object files compiled from master and PR #870.

Licence hygiene: uiCA is AGPL-3.0 and is invoked strictly as an external CLI tool
(path supplied via --uica or $UICA_PY, default /tmp/uica-tool/uiCA.py). No uiCA code
is imported or included here.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path


OUT_DIR = Path(__file__).resolve().parent


def find_mangled(obj: Path, exact_demangled: str) -> str:
    dem = subprocess.check_output(["nm", "-C", str(obj)], text=True).splitlines()
    raw = subprocess.check_output(["nm", str(obj)], text=True).splitlines()
    for dline, rline in zip(dem, raw):
        parts = dline.split(None, 2)
        if len(parts) == 3 and parts[2] == exact_demangled and parts[1] in ("T", "t"):
            return rline.split()[-1]
    raise RuntimeError(f"Symbol {exact_demangled!r} not found in {obj}")


def disassemble_fn(obj: Path, exact_demangled: str) -> list[tuple[int, str, str]]:
    """Return [(addr, hex_bytes, att_insn), ...] for exact_demangled in obj."""
    mangled = find_mangled(obj, exact_demangled)
    out = subprocess.check_output(
        ["objdump", "-d", "--no-show-raw-insn=no" if False else "--wide", "--disassemble=" + mangled, str(obj)],
        text=True,
    )
    insns: list[tuple[int, str, str]] = []
    # Match lines like:   1bb0:\t48 89 cd             \tmov    %rcx,%rbp
    pat = re.compile(r"^\s*([0-9a-f]+):\s+((?:[0-9a-f]{2}\s+)+)\s*(.*)$")
    for line in out.splitlines():
        m = pat.match(line)
        if not m:
            continue
        addr = int(m.group(1), 16)
        raw_hex = m.group(2).strip()
        asm = m.group(3).strip()
        if not asm:
            # continuation bytes of previous instruction
            if insns:
                pa, ph, pasm = insns[-1]
                insns[-1] = (pa, ph + " " + raw_hex, pasm)
            continue
        insns.append((addr, raw_hex, asm))
    return insns


def extract_trace(
    insns: list[tuple[int, str, str]],
    segments: list[tuple[int, int]],
    drop_taken_jumps: set[int] | None = None,
) -> list[tuple[int, str]]:
    """Extract ordered AT&T assembly instructions along the fast-path basic-block trace.

    Non-taken conditional branches are preserved (rewritten to jump to `.Lcold` after the
    loop) so both uiCA and llvm-mca model x86 macro-fusion (`cmp/test` + `jcc`) identically.
    Unconditional glue jumps between non-contiguous basic blocks and the loop backedge
    listed in `drop_taken_jumps` are omitted (both tools implicitly loop the body; we
    retain the loop-closing conditional branch if the loop closes with `jne`).
    """
    drop_taken_jumps = drop_taken_jumps or set()
    by_addr = {a: asm for a, _, asm in insns}
    ordered_addrs = [a for a, _, _ in insns]

    trace: list[tuple[int, str]] = []
    branch_re = re.compile(r"^(j[a-z]+)\s+[0-9a-f]+\s+<.*>$")
    for lo, hi in segments:
        for a in ordered_addrs:
            if not (lo <= a <= hi):
                continue
            if a in drop_taken_jumps:
                continue
            asm = by_addr[a]
            # Strip objdump trailing '# ...' comments
            asm = asm.split("#", 1)[0].strip()
            # Rewrite branch targets
            bm = branch_re.match(asm)
            if bm:
                mnem = bm.group(1)
                if mnem == "jmp":
                    # Unconditional jump in a straight-line trace is a taken glue/backedge
                    continue
                # Conditional jump on the fast path -> rewrite to cold exit label
                asm = f"{mnem} .Lcold"
            # Replace RIP-relative or symbol-annotated memory operands if any
            if "(%rip)" in asm:
                asm = re.sub(r"0x[0-9a-f]+\(%rip\)", "0(%r11)", asm)
            trace.append((a, asm))
    return trace


def assemble_and_measure(
    name: str,
    trace: list[tuple[int, str]],
    uica_py: Path,
    workdir: Path,
) -> dict:
    """Assemble exact trace with GNU as, then run stock uiCA (-arch CFL) and llvm-mca (-mcpu=skylake)."""
    s_mca = workdir / f"{name}_mca.s"
    s_obj = workdir / f"{name}_uica.s"
    o_file = workdir / f"{name}.o"
    bin_file = workdir / f"{name}.bin"

    # Build AT&T assembly for GNU as (with .Lloop and .Lcold labels)
    # Note: for uiCA -raw, we assemble the loop body WITHOUT .Lcold instructions,
    # placing .Lcold right at the end of the body so every non-taken jcc encodes cleanly.
    asm_lines = [insn for _, insn in trace]

    # For GNU as -> raw binary (uiCA -raw)
    obj_text = (
        ".text\n"
        ".globl loop_fn\n"
        ".p2align 6\n"
        "loop_fn:\n"
        + "\n".join(f"  {l}" for l in asm_lines)
        + "\n.Lcold:\n"
    )
    s_obj.write_text(obj_text)
    subprocess.check_call(["as", "--64", "-o", str(o_file), str(s_obj)])
    subprocess.check_call(["objcopy", "-O", "binary", "-j", ".text", str(o_file), str(bin_file)])

    # For llvm-mca: wrap ONLY the hot body with LLVM-MCA-BEGIN/END (exclude .Lcold target label from body)
    mca_text = (
        ".text\n"
        "# LLVM-MCA-BEGIN " + name + "\n"
        + "\n".join(f"  {l}" for l in asm_lines)
        + "\n# LLVM-MCA-END\n"
        ".Lcold:\n"
        "  ret\n"
    )
    s_mca.write_text(mca_text)

    # Run llvm-mca -mcpu=skylake
    mca_out = subprocess.check_output(
        ["llvm-mca", "-mcpu=skylake", "-iterations=500", "-bottleneck-analysis", str(s_mca)],
        text=True,
    )
    # Parse llvm-mca summary
    def grab(pat: str, text: str) -> float:
        m = re.search(pat, text)
        return float(m.group(1)) if m else float("nan")

    mca_iters = grab(r"Iterations:\s+([0-9.]+)", mca_out)
    mca_insns = grab(r"Instructions:\s+([0-9.]+)", mca_out)
    mca_total_cyc = grab(r"Total Cycles:\s+([0-9.]+)", mca_out)
    mca_total_uops = grab(r"Total uOps:\s+([0-9.]+)", mca_out)
    mca_rthroughput = grab(r"Block RThroughput:\s+([0-9.]+)", mca_out)
    mca_cyc_per_iter = mca_total_cyc / mca_iters if mca_iters else float("nan")
    mca_uops_per_iter = mca_total_uops / mca_iters if mca_iters else float("nan")

    # Run stock uiCA.py -arch CFL -raw
    uica_out = subprocess.check_output(
        [sys.executable, str(uica_py), "-arch", "CFL", "-raw", str(bin_file)],
        text=True,
    )
    uica_tp = grab(r"Throughput \(in cycles per iteration\):\s+([0-9.]+)", uica_out)
    m_bn = re.search(r"Bottleneck:\s*(.+)", uica_out)
    uica_bn = m_bn.group(1).strip() if m_bn else ""

    # Also test uiCA across all 64B cache-line alignment offsets ("-alignmentOffset all" or 0,16,32,48)
    uica_aligns = []
    for off in (0, 16, 32, 48):
        out_o = subprocess.check_output(
            [sys.executable, str(uica_py), "-arch", "CFL", "-raw", "-TPonly", "-alignmentOffset", str(off), str(bin_file)],
            text=True,
        ).strip()
        try:
            uica_aligns.append(float(out_o))
        except ValueError:
            pass

    return {
        "name": name,
        "num_insns": len(asm_lines),
        "bytes": bin_file.stat().st_size,
        "uica_cfl_cyc": uica_tp,
        "uica_cfl_align_range": [min(uica_aligns), max(uica_aligns)] if uica_aligns else [],
        "uica_bottleneck": uica_bn,
        "mca_skylake_cyc": round(mca_cyc_per_iter, 4),
        "mca_rthroughput": mca_rthroughput,
        "mca_uops_per_iter": round(mca_uops_per_iter, 2),
        "uica_vs_mca_pct": round(100.0 * (uica_tp - mca_cyc_per_iter) / mca_cyc_per_iter, 2),
        "asm_file": str(s_mca),
        "uica_full": uica_out,
        "mca_full": mca_out,
    }


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--master-obj", default="/tmp/mca_build/target_master/release/deps/mibz2.o")
    ap.add_argument("--pr870-obj", default="/tmp/mca_build/target_pr870/release/deps/mibz2.o")
    ap.add_argument("--uica", default=os.environ.get("UICA_PY", "/tmp/uica-tool/uiCA.py"))
    args = ap.parse_args()

    master_obj = Path(args.master_obj)
    pr870_obj = Path(args.pr870_obj)
    uica_py = Path(args.uica)

    if not uica_py.exists():
        raise SystemExit(f"uiCA.py not found at {uica_py}")

    insns_huff_master = disassemble_fn(master_obj, "mibz2::block::read_item_with_scratch")
    insns_huff_pr870 = disassemble_fn(pr870_obj, "mibz2::block::read_item_with_scratch")
    insns_a6b = disassemble_fn(
        master_obj,
        "<mibz2::open::Kernel>::induced_with::<mibz2::open::endpoint::Narrow>",
    )
    insns_runs = disassemble_fn(master_obj, "mibz2::runs::front")

    # Exact fast-path loop segments verified directly from objdump -d -l output:
    loops = [
        # 1a. Master src/huff.rs decode_group_win: full 5-lookup batch + refill (0x1bb0..0x1d69)
        (
            "huff_master_batch5",
            insns_huff_master,
            [(0x1BB0, 0x1D69)],
            set(),
            5.0,  # lookups per iteration
        ),
        # 1b. Master src/huff.rs single lookup step_win!() (step 5: 0x1cdc..0x1d25)
        (
            "huff_master_step1",
            insns_huff_master,
            [(0x1CDC, 0x1D25)],
            set(),
            1.0,
        ),
        # 2a. PR #870 src/huff.rs decode_group_win: full 5-lookup batch + refill (0x1827..0x19c1)
        (
            "huff_pr870_batch5",
            insns_huff_pr870,
            [(0x1827, 0x19C1)],
            set(),
            5.0,
        ),
        # 2b. PR #870 src/huff.rs single lookup step_win!(false, acc) (step 3: 0x18a0..0x18df)
        (
            "huff_pr870_step1",
            insns_huff_pr870,
            [(0x18A0, 0x18DF)],
            set(),
            1.0,
        ),
        # 3a. Master src/open/endpoint.rs A6b Narrow step: rule-push path (aw < bw, len > 15)
        #     0xac0..0xb54 -> (jne 0xbe0 taken) 0xbe0..0xc66 -> (ja 0xdec taken)
        #     0xdec..0xdf3 -> (jmp 0xf98) 0xf98..0xff8 -> (jbe 0xcb2 taken) 0xcb2..0xd27 (jne 0xac0)
        (
            "a6b_narrow_rule",
            insns_a6b,
            [(0xAC0, 0xB54), (0xBE0, 0xC66), (0xDEC, 0xDF3), (0xF98, 0xFF8), (0xCB2, 0xD27)],
            {0xB4F, 0xC61, 0xDF3, 0xFF3, 0xD22},
            1.0,  # 1 induction step per iter
        ),
        # 3b. Master src/open/endpoint.rs A6b Narrow step: zero-word fast path (aw < bw, w(x)==0)
        #     0xac0..0xb54 -> (jne 0xbe0 taken) 0xbe0..0xc16 -> (je 0xca6 taken) 0xca6..0xd27 (jne 0xac0)
        (
            "a6b_narrow_zero",
            insns_a6b,
            [(0xAC0, 0xB54), (0xBE0, 0xC16), (0xCA6, 0xD27)],
            {0xB4F, 0xC11, 0xD22},
            1.0,
        ),
        # 4a. Master src/runs.rs front_avx2 Pass 1: 32-symbol AVX2 bitmap loop (no EOB)
        #     0xf0..0x157 -> (je 0x180 taken) 0x180..0x1b7 (jne 0xf0)
        (
            "runs_avx2_pass1_32sym",
            insns_runs,
            [(0xF0, 0x157), (0x180, 0x1B7)],
            {0x156, 0x1B2},
            32.0,  # 32 symbols per iter
        ),
        # 4b. Master src/runs.rs front_avx2 Pass 2: literal MTF/RLE2 fast loop (s<33, run<=127, same word)
        #     0xb80..0xbc4 -> (jne 0xdaa taken) 0xdaa..0xddf (jmp 0xb80)
        (
            "runs_avx2_pass2_lit",
            insns_runs,
            [(0xB80, 0xBC4), (0xDAA, 0xDDF)],
            {0xBBF, 0xDDB},
            1.0,  # 1 literal/run per iter
        ),
    ]

    workdir = Path("/tmp/mca_build/extracted_loops")
    workdir.mkdir(parents=True, exist_ok=True)

    summary = []
    print(
        f"{'loop':<23} {'insns':>5} {'B':>4} | "
        f"{'uiCA(CFL)':>10} {'align':>13} | {'mca(SKL)':>9} {'RThr':>6} | "
        f"{'per-unit uiCA':>13} {'per-unit mca':>12} {'diff%':>6}"
    )
    for name, fn_insns, segs, drops, units in loops:
        trace = extract_trace(fn_insns, segs, drops)
        res = assemble_and_measure(name, trace, uica_py, workdir)
        res["units_per_iter"] = units
        res["uica_per_unit"] = round(res["uica_cfl_cyc"] / units, 3)
        res["mca_per_unit"] = round(res["mca_skylake_cyc"] / units, 3)
        al = res["uica_cfl_align_range"]
        al_str = f"{al[0]:.2f}..{al[1]:.2f}" if al else "-"
        print(
            f"{name:<23} {res['num_insns']:5d} {res['bytes']:4d} | "
            f"{res['uica_cfl_cyc']:10.2f} {al_str:>13} | {res['mca_skylake_cyc']:9.2f} {res['mca_rthroughput']:6.2f} | "
            f"{res['uica_per_unit']:13.3f} {res['mca_per_unit']:12.3f} {res['uica_vs_mca_pct']:+5.1f}%"
        )
        # Keep summary clean (omit huge raw outputs in JSON, save them to /tmp)
        (workdir / f"{name}.uica.txt").write_text(res.pop("uica_full"))
        (workdir / f"{name}.mca.txt").write_text(res.pop("mca_full"))
        summary.append(res)

    out_json = OUT_DIR / "uica_vs_mca_results.json"
    out_json.write_text(json.dumps(summary, indent=2))
    print(f"\nWrote {out_json}")


if __name__ == "__main__":
    main()
