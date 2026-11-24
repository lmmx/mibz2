#!/usr/bin/env python3
"""Compile and run C-1 (Bipartite Boundary-Return Chords) across the 6 admitted BWT blocks."""

from __future__ import annotations

import hashlib
import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
C_SRC = ROOT / "research/carbon/src/c1_boundary_chords.c"
OUT_JSON = ROOT / "research/carbon/results/c1_results.json"


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
        # Also compute m16 (Open LIMIT=16 retained count) for exact T^2 = m16 / 2 reference
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
        bin_path = tdir / "c1_boundary_chords"
        subprocess.check_call(
            ["gcc", "-O3", "-Wall", "-Wextra", "-Werror", str(C_SRC), "-o", str(bin_path)]
        )
        selftest = subprocess.check_output([str(bin_path), "--self-test"], text=True).strip()
        assert selftest == "SELF_TEST_OK", selftest

        blocks = stage_admitted_blocks(tdir)
        configs = [
            ("push_d2", 4, 0, 2),
            ("push_d4", 4, 0, 4),
            ("sweep_k1_L16", 16, 1, 1),
            ("sweep_k1_L4", 4, 1, 1),
            ("sweep_k1_L2", 2, 1, 1),
            ("sweep_k2_L4", 4, 1, 2),
            ("sweep_k2_L2", 2, 1, 2),
            ("sweep_k3_L2", 2, 1, 3),
            ("exact_L16", 16, 2, 0),
            ("exact_L4", 4, 2, 0),
            ("exact_L2", 2, 2, 0),
        ]

        all_rows = []
        for cfg_name, lim, mode, param in configs:
            print(f"\n=== {cfg_name} (L={lim}, mode={mode}, param={param}) ===")
            for blk in blocks:
                out = subprocess.check_output(
                    [
                        str(bin_path),
                        blk["path"],
                        str(blk["primary_index"]),
                        str(lim),
                        str(mode),
                        str(param),
                    ],
                    text=True,
                ).strip()
                rec = json.loads(out)
                assert rec["exact"] is True
                rec["block"] = blk["file"]
                rec["config"] = cfg_name
                rec["m16"] = blk["m16"]
                rec["t2"] = blk["t2"]
                rec["q_total"] = rec["q_acq_thick"] + rec["q_acq_thin"] + rec["q_run"]
                rec["q_run_over_h1"] = rec["q_run"] / rec["h1"]
                rec["q_run_over_t2"] = rec["q_run"] / blk["t2"]
                rec["q_tot_over_h1"] = rec["q_total"] / rec["h1"]
                rec["chi_over_h1"] = rec["chi_L"] / rec["h1"]
                all_rows.append(rec)
                print(
                    f"  {blk['file']:12s} h1={rec['h1']:6d} T2={blk['t2']:8.1f} | "
                    f"chi_L={rec['chi_L']:5d} ({rec['chi_over_h1']:.3f}*h1) | "
                    f"Q_run={rec['q_run']:6d} ({rec['q_run_over_h1']:.3f}*h1, {rec['q_run_over_t2']:.3f}*T2) | "
                    f"Q_acq=(thick:{rec['q_acq_thick']}, thin:{rec['q_acq_thin']}) "
                    f"Q_tot={rec['q_total']:6d} ({rec['q_tot_over_h1']:.3f}*h1) P={rec['P_passes']:.1f}"
                )

        OUT_JSON.write_text(json.dumps(all_rows, indent=2) + "\n")
        print(f"\nSaved {len(all_rows)} verified runs to {OUT_JSON}")


if __name__ == "__main__":
    main()
