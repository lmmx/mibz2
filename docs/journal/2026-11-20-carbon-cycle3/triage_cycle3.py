#!/usr/bin/env python3
"""Cycle 3 Triage Verifier & 6-Block Structural Census:
1. Triage 1 (OPEN-QUESTIONS.md §1 — Recursive / Second Open Cut on g from Open's bands):
   Measures exact band count B, maximal affine piece count P(g^-1) on [0, m), width distribution of
   those affine pieces (verifying No-Widening max_w <= LIMIT=16), and second-level retained row count
   m_2 vs construction piece count when eliminating all multi-row pieces (w >= 2) of g^-1.
2. Triage 2 (UUID v4 Shared-Format Conjecture — 2026-11-20-toy-transfer.md & working-backwards.md):
   Synthesises structured JSON blocks containing M = 1,000 v4 UUIDs ("xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx")
   versus unformatted 32-hex strings ("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx") with identical 122-bit random
   entropy per record, verifying that:
   - Fixed format bytes ('-' at 8,13,18,23 and '4' at 14) already coalesce into thick/medium runs
     eliminated by Open, whereas interior random hex nibbles remain in width-1 isolated runs;
   - Knowing the exact v4 UUID template + regex leaves all 122*M bits in random singleton permutations
     between suffix-sorted hex buckets.
3. Triage 3 (Where Repeated Thin Runs Live — LIMIT = 1, 2, 4, 8, 16 Exact Census):
   Measures m_L, H_L (eliminated strips), endpoint.rs steps, band count B_L, raw flattened spelling_L,
   and entry-byte-factored (tail-separated) spelling bytes across all 6 blocks.
"""

import hashlib
import json
import pathlib
import random
import sys

CYCLE1_DIR = pathlib.Path(__file__).resolve().parent.parent / "2026-11-20-carbon-word-transport"
sys.path.insert(0, str(CYCLE1_DIR))
import verify_cycle1 as c1  # noqa: E402

BLOCKS_DIR = pathlib.Path(
    sys.argv[1] if len(sys.argv) > 1 else "/tmp/admitted-bwt/admitted-bwt"
)


def bwt_from_text(t: bytes):
    n = len(t)
    # Double text or cyclic rotations via suffix sort with sentinel-free rotation comparison
    # For moderate n <= 60,000, cyclic key t[i:] + t[:i] is exact and fast.
    rots = sorted(range(n), key=lambda i: t[i:] + t[:i])
    p = rots.index(0)
    L = bytes(t[(i - 1) % n] for i in rots)
    return L, p


def triage_uuid_format():
    rng = random.Random(20261120)
    M = 600
    hex_chars = "0123456789abcdef"
    var_chars = "89ab"

    records_uuid = []
    records_raw32 = []
    for i in range(M):
        # Generate exact 122 random bits: 30 random hex nibbles + 2 random bits in variant
        nibs = [rng.choice(hex_chars) for _ in range(30)]
        v_nib = var_chars[rng.randrange(4)]
        # UUID v4: 8-4-4-4-12 = 36 chars (32 hex + 4 dashes)
        u = (
            "".join(nibs[0:8])
            + "-"
            + "".join(nibs[8:12])
            + "-4"
            + "".join(nibs[12:15])
            + "-"
            + v_nib
            + "".join(nibs[15:18])
            + "-"
            + "".join(nibs[18:30])
        )
        # Unformatted 31-nibble (122-bit) raw hex token inside identical JSON wrapper
        r_tok = "".join(nibs[0:15]) + v_nib + "".join(nibs[15:30])
        records_uuid.append(f'{{"id":"Q{i % 25}$' + u + '","type":"statement"},\n')
        records_raw32.append(f'{{"id":"Q{i % 25}$' + r_tok + '","type":"statement"},\n')

    t_uuid = "".join(records_uuid).encode("ascii")
    t_raw = "".join(records_raw32).encode("ascii")

    for label, txt in (("UUID-v4 (formatted)", t_uuid), ("Raw-122b (unformatted)", t_raw)):
        L, p = bwt_from_text(txt)
        recs = c1.to_capped_records(L)
        # Count maximal runs and width-1 runs
        runs = []
        for b in L:
            if runs and runs[-1][0] == b:
                runs[-1][1] += 1
            else:
                runs.append([b, 1])
        r_tot = len(runs)
        r_w1 = sum(1 for _, w in runs if w == 1)
        m, mark, next_j, codes, sp, offs, wcnt = c1.build_open(recs, len(L), p, limit=16)
        bits = m.bit_length() or 1
        edges = [(codes[z] << bits) | next_j[z] for z in range(m)]
        iso1 = sum(
            1
            for z in range(m)
            if not (
                (z > 0 and z != mark and (z - 1) != mark and edges[z] == edges[z - 1] + 1)
                or (z + 1 < m and z != mark and (z + 1) != mark and edges[z + 1] == edges[z] + 1)
            )
        )
        print(
            f"  {label:22s}: n={len(L):5d} r={r_tot:5d} w1_runs={r_w1:5d} "
            f"({r_w1/M:4.1f}/item) | Open<16> m={m:5d} ({m/M:4.1f}/item) iso1={iso1:5d} ({iso1/M:4.1f}/item)"
        )


