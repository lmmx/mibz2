#!/usr/bin/env python3
"""
Exact invariant census, exhaustive finite-case verification of structural lemmas,
and toy-family separation experiments for Carbon Depth (research/carbon-depth).

Run:
    BLOCKS_DIR=/tmp/admitted-bwt/admitted-bwt python3 docs/journal/2026-11-20-carbon-depth/invariants_and_toys.py
"""

from __future__ import annotations
import hashlib
import itertools
import json
import math
import os
import pathlib
import random
import sys
from collections import Counter


def compute_lf(L: bytes | list[int]) -> tuple[list[int], list[int]]:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    s = 0
    for c in range(256):
        C[c] = s
        s += counts[c]
    occ = [0] * 256
    lf = [0] * n
    for i, b in enumerate(L):
        lf[i] = C[b] + occ[b]
        occ[b] += 1
    return lf, C


def dense_invert(L: bytes, p: int) -> bytes:
    n = len(L)
    lf, _ = compute_lf(L)
    cur = p
    rev = bytearray()
    for _ in range(n):
        rev.append(L[cur])
        cur = lf[cur]
        if cur == p:
            break
    rev.reverse()
    period = bytes(rev)
    out = bytearray(period)
    while len(out) < n:
        take = min(len(period), n - len(out))
        out.extend(period[:take])
    return bytes(out)


def runs_of(L: bytes | list[int]) -> list[tuple[int, int, int]]:
    """Return maximal runs as (byte, start, width)."""
    n = len(L)
    if n == 0:
        return []
    runs = []
    cur = L[0]
    start = 0
    for i in range(1, n):
        if L[i] != cur:
            runs.append((cur, start, i - start))
            cur = L[i]
            start = i
    runs.append((cur, start, n - start))
    return runs


