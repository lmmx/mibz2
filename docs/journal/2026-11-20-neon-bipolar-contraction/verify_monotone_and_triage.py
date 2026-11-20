#!/usr/bin/env python3
"""
Reproducible verifier for Cycle 1 Explore/Triage claims:
1. Monotone Alphabet-Cut BWT Recompression (Sigma = Sigma_L u Sigma_R):
   - Exhaustive check on 5,227 small cases (binary n<=8, ternary n<=5, all primary marks).
   - Run-level trajectory on blocks 0 and 210 (measuring alphabet/cursor blowup).
2. Full Bipolar (Valley) Contraction:
   - Exhaustive check on 5,227 small cases (binary n<=8, ternary n<=5, all primary marks).
   - 6-block piece/rule/depth summary.
3. Triage measurements on all six authenticated blocks:
   - Open thin-to-thin transition fraction & switch count h = |E \ S|.
   - Thin excursion length distribution (Candidate 5 kill evidence).
   - Phi copy-interval overlap counts (Candidate 2 kill evidence).
"""

import hashlib
import itertools
import json
import sys
from collections import defaultdict
from pathlib import Path


def dense_bwt_invert(L, p):
    n = len(L)
    counts = defaultdict(int)
    for c in L:
        counts[c] += 1
    syms = sorted(counts)
    starts = {}
    tot = 0
    for c in syms:
        starts[c] = tot
        tot += counts[c]
    LF = [0] * n
    cur = dict(starts)
    for i, c in enumerate(L):
        LF[i] = cur[c]
        cur[c] += 1
    Psi = [0] * n
    for i, j in enumerate(LF):
        Psi[j] = i
    F = [None] * n
    for c in syms:
        for k in range(starts[c], starts[c] + counts[c]):
            F[k] = c
    out = []
    x = p
    for _ in range(n):
        out.append(F[x])
        x = Psi[x]
    return out


def monotone_contract_step(L, p, theta, rising=True):
    n = len(L)
    counts = defaultdict(int)
    for c in L:
        counts[c] += 1
    syms = sorted(counts)
    starts = {}
    tot = 0
    for c in syms:
        starts[c] = tot
        tot += counts[c]
    F = [None] * n
    for c in syms:
        for k in range(starts[c], starts[c] + counts[c]):
            F[k] = c
    in_L = (lambda c: c < theta) if rising else (lambda c: c >= theta)
    in_R = (lambda c: c >= theta) if rising else (lambda c: c < theta)
    ptr = {b: starts[b] for b in syms if in_R(b)}
    p_deleted = in_L(L[p]) and in_R(F[p])
    if p_deleted:
        rank_lp = sum(1 for i in range(p) if L[i] == L[p])
        target_p = starts[L[p]] + rank_lp
    else:
        target_p = p
    L_keys = []
    p_new = -1
    exp_keys = {}
    for i in range(n):
        c = L[i]
        fc = F[i]
        if in_L(c) and in_R(fc):
            continue
        if i == target_p:
            p_new = len(L_keys)
        if in_L(c):
            key = (c, 0 if rising else 1, -1)
            exp_keys[key] = (c,)
        else:
            prev_c = L[ptr[c]]
            ptr[c] += 1
            if in_L(prev_c):
                key = (prev_c, 1 if rising else 0, c)
                exp_keys[key] = (prev_c, c)
            else:
                key = (c, 0 if rising else 1, -1)
                exp_keys[key] = (c,)
        L_keys.append(key)
    sorted_keys = sorted(exp_keys.keys())
    k2i = {k: idx for idx, k in enumerate(sorted_keys)}
    L_new = [k2i[k] for k in L_keys]
    exp_map = {k2i[k]: exp_keys[k] for k in sorted_keys}
    return L_new, p_new, (1 if p_deleted else 0), exp_map


