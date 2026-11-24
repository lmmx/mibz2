#!/usr/bin/env python3
"""Measure F baseline (232 census blocks + 6 admitted blocks) and NEEDS M-2.

Computes:
1. Cost-weighted applicability and h1/m distribution on the 232 distinct two-ended
   Wikidata blocks from docs/journal/2026-11-24-wikidata-block-census/{census,features}.tsv.
2. Exact h1, h1R (sigma_hex: [0-9a-fA-F]), h1N = h1 - h1R, content-class decomposition,
   and orbit N<->R boundary counts (Pglue cost proxy) on the 6 admitted blocks from
   data/admitted-bwt.
"""

from __future__ import annotations

import csv
import json
import os
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path

HEX_BYTES = set(b"0123456789abcdefABCDEF")
GUID_RE = re.compile(
    rb"[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-4[0-9a-fA-F]{3}-[89abAB][0-9a-fA-F]{3}-[0-9a-fA-F]{12}"
)
SHA1_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9a-fA-F]{40}(?![0-9a-fA-F])")
ENTITY_RE = re.compile(rb'"[QPL][0-9]+"')
JSON_KEY_RE = re.compile(rb'"[^"\\]{1,40}":')
NUM_RE = re.compile(rb"(?<![0-9a-fA-F])[0-9]{4,}(?![0-9a-fA-F])")


def load_admitted_blocks() -> list[tuple[str, int, bytes]]:
    env_dir = os.environ.get("ADMITTED_BWT_DIR")
    if env_dir and (Path(env_dir) / "manifest.json").exists():
        base = Path(env_dir)
        manifest = json.loads((base / "manifest.json").read_text())
        out = []
        for entry in manifest["blocks"]:
            data = (base / entry["file"]).read_bytes()
            assert len(data) == entry["n"]
            out.append((entry["file"], entry["primary_index"], data))
        return out

    # Fallback: read directly from git ref origin/data/admitted-bwt
    manifest_raw = subprocess.check_output(
        ["git", "show", "origin/data/admitted-bwt:admitted-bwt/manifest.json"]
    )
    manifest = json.loads(manifest_raw)
    out = []
    for entry in manifest["blocks"]:
        data = subprocess.check_output(
            ["git", "show", f"origin/data/admitted-bwt:admitted-bwt/{entry['file']}"]
        )
        assert len(data) == entry["n"]
        out.append((entry["file"], entry["primary_index"], data))
    return out


def compute_lf(L: bytes) -> list[int]:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for b in range(256):
        C[b] = acc
        acc += counts[b]
    seen = [0] * 256
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = C[b] + seen[b]
        seen[b] += 1
    return lf


def invert_with_row_map(L: bytes, p: int) -> tuple[bytes, list[int]]:
    """Return (T, row_of_pos) where T[pos] = L[row_of_pos[pos]]."""
    lf = compute_lf(L)
    n = len(L)
    psi = [0] * n
    for i, j in enumerate(lf):
        psi[j] = i
    T = bytearray(n)
    row_of_pos = [0] * n
    x = p
    for j in range(n):
        x = psi[x]
        T[j] = L[x]
        row_of_pos[j] = x
    assert x == p, "Expected single-cycle column"
    return bytes(T), row_of_pos