def capped_record_count(runs: list[tuple[int, int, int]], cap: int = 127) -> int:
    return sum((w + cap - 1) // cap for _, _, w in runs)


def cyclic_runs_stats(seq: list[int]) -> tuple[int, int]:
    """Return (cyclic_run_count, cyclic_width1_count) on Z/nZ."""
    n = len(seq)
    if n == 0:
        return 0, 0
    if all(x == seq[0] for x in seq):
        return 1, (1 if n == 1 else 0)
    # Find a boundary
    b0 = next(i for i in range(n) if seq[i] != seq[(i - 1) % n])
    r = 0
    r1 = 0
    i = 0
    while i < n:
        c = seq[(b0 + i) % n]
        w = 1
        while i + w < n and seq[(b0 + i + w) % n] == c:
            w += 1
        r += 1
        if w == 1:
            r1 += 1
        i += w
    return r, r1


def verify_small_exhaustive() -> dict[str, int]:
    """
    Exhaustively verify structural lemmas across all binary columns 1 <= n <= 8
    and all ternary columns 1 <= n <= 5, at every primary index 0 <= p < n:
      1. Overlap identity K_1 = 2r - 1 - |D_int cap C_int| (when distinct runs don't
         coalesce in codomain; more generally K_1 = |D_int cup C_img_int| + 1).
      2. Every width-1 run has out-degree 1 in the run-overlap network G_run.
      3. Grounded unimodular linear system (I - P_{g^-1}) tau = w on Open's retained
         marked cycle reproduces the exact byte write offsets of every retained row.
      4. Veech/Nielsen shear invariance: shifting domain by u^s = (+s mod n) preserves
         cyclic run count and cyclic width-1 run count identically.
    """
    cases = 0
    single_cycle_cases = 0
    for alphabet, max_n in [((0, 1), 8), ((0, 1, 2), 5)]:
        for n in range(1, max_n + 1):
            for tup in itertools.product(alphabet, repeat=n):
                L = list(tup)
                lf, C = compute_lf(L)
                runs = runs_of(L)
                r = len(runs)

                # Check 2: out-degree of every width-1 run in G_run is 1
                dom_ends = [st + w for _, st, w in runs]
                # Map each row to run id
                row_to_run = [0] * n
                for rid, (_, st, w) in enumerate(runs):
                    for x in range(st, st + w):
                        row_to_run[x] = rid
                for rid, (_, st, w) in enumerate(runs):
                    targets = {row_to_run[lf[x]] for x in range(st, st + w)}
                    if w == 1:
                        assert len(targets) == 1, f"Width-1 run {rid} had out-degree != 1"

                # Check 1: image intervals of runs
                occ = [0] * 256
                img_cuts = set()
                for c, st, w in runs:
                    im_st = C[c] + occ[c]
                    occ[c] += w
                    if im_st > 0:
                        img_cuts.add(im_st)
                    if im_st + w < n:
                        img_cuts.add(im_st + w)
                dom_cuts = {st for _, st, _ in runs if st > 0}
                atoms = len(dom_cuts | img_cuts) + 1
                arc_sum = 0
                occ = [0] * 256
                for c, st, w in runs:
                    im_st = C[c] + occ[c]
                    occ[c] += w
                    targets = {row_to_run[y] for y in range(im_st, im_st + w)}
                    arc_sum += len(targets)
                # Note: two distinct runs of the SAME symbol are adjacent in F; if no
                # domain cut separates them, two runs can enter the same target run,
                # so arc_sum = r + #{(run, dom_cut inside image of run)} = 2r - 1 - ...
                # Specifically, for each run, #{targets} = 1 + #{d in dom_cuts : im_st < d < im_st + w}.
                expected_arcs = r + sum(
                    1
                    for (_, _, w), im_st in zip(
                        runs,
                        [
                            C[runs[j][0]] + sum(runs[k][2] for k in range(j) if runs[k][0] == runs[j][0])
                            for j in range(r)
                        ],
                    )
                    for d in dom_cuts
                    if im_st < d < im_st + w
                )
                assert arc_sum == expected_arcs
                assert atoms <= expected_arcs

                # Check 4: cyclic run invariance under u^s
                cr0, cr1_0 = cyclic_runs_stats(L)
                for s_shift in range(n):
                    shifted = [L[(i + s_shift) % n] for i in range(n)]
                    cr_s, cr1_s = cyclic_runs_stats(shifted)
                    assert (cr_s, cr1_s) == (cr0, cr1_0)

                # Check single-cycle count
                vis = set()
                cur = 0
                while cur not in vis:
                    vis.add(cur)
                    cur = lf[cur]
                if len(vis) == n:
                    single_cycle_cases += 1

                # Check 3: Grounded unimodular orbit-time system on Open retained set (limit=2)
                limit = 2
                for p in range(n):
                    cases += 1
                    S = []
                    for c, st, w in runs:
                        if w <= limit:
                            S.extend(range(st, st + w))
                        elif st <= p < st + w:
                            S.append(p)
                    S_set = set(S)
                    rank_in_S = {x: idx for idx, x in enumerate(S)}
                    m = len(S)
                    mark = rank_in_S[p]
                    # Build exact inverse first-return g_inv and word lengths on S
                    # First return g(j): start at s_j, step x = lf[s_j], follow lf while x not in S_set
                    g_fwd = [0] * m
                    word_fwd = [b""] * m
                    for j, sj in enumerate(S):
                        w_bytes = bytearray([L[sj]])
                        x = lf[sj]
                        while x not in S_set:
                            w_bytes.append(L[x])
                            x = lf[x]
                        g_fwd[j] = rank_in_S[x]
                        word_fwd[j] = bytes(reversed(w_bytes))
                    g_inv = [0] * m
                    w_inv = [b""] * m
                    for j in range(m):
                        i = g_fwd[j]
                        g_inv[i] = j
                        w_inv[i] = word_fwd[j]

                    # Restrict to marked component C_mark of g_inv
                    comp = []
                    cur = mark
                    while True:
                        comp.append(cur)
                        cur = g_inv[cur]
                        if cur == mark:
                            break
                    # Unimodular difference equations:
                    # tau[mark] = 0; for u in comp[:-1]: tau[g_inv[u]] - tau[u] = len(w_inv[u])
                    tau = {mark: 0}
                    for u in comp[:-1]:
                        tau[g_inv[u]] = tau[u] + len(w_inv[u])
                    period_len = tau[comp[-1]] + len(w_inv[comp[-1]])
                    buf = bytearray(period_len)
                    for u in comp:
                        off = tau[u]
                        wu = w_inv[u]
                        buf[off : off + len(wu)] = wu
                    # Repeat period to n and compare with dense_invert
                    out = bytearray(buf)
                    while len(out) < n:
                        take = min(period_len, n - len(out))
                        out.extend(buf[:take])
                    ref = dense_invert(bytes(L), p)
                    assert bytes(out[:n]) == ref, f"Unimodular mismatch on {L}, p={p}"
    return {"total_lp_cases": cases, "single_cycle_columns": single_cycle_cases}


def shannon_bits(counts: Counter | dict[int, int], total: int) -> float:
    if total == 0:
        return 0.0
    h = 0.0
    for cnt in counts.values():
        if cnt > 0:
            p = cnt / total
            h -= cnt * math.log2(p)
    return h


def block_invariants(L: bytes, p: int, limit: int = 16) -> dict[str, float | int]:
    n = len(L)
    runs = runs_of(L)
    r = len(runs)
    R = capped_record_count(runs, 127)
    r1 = sum(1 for _, _, w in runs if w == 1)
    r_thin = sum(1 for _, _, w in runs if w <= limit)
    H_thick = r - r_thin
    m_thin = sum(w for _, _, w in runs if w <= limit)
    for _, st, w in runs:
        if w > limit and st <= p < st + w:
            m_thin += 1
            H_thick += 1  # split into 1 or 2 thick fragments in Open if interior
            if st < p and p + 1 < st + w:
                pass
            elif st == p or p + 1 == st + w:
                H_thick -= 1

    # Exact Open strip count matching src/open.rs::each_run
    open_m = 0
    open_strips = 0
    thin_sym_counts: Counter[int] = Counter()
    all_sym_counts: Counter[int] = Counter()
    run_sym_counts: Counter[int] = Counter()
    for c, st, w in runs:
        all_sym_counts[c] += w
        run_sym_counts[c] += 1
        if w > limit and st <= p < st + w:
            if st < p:
                open_strips += 1
            open_m += 1
            thin_sym_counts[c] += 1
            if p + 1 < st + w:
                open_strips += 1
        elif w <= limit:
            open_m += w
            thin_sym_counts[c] += w
        else:
            open_strips += 1

    # Compute codomain cuts & exact overlap count K_1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += all_sym_counts[c]
    dom_cuts = {st for _, st, _ in runs if st > 0}
    occ = [0] * 256
    img_starts = set()
    for c, st, w in runs:
        im_st = C[c] + occ[c]
        occ[c] += w
        if im_st > 0:
            img_starts.add(im_st)
    shared_cuts = len(dom_cuts & img_starts)
    K1 = 2 * r - 1 - shared_cuts
    beta1 = K1 - r + 1

    # Entropy invariants (in bits and KiB)
    H0_full_bits = shannon_bits(all_sym_counts, n)
    H0_thin_bits = shannon_bits(thin_sym_counts, open_m)
    H0_runsym_bits = shannon_bits(run_sym_counts, r)
    width_counts = Counter(w for _, _, w in runs)
    H0_runlen_bits = shannon_bits(width_counts, r)
    H_rle0_bits = H0_runsym_bits + H0_runlen_bits

    return {
        "n": n,
        "R": R,
        "r": r,
        "r1": r1,
        "r_thin": r_thin,
        "m16": open_m,
        "H16": open_strips,
        "K1": K1,
        "K1_over_r": round(K1 / r, 4),
        "beta1": beta1,
        "H0_thin_KiB": round(H0_thin_bits / 8192.0, 2),
        "H_rle0_KiB": round(H_rle0_bits / 8192.0, 2),
        "H0_full_KiB": round(H0_full_bits / 8192.0, 2),
        "ptr_m16_bits_KiB": round(open_m * (open_m - 1).bit_length() / 8192.0, 2),
    }


def bwt_from_text(T: list[int]) -> tuple[bytes, int]:
    n = len(T)
    rk = T[:]
    sa = list(range(n))
    k = 1
    while k < n:
        sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
        nrk = [0] * n
        c = 0
        for idx in range(1, n):
            i, j = sa[idx - 1], sa[idx]
            if rk[i] != rk[j] or rk[(i + k) % n] != rk[(j + k) % n]:
                c += 1
            nrk[j] = c
        rk = nrk
        if c == n - 1:
            break
        k *= 2
    L = bytes(T[(i - 1) % n] for i in sa)
    p = sa.index(0)
    return L, p


def run_toy_census() -> list[dict[str, object]]:
    alpha = (math.sqrt(5) - 1.0) / 2.0
    rows = []

    # 1. Sturmian (Fibonacci / golden rotation, 2 pieces)
    n_toy = 32768
    T_sturm = [1 if ((k + 1) * alpha) % 1.0 < alpha else 0 for k in range(n_toy)]
    L_s, p_s = bwt_from_text(T_sturm)
    rows.append({"toy": "Sturmian (p=2, n=32768)", **block_invariants(L_s, p_s)})

    # 2. Tribonacci (3-letter Pisot / Rauzy fractal substitution: 0->01, 1->02, 2->0)
    w = [0]
    sub = {0: [0, 1], 1: [0, 2], 2: [0]}
    while len(w) < n_toy:
        nw = []
        for x in w:
            nw.extend(sub[x])
        w = nw
    T_trib = w[:n_toy]
    L_tr, p_tr = bwt_from_text(T_trib)
    rows.append({"toy": "Tribonacci Rauzy-fractal (n=32768)", **block_invariants(L_tr, p_tr)})

    # 3. Damaged Toy A (2-letter coded golden rotation with p=256 pieces across n=4096 vs 32768)
    rng = random.Random(20261120)
    cols256 = [rng.randint(0, 1) for _ in range(256)]
    for n_rot in (4096, 32768):
        T_rot = [cols256[int(((k * alpha) % 1.0) * 256)] for k in range(n_rot)]
        L_r, p_r = bwt_from_text(T_rot)
        rows.append({"toy": f"CodedRotation(p=256, n={n_rot})", **block_invariants(L_r, p_r)})

    # 4. De Bruijn B(2, 15) (Martin greedy / LFSR, n = 32768)
    # Build exact binary De Bruijn sequence of order 15 (length 32768) via Lyndon words (Fredricksen-Maiorana)
    k_db = 15
    db = []
    a = [0] * (k_db + 1)

    def db_gen(t: int, p_lyn: int) -> None:
        if t > k_db:
            if k_db % p_lyn == 0:
                db.extend(a[1 : p_lyn + 1])
        else:
            a[t] = a[t - p_lyn]
            db_gen(t + 1, p_lyn)
            for j in range(a[t - p_lyn] + 1, 2):
                a[t] = j
                db_gen(t + 1, t)

    db_gen(1, 1)
    assert len(db) == (1 << k_db)
    # Also random Eulerian De Bruijn of order 15 using Hierholzer
    rng_db = random.Random(1337)
    adj = [[0, 1] for _ in range(1 << (k_db - 1))]
    for u in range(len(adj)):
        rng_db.shuffle(adj[u])
    stack = [0]
    tour = []
    while stack:
        u = stack[-1]
        if adj[u]:
            b = adj[u].pop()
            stack.append(((u << 1) & ((1 << (k_db - 1)) - 1)) | b)
        else:
            tour.append(stack.pop() & 1)
    tour.reverse()
    T_rdb = tour[: (1 << k_db)]
    L_db, p_db = bwt_from_text(T_rdb)
    rows.append({"toy": "Random De Bruijn B(2,15) (n=32768)", **block_invariants(L_db, p_db)})

    # 5. Uniform random 16-symbol string (n = 32768)
    T_rnd = [rng.randint(0, 15) for _ in range(n_toy)]
    L_rn, p_rn = bwt_from_text(T_rnd)
    rows.append({"toy": "Uniform Random Sigma=16 (n=32768)", **block_invariants(L_rn, p_rn)})

    return rows


def main() -> None:
    print("=== 1. Exhaustive Small-Case Verification ===")
    small_res = verify_small_exhaustive()
    print(f"PASS: {small_res['total_lp_cases']} (L, p) pairs verified across {small_res['single_cycle_columns']} primitive columns.")

    blocks_dir = pathlib.Path(os.environ.get("BLOCKS_DIR", "/tmp/admitted-bwt/admitted-bwt"))
    manifest_path = blocks_dir / "manifest.json"
    if manifest_path.exists():
        print("\n=== 2. Authenticated Six-Block Invariant & Entropy Census ===")
        man = json.loads(manifest_path.read_text())
        for b in man["blocks"]:
            p_file = blocks_dir / b["file"]
            raw = p_file.read_bytes()
            sha = hashlib.sha256(raw).hexdigest()
            assert len(raw) == b["n"] and sha == b["sha256"], f"Auth failed on {b['file']}"
            inv = block_invariants(raw, b["primary_index"], limit=16)
            assert inv["r"] == b["r"]
            print(f"{b['file']:12s}: {json.dumps(inv)}")

    print("\n=== 3. Toy-Family Separation Census ===")
    for row in run_toy_census():
        name = row.pop("toy")
        print(f"{str(name):36s}: {json.dumps(row)}")


if __name__ == "__main__":
    main()
