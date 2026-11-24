#!/usr/bin/env python3
"""Compile and run C-3 (Polychromatic Quotient Slabs) across the 6 admitted BWT blocks."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
C_SRC = ROOT / "research/carbon/src/c3_quotient_slabs.c"
OUT_JSON = ROOT / "research/carbon/results/c3_results.json"


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
        bin_path = tdir / "c3_quotient_slabs"
        subprocess.check_call(
            ["gcc", "-O3", "-Wall", "-Wextra", "-Werror", str(C_SRC), "-o", str(bin_path)]
        )
        selftest = subprocess.check_output([str(bin_path), "--self-test"], text=True).strip()
        assert selftest == "SELF_TEST_OK", selftest

        blocks = stage_admitted_blocks(tdir)
        configs = [
            ("hex16_pure", 0, 0),
            ("hex16_thin16", 0, 1),
            ("hex23_pure", 1, 0),
            ("hex23_thin16", 1, 1),
        ]

        all_rows = []
        for cfg_name, sk, ft in configs:
            print(f"\n=== {cfg_name} (sub_kind={sk}, fuse_thick={ft}) ===")
            for blk in blocks:
                out = subprocess.check_output(
                    [str(bin_path), blk["path"], str(blk["primary_index"]), str(sk), str(ft)],
                    text=True,
                ).strip()
                rec = json.loads(out)
                assert rec["exact"] is True
                rec["block"] = blk["file"]
                rec["config"] = cfg_name
                rec["m16"] = blk["m16"]
                rec["t2"] = blk["t2"]
                rec["r_pi_over_h1"] = rec["r_pi"] / rec["h1"]
                rec["r_pi_over_t2"] = rec["r_pi"] / blk["t2"]
                rec["q_w1_over_h1"] = rec["q_w1"] / rec["h1"]
                all_rows.append(rec)
                print(
                    f"  {blk['file']:12s} h1={rec['h1']:6d} T2={blk['t2']:8.1f} | "
                    f"r_pi={rec['r_pi']:5d} ({rec['r_pi_over_h1']:.3f}*h1, {rec['r_pi_over_t2']:.3f}*T2) "
                    f"q_w1={rec['q_w1']:5d} ({rec['q_w1_over_h1']:.3f}*h1) | "
                    f"Q_acq={rec['q_acq']} P={rec['P_passes']:.2f} S={rec['S_bytes']/1e6:.2f}MB | "
                    f"Q_run_thin={rec['q_run_thin_steps']:6d} ({rec['q_run_thin_steps']/rec['h1']:.2f}*h1)"
                )

        OUT_JSON.write_text(json.dumps(all_rows, indent=2) + "\n")
        print(f"\nSaved {len(all_rows)} verified runs to {OUT_JSON}")


if __name__ == "__main__":
    main()