def analyze_census() -> dict[str, dict[str, float]]:
    census_path = Path("docs/journal/2026-11-24-wikidata-block-census/census.tsv")
    feat_path = Path("docs/journal/2026-11-24-wikidata-block-census/features.tsv")

    census_by_seq = {}
    with census_path.open() as f:
        for row in csv.DictReader(f, delimiter="\t"):
            census_by_seq[int(row["seq"])] = row

    # Join with features.tsv and aggregate over the 4 copies per distinct block
    by_id: dict[str, list[tuple[dict[str, str], dict[str, str]]]] = {}
    with feat_path.open() as f:
        for frow in csv.DictReader(f, delimiter="\t"):
            seq = int(frow["seq"])
            crow = census_by_seq[seq]
            by_id.setdefault(crow["id"], []).append((crow, frow))

    blocks = []
    for bid, pairs in by_id.items():
        crow = pairs[0][0]
        m = int(crow["m"])
        n = int(crow["n"])
        h1 = int(crow["runs_1"])
        runs = int(crow["runs"])
        rows_2_3 = int(crow["rows_2_3"])
        rows_4_16 = int(crow["rows_4_16"])
        strips = int(crow["strips"])
        emit_ns_row = sum(float(fr["emit_ns_per_row"]) for _, fr in pairs) / len(pairs)
        cons_ns_row = sum(float(fr["construct_ns_per_row"]) for _, fr in pairs) / len(pairs)
        emit_ns = emit_ns_row * m
        cons_ns = cons_ns_row * m
        total_ns = emit_ns + cons_ns
        blocks.append(
            {
                "id": bid,
                "n": n,
                "p": int(crow["p"]),
                "m": m,
                "runs": runs,
                "h1": h1,
                "rows_2_3": rows_2_3,
                "rows_4_16": rows_4_16,
                "strips": strips,
                "emit_ns": emit_ns,
                "cons_ns": cons_ns,
                "total_ns": total_ns,
                "emit_ns_row": emit_ns_row,
                "cons_ns_row": cons_ns_row,
            }
        )

    tot_ns = sum(b["total_ns"] for b in blocks)
    tot_emit = sum(b["emit_ns"] for b in blocks)
    tot_cons = sum(b["cons_ns"] for b in blocks)
    tot_m = sum(b["m"] for b in blocks)
    tot_h1 = sum(b["h1"] for b in blocks)
    tot_runs = sum(b["runs"] for b in blocks)

    # Cost-weighted h1/m and emit share
    cw_h1_over_m = sum((b["total_ns"] / tot_ns) * (b["h1"] / b["m"]) for b in blocks)
    cw_emit_h1_share = sum((b["emit_ns"] / tot_ns) * (b["h1"] / b["m"]) for b in blocks)
    h1_over_m_vals = sorted(b["h1"] / b["m"] for b in blocks)

    print("=== 232 DISTINCT TWO-ENDED CENSUS BLOCKS (#805, 1-worker i9 costs) ===")
    print(f"Distinct blocks: {len(blocks)}")
    print(f"Total kernel time (232 blocks, 1W): {tot_ns/1e6:.2f} ms "
          f"(emit {tot_emit/1e6:.2f} ms = {100*tot_emit/tot_ns:.1f}%, "
          f"construct {tot_cons/1e6:.2f} ms = {100*tot_cons/tot_ns:.1f}%)")
    print(f"Total m = {tot_m:,}, total h1 = {tot_h1:,} ({100*tot_h1/tot_m:.2f}% of m), "
          f"total runs = {tot_runs:,} (h1/runs = {100*tot_h1/tot_runs:.2f}%)")
    print(f"h1/m across blocks: min={h1_over_m_vals[0]:.4f}, p10={h1_over_m_vals[len(blocks)//10]:.4f}, "
          f"median={h1_over_m_vals[len(blocks)//2]:.4f}, p90={h1_over_m_vals[9*len(blocks)//10]:.4f}, "
          f"max={h1_over_m_vals[-1]:.4f}")
    print(f"Cost-weighted mean(h1/m): {cw_h1_over_m:.4f}")
    print(f"Cost-weighted emit share attributable to h1 rows (proportional to row count): "
          f"{100*cw_emit_h1_share:.2f}% of total kernel time "
          f"({100*cw_emit_h1_share/(tot_emit/tot_ns):.2f}% of emit time)")

    # Stratify by density regime (main mode vs dense tail m/n >= 0.15)
    dense = [b for b in blocks if b["m"] / b["n"] >= 0.15]
    main_mode = [b for b in blocks if b["m"] / b["n"] < 0.15]
    print(
        f"Main mode (m/n < 0.15): {len(main_mode)} blocks, "
        f"{100*sum(b['total_ns'] for b in main_mode)/tot_ns:.1f}% of kernel time, "
        f"sum(h1)/sum(m) = {sum(b['h1'] for b in main_mode)/sum(b['m'] for b in main_mode):.4f}"
    )
    print(
        f"Dense tail (m/n >= 0.15): {len(dense)} blocks, "
        f"{100*sum(b['total_ns'] for b in dense)/tot_ns:.1f}% of kernel time, "
        f"sum(h1)/sum(m) = {sum(b['h1'] for b in dense)/sum(b['m'] for b in dense):.4f}"
    )

    return {(b["n"], b["p"], b["runs"]): b for b in blocks}