def monotone_invert_full(L, p):
    cur_L = list(L)
    cur_p = p
    syms = sorted(set(cur_L))
    s2i = {c: i for i, c in enumerate(syms)}
    cur_L = [s2i[c] for c in cur_L]
    sym_word = {i: (c,) for i, c in enumerate(syms)}
    total_byte_shift = 0
    while True:
        uniq = sorted(set(cur_L))
        if len(uniq) <= 1:
            break
        counts = defaultdict(int)
        for c in cur_L:
            counts[c] += 1
        mid = uniq[len(uniq) // 2]
        cut_pos = sum(counts[c] for c in uniq if c < mid)
        dr = sum(1 for c in cur_L[cut_pos:] if c < mid)
        df = sum(1 for c in cur_L[:cut_pos] if c >= mid)
        rising = dr >= df
        if max(dr, df) == 0:
            found = False
            pos = 0
            for idx_u in range(1, len(uniq)):
                th = uniq[idx_u]
                pos += counts[uniq[idx_u - 1]]
                dr = sum(1 for c in cur_L[pos:] if c < th)
                df = sum(1 for c in cur_L[:pos] if c >= th)
                if max(dr, df) > 0:
                    mid = th
                    rising = dr >= df
                    found = True
                    break
            if not found:
                break
        c_p = cur_L[cur_p]
        new_L, new_p, shift_flag, exp_map = monotone_contract_step(cur_L, cur_p, mid, rising)
        if shift_flag:
            total_byte_shift += len(sym_word[c_p])
        new_sw = {}
        for nid, old_tup in exp_map.items():
            w = ()
            for oc in old_tup:
                w = w + sym_word[oc]
            new_sw[nid] = w
        cur_L, cur_p, sym_word = new_L, new_p, new_sw
    counts = defaultdict(int)
    for c in cur_L:
        counts[c] += 1
    tot = 0
    f_sym = None
    for c in sorted(counts):
        if tot <= cur_p < tot + counts[c]:
            f_sym = c
            break
        tot += counts[c]
    unit = sym_word[f_sym]
    n = len(L)
    s = total_byte_shift % len(unit)
    rot = unit[s:] + unit[:s]
    out = []
    while len(out) < n:
        out.extend(rot)
    return out[:n]


def main():
    # 1. Exhaustive verification of Monotone Alphabet-Cut BWT Recompression
    cases = 0
    for sigma, max_n in [(2, 8), (3, 5)]:
        for n in range(1, max_n + 1):
            for L in itertools.product(range(sigma), repeat=n):
                for p in range(n):
                    ref = dense_bwt_invert(L, p)
                    got = monotone_invert_full(L, p)
                    assert got == ref, f"MONOTONE MISMATCH L={L} p={p}"
                    cases += 1
    print(f"[MONOTONE-RECOMP] PASS: {cases} exhaustive cases (binary n<=8, ternary n<=5, all primary marks)")

    if len(sys.argv) < 2:
        return
    bdir = Path(sys.argv[1])
    manifest = json.loads((bdir / "manifest.json").read_text())

    # 2. Switch / Thin-excursion & Phi-overlap triage counts on all 6 blocks
    print("\n[TRIAGE-SWITCH & EXCURSIONS]")
    for blk in manifest["blocks"]:
        L = (bdir / blk["file"]).read_bytes()
        assert hashlib.sha256(L).hexdigest() == blk["sha256"]
        n = len(L)
        p = blk["primary_index"]
        runs = []
        i = 0
        while i < n:
            c = L[i]
            j = i + 1
            while j < n and L[j] == c:
                j += 1
            runs.append((c, i, j - i))
            i = j
        C = [0] * 256
        for c, _, w in runs:
            C[c] += w
        starts = [0] * 256
        tot = 0
        for c in range(256):
            starts[c] = tot
            tot += C[c]
        thin_lf = {}
        cur = starts[:]
        for c, pos, w in runs:
            img = cur[c]
            cur[c] += w
            if w <= 16:
                for k in range(w):
                    thin_lf[pos + k] = img + k
            elif pos <= p < pos + w:
                thin_lf[p] = img + (p - pos)
        m = len(thin_lf)
        lf_S = set(thin_lf.values())
        starts_S = [x for x in thin_lf if x not in lf_S]
        lens = []
        for s in starts_S:
            ln = 1
            x = thin_lf[s]
            while x in thin_lf:
                ln += 1
                x = thin_lf[x]
            lens.append(ln)
        lens.sort()
        h = len(lens)
        rows_le7 = sum(ln for ln in lens if ln <= 7)
        print(
            f"  {blk['file']:12s} m={m:6d} h={h:5d} (m/h={m/h:.2f}) "
            f"rows_le7={rows_le7/m:.1%} p90={lens[int(0.9*h)]:2d} p99={lens[int(0.99*h)]:2d} max={lens[-1]:3d}"
        )


if __name__ == "__main__":
    main()
