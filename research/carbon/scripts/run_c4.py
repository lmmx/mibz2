#!/usr/bin/env python3
"""Compile and run C-4 (De Bruijn Unitig-Interval Duality) across the 6 admitted BWT blocks."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
C_SRC = ROOT / "research/carbon/src/c4_debruijn_unitigs.c"
OUT_JSON = ROOT / "research/carbon/results/c4_results.json"


def stage_admitted_blocks(tmpdir: Path) -> list[dict]:
    manifest = json.loads(
        subprocess.check_output(
            ["git", "show", "origin/data/admitted-bwt:admitted-bwt/manifest.json"],
            cwd=ROOT,
        )
    )
    staged = []
    for entry in manifest["blocks"]:
        raw = subprocess.check_output(
            ["git", "show", f"origin/data/admitted-bwt:admitted-bwt/{entry['file']}"],
            cwd=ROOT,
        )
        assert len(raw) == entry["n"]
        assert hashlib.sha256(raw).hexdigest() == entry["sha256"]
        fpath = tmpdir / entry["file"]
        fpath.write_bytes(raw)
        p = entry["primary_index"]
        m16 = 0
        i = 0
        n = len(raw)
        while i < n:
            c = raw[i]
            j = i + 1
            while j < n and raw[j] == c:
                j += 1
            w = j - i
            if w <= 16:
                m16 += w
            elif i <= p < j:
                m16 += 1
            i = j
        staged.append({**entry, "path": str(fpath), "m16": m16, "t2": m16 / 2.0})
    return staged


def main() -> None:
    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory() as td:
        tdir = Path(td)
        bin_path = tdir / "c4_debruijn_unitigs"
        subprocess.check_call(
            ["gcc", "-O3", "-Wall", "-Wextra", "-Werror", str(C_SRC), "-o", str(bin_path)]
        )
        selftest = subprocess.check_output([str(bin_path), "--self-test"], text=True).strip()
        assert selftest == "SELF_TEST_OK", selftest

        blocks = stage_admitted_blocks(tdir)
        configs = [
            ("u8_L16_cursor2sw", 16, 8, 2),
            ("u8_L16_exact", 16, 8, 3),
            ("u8_L1_cursor1sw", 1, 8, 1),
            ("u8_L1_cursor2sw", 1, 8, 2),
            ("u8_L1_exact", 1, 8, 3),
            ("u6_L1_exact", 1, 6, 3),
        ]

        all_rows = []
        for cfg_name, lim, km, md in configs:
            print(f"\n=== {cfg_name} (L={lim}, k={km}, mode={md}) ===")
            for blk in blocks:
                out = subprocess.check_output(
                    [str(bin_path), blk["path"], str(blk["primary_index"]), str(lim), str(km), str(md)],
                    text=True,
                ).strip()
                rec = json.loads(out)
                assert rec["exact"] is True
                rec["block"] = blk["file"]
                rec["config"] = cfg_name
                rec["m16"] = blk["m16"]
                rec["t2"] = blk["t2"]
                rec["u_over_h1"] = rec["exact_unitigs"] / rec["h1"]
                rec["q_run_over_h1"] = rec["q_run"] / rec["h1"]
                rec["q_run_over_t2"] = rec["q_run"] / blk["t2"]
                all_rows.append(rec)
                print(
                    f"  {blk['file']:12s} h1={rec['h1']:6d} | "
                    f"v1_w1={rec['v1_w1']/rec['h1']:.3f} U_k={rec['exact_unitigs']:5d} ({rec['u_over_h1']:.3f}*h1) | "
                    f"Q_acq_u={rec['q_acq_unitig']:6d} | "
                    f"Q_run={rec['q_run']:6d} [unitig:{rec['q_run_unitig']:5d} + branch:{rec['q_run_branch']:5d}] "
                    f"({rec['q_run_over_h1']:.3f}*h1, {rec['q_run_over_t2']:.3f}*T2)"
                )

        OUT_JSON.write_text(json.dumps(all_rows, indent=2) + "\n")
        print(f"\nSaved {len(all_rows)} verified runs to {OUT_JSON}")


if __name__ == "__main__":
    main()