def analyze_admitted(census_lookup: dict) -> None:
    blocks = load_admitted_blocks()
    print("\n=== 6 ADMITTED BLOCKS (data/admitted-bwt): NEEDS M-2 & STREAM N CEILING ===")
    header = (
        f"{'block':<14} {'n':>7} {'r':>7} {'m16':>7} {'h1':>7} "
        f"{'h1R(hex)':>8} {'h1R/h1':>7} {'h1N':>7} {'h1N/h1':>7} "
        f"{'cost_ms':>8} {'Pglue_b':>8} {'b/h1N':>6}"
    )
    print(header)
    print("-" * len(header))

    class_rows = []
    for fname, p, L in blocks:
        n = len(L)
        # Run lengths
        is_w1 = bytearray(n)
        is_thin = bytearray(n)
        r = 0
        h1 = 0
        h1R = 0
        m16 = 0
        i = 0
        while i < n:
            j = i + 1
            b = L[i]
            while j < n and L[j] == b:
                j += 1
            w = j - i
            r += 1
            if w == 1:
                h1 += 1
                is_w1[i] = 1
                if b in HEX_BYTES:
                    h1R += 1
            if w <= 16:
                m16 += w
                for k in range(i, j):
                    is_thin[k] = 1
            elif i <= p < j:
                m16 += 1
                is_thin[p] = 1
            i = j
        h1N = h1 - h1R

        # Invert and classify text bytes
        T, row_of_pos = invert_with_row_map(L, p)

        # Orbit N<->R boundary switches under sigma_hex:
        # Two definitions of boundary:
        # (a) Among ALL rows: switches between "width-1 hex (R)" and "non-(width-1 hex)"
        # (b) Restricted to width-1 rows in orbit order: switches between h1R and h1N
        w1_stream_seq = []
        all_switches = 0
        prev_r = (is_w1[row_of_pos[0]] == 1) and (L[row_of_pos[0]] in HEX_BYTES)
        for pos in range(n):
            row = row_of_pos[pos]
            cur_r = (is_w1[row] == 1) and (L[row] in HEX_BYTES)
            if pos > 0 and cur_r != prev_r:
                all_switches += 1
            prev_r = cur_r
            if is_w1[row]:
                w1_stream_seq.append(1 if (L[row] in HEX_BYTES) else 0)
        # wrap-around switch
        first_r = (is_w1[row_of_pos[0]] == 1) and (L[row_of_pos[0]] in HEX_BYTES)
        if prev_r != first_r:
            all_switches += 1

        w1_switches = sum(
            1
            for k in range(len(w1_stream_seq))
            if w1_stream_seq[k] != w1_stream_seq[(k + 1) % len(w1_stream_seq)]
        )

        # Classify text positions
        tag = bytearray(n)  # 0=other, 1=GUID_rand, 2=GUID_fix, 3=SHA1, 4=Entity, 5=JSON_key, 6=Num
        for mobj in JSON_KEY_RE.finditer(T):
            for k in range(mobj.start(), mobj.end()):
                tag[k] = 5
        for mobj in NUM_RE.finditer(T):
            for k in range(mobj.start(), mobj.end()):
                if tag[k] == 0:
                    tag[k] = 6
        for mobj in ENTITY_RE.finditer(T):
            for k in range(mobj.start(), mobj.end()):
                tag[k] = 4
        for mobj in SHA1_RE.finditer(T):
            for k in range(mobj.start(), mobj.end()):
                tag[k] = 3
        for mobj in GUID_RE.finditer(T):
            s = mobj.start()
            for off in range(36):
                # Fixed positions in RFC 4122 v4 GUID: hyphens (8,13,18,23), version (14), variant (19)
                if off in (8, 13, 18, 23, 14, 19):
                    tag[s + off] = 2
                else:
                    tag[s + off] = 1

        c_w1 = Counter()
        c_w1_hex = Counter()
        c_w1_nonhex = Counter()
        for pos in range(n):
            row = row_of_pos[pos]
            if is_w1[row]:
                t = tag[pos]
                c_w1[t] += 1
                if L[row] in HEX_BYTES:
                    c_w1_hex[t] += 1
                else:
                    c_w1_nonhex[t] += 1

        cinfo = census_lookup.get((n, p, r))
        cost_ms = cinfo["total_ns"] / 1e6 if cinfo else float("nan")

        print(
            f"{fname:<14} {n:>7} {r:>7} {m16:>7} {h1:>7} "
            f"{h1R:>8} {h1R/h1:>7.4f} {h1N:>7} {h1N/h1:>7.4f} "
            f"{cost_ms:>8.3f} {w1_switches:>8} {w1_switches/h1N:>6.2f}"
        )
        class_rows.append((fname, h1, h1R, h1N, all_switches, w1_switches, c_w1, c_w1_hex, c_w1_nonhex))

    print("\n=== CONTENT CLASS BREAKDOWN OF WIDTH-1 RUNS (6 admitted blocks) ===")
    names = {1: "GUID_rand", 2: "GUID_fix", 3: "SHA1", 4: "Entity_QPL", 5: "JSON_key", 6: "Num4+", 0: "Other"}
    hdr = f"{'block':<12} " + " ".join(f"{names[k]:>10}" for k in [1, 2, 3, 4, 5, 6, 0]) + "  |  nonGUID/h1  all_sw/h1"
    print(hdr)
    print("-" * len(hdr))
    for fname, h1, h1R, h1N, all_sw, w1_sw, c_w1, c_hex, c_nh in class_rows:
        non_guid = h1 - c_w1[1] - c_w1[2]
        parts = " ".join(f"{100*c_w1[k]/h1:>9.1f}%" for k in [1, 2, 3, 4, 5, 6, 0])
        print(f"{fname:<12} {parts}  |  {100*non_guid/h1:>9.1f}%  {all_sw/h1:>9.3f}")

    print("\n=== INSIDE h1R (sigma_hex): GUID vs NON-GUID HEX WIDTH-1 RUNS ===")
    for fname, h1, h1R, h1N, all_sw, w1_sw, c_w1, c_hex, c_nh in class_rows:
        guid_in_hex = c_hex[1] + c_hex[2]
        sha1_in_hex = c_hex[3]
        ent_in_hex = c_hex[4]
        num_in_hex = c_hex[6]
        other_in_hex = h1R - guid_in_hex - sha1_in_hex - ent_in_hex - num_in_hex
        print(
            f"  {fname:<12}: h1R={h1R:>6} ({100*h1R/h1:.1f}% of h1) -> "
            f"GUID={100*guid_in_hex/h1R:.1f}%, SHA1={100*sha1_in_hex/h1R:.1f}%, "
            f"QPL={100*ent_in_hex/h1R:.1f}%, Num4+={100*num_in_hex/h1R:.1f}%, "
            f"OtherHex={100*other_in_hex/h1R:.1f}%"
        )


if __name__ == "__main__":
    lookup = analyze_census()
    analyze_admitted(lookup)