def second_cut_and_limit_census():
    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    print("\n=== Triage 1: Second Open Cut on g^-1 (OPEN-QUESTIONS.md §1) ===")
    for b in manifest["blocks"]:
        fn = b["file"]
        L = (BLOCKS_DIR / fn).read_bytes()
        assert hashlib.sha256(L).hexdigest() == b["sha256"]
        n, p = b["n"], b["primary_index"]
        recs = c1.to_capped_records(L)
        m, mark, next_j, codes, sp, offs, wcnt = c1.build_open(recs, n, p, limit=16)

        # Count maximal affine pieces of g^-1 (with mark isolated, matching Open):
        # two adjacent retained rows z-1, z lie in the same piece iff code[z]==code[z-1] and next_j[z]==next_j[z-1]+1
        pieces = []
        z = 0
        while z < m:
            w = 1
            if z != mark:
                while (
                    z + w < m
                    and (z + w) != mark
                    and codes[z + w] == codes[z]
                    and next_j[z + w] == next_j[z] + w
                ):
                    w += 1
            pieces.append(w)
            z += w

        max_pw = max(pieces)
        p_tot = len(pieces)
        elim_pieces_w17 = sum(1 for w in pieces if w > 16)
        m2_w17 = sum(w for w in pieces if w <= 16)
        # Suppose we eliminate all pieces of width >= 2 (the maximal structural cut of g^-1!)
        elim_pieces_w2 = sum(1 for w in pieces if w >= 2)
        m2_w2 = sum(w for w in pieces if w == 1)
        # Or eliminate pieces of width >= 4
        elim_pieces_w4 = sum(1 for w in pieces if w >= 4)
        m2_w4 = sum(w for w in pieces if w < 4)
        print(
            f"  {fn:12s}: m={m:6d} | pieces={p_tot:6d} max_w={max_pw:3d} "
            f"| w>16: H2={elim_pieces_w17:4d} m2={m2_w17:6d} "
            f"| w>=4: H2={elim_pieces_w4:5d} m2={m2_w4:6d} "
            f"| w>=2: H2={elim_pieces_w2:5d} m2={m2_w2:6d}"
        )

    print("\n=== Triage 3: Open Retained/Spelling Scaling Across LIMIT in {16, 8, 4, 2, 1} ===")
    for fn_target in ("block-0.L", "block-143.L", "block-210.L"):
        b = next(x for x in manifest["blocks"] if x["file"] == fn_target)
        L = (BLOCKS_DIR / fn_target).read_bytes()
        n, p = b["n"], b["primary_index"]
        recs = c1.to_capped_records(L)
        print(f"  --- {fn_target} (n={n}, r={b['r']}) ---")
        for lim in (16, 8, 4, 2, 1):
            m, mark, next_j, codes, sp, offs, wcnt = c1.build_open(recs, n, p, limit=lim)
            # Count unique thick excursion prefixes (word without final thin entry byte!)
            uniq_prefixes = set()
            prefix_bytes = 0
            for w in range(wcnt):
                word = sp[offs[w] : offs[w + 1]]
                pref = word[:-1]
                if pref not in uniq_prefixes:
                    uniq_prefixes.add(pref)
                    prefix_bytes += len(pref)
            print(
                f"    LIMIT={lim:2d}: m={m:6d} ({100*m/n:4.1f}% n) | words={wcnt:5d} "
                f"| flat_spell={len(sp)/1024:6.1f} KiB | dedup_node_spell={prefix_bytes/1024:6.1f} KiB "
                f"({len(uniq_prefixes):5d} uniq)"
            )


if __name__ == "__main__":
    print("=== Triage 2: UUID v4 Shared-Format Conjecture Check ===")
    triage_uuid_format()
    second_cut_and_limit_census()
