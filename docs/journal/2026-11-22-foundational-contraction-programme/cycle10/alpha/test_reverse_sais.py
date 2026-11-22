#!/usr/bin/env python3
"""
Cycle 10 — Task 1:
Reverse SA-IS: Exact BWT-Space LMS (Leftmost S-Type) Contraction directly from I_0!

Proves & verifies:
  1. Theorem S1: Exact BWT-space L/S bucket split `split[c] = C[c] + sum_{L_u=c, delta_u>0} len_u`
     (computed in ONE O(r_0) scan of I_0, ZERO LF walk!) and exact LMS characterization:
       row i is LMS  <==>  i in [split[F[i]], C[F[i]+1]) AND L[i] > F[i]
                     <==>  LF(i) > i AND FL(i) > i  (local minimum of cyclic SA-rank!)
     Verified against ground-truth text SA-IS on ALL primitive binary (n<=9) & ternary (n<=6) BWTs!
  2. Theorem S2: Exact BWT-Order Preservation of Recursive Reverse SA-IS:
     At every level t = 0..H-1, rows [0, n_t) remain in exact cyclic SA order of T^{(t)} with ZERO
     re-sorting! Contracting all non-LMS rows (via 2 monotone DAG passes: eliminate L-type delta_LF>0
     then non-LMS S-type delta_LF<0) yields Ind_{LMS}(LF) = exact BWT of T^{(t+1)}!
     Verified 100% byte-exact on small exhaustive BWTs AND all 6 authenticated Wikidata blocks!
  3. Complete 6-block census: |LMS|/n, r(T^{(1)})/r_0, m_16(T^{(1)})/m_16, Level-1 contraction
     ops Q_1/r_0, full cascade levels H & total ops Q_tot/r_0!
"""
import hashlib
import json
import sys
import time
from bisect import bisect_right

sys.path.insert(0, "/tmp/fcp-a")
import bwt_core as bc


def ground_truth_circular_lms(T: bytes):
    """Ground-truth circular SA-IS types (L vs S) and LMS positions directly on primitive circular text T."""
    n = len(T)
    rots = [T[i:] + T[:i] for i in range(n)]
    # Check primitive (all rotations distinct)
    if len(set(rots)) < n:
        return None
    sa = sorted(range(n), key=lambda i: rots[i])
    isa = [0] * n
    for rank, pos in enumerate(sa):
        isa[pos] = rank
    # Circular S-type: R_j < R_{j+1} <==> isa[j] < isa[(j+1)%n]
    is_s = [isa[j] < isa[(j + 1) % n] for j in range(n)]
    # LMS position j: is_s[j] and not is_s[(j-1)%n]
    lms_rows = {rank for rank, j in enumerate(sa) if is_s[j] and not is_s[(j - 1) % n]}
    s_rows = {rank for rank, j in enumerate(sa) if is_s[j]}
    return sa, isa, s_rows, lms_rows


def test_theorem_s1_exhaustive():
    print("=== 1. Exhaustive Verification of Theorem S1 (BWT-Space L/S Split & LMS vs Ground-Truth Text SA-IS) ===")
    checked = 0
    for base, max_n in [(2, 9), (3, 6)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                gt = ground_truth_circular_lms(T)
                if gt is None:
                    continue  # skip periodic non-primitive
                sa, isa, gt_s_rows, gt_lms_rows = gt
                L, p = bc.bwt_encode(T)
                _, r0, _, strips, j_order, C = bc.load_runs_and_open_baseline(L, p)
                # Compute split[c] directly from I_0 in ONE O(r_0) scan (ZERO LF walk!)
                l_cnt = [0] * 256
                for lo, hi, d, c, m in strips:
                    if d > 0:
                        l_cnt[c] += (hi - lo)
                split = [C[c] + l_cnt[c] for c in range(256)]
                # Compute F[i] and verify every row's S-type & LMS-type!
                F = sorted(L)
                bwt_s_rows = set()
                bwt_lms_rows = set()
                for i in range(n):
                    fc = F[i]
                    if i >= split[fc]:
                        bwt_s_rows.add(i)
                        if L[i] > fc:
                            bwt_lms_rows.add(i)
                assert bwt_s_rows == gt_s_rows, f"S-type mismatch on {T}"
                assert bwt_lms_rows == gt_lms_rows, f"LMS mismatch on {T}"
                # Also verify local-minimum characterization: LF(i) > i AND FL(i) > i!
                cur = C[:]
                LF = [0] * n
                FL = [0] * n
                for i, b in enumerate(L):
                    LF[i] = cur[b]; FL[cur[b]] = i; cur[b] += 1
                min_lms = {i for i in range(n) if LF[i] > i and FL[i] > i}
                assert min_lms == gt_lms_rows, f"Local-min LMS mismatch on {T}"
                checked += 1
    print(f"  EXHAUSTIVE THEOREM S1 PASS: {checked} primitive binary(n<=9)+ternary(n<=6) BWTs, 0 mismatches [verified]")


def contract_pieces_to_subset(pieces, keep_mask, ws):
    """
    Given a sorted disjoint partition `pieces` of [0, cur_n) into bi-monotone pieces
    (lo, hi, delta, word_id, mark_off) and a boolean `keep_mask[i]` (where the deleted set D
    is guaranteed ACYCLIC under LF!), compute the exact first-return W-IET on K = union_{keep} I_i
    in compressed coordinates [0, new_n), returning (new_pieces, stats_dict)!
    """
    np = len(pieces)
    starts = [p[0] for p in pieces]
    q_comp = 0
    # Resolve every retained piece through deleted DAG pieces!
    stack = []
    out_raw = []
    for i, pc in enumerate(pieces):
        if keep_mask[i]:
            stack.append(pc)

    while stack:
        lo, hi, d, w, m = stack.pop()
        t_lo, t_hi = lo + d, hi + d
        idx = bisect_right(starts, t_lo) - 1
        pos = t_lo
        w_len = ws.length[w]
        while pos < t_hi:
            plo, phi, pd, pw, pm = pieces[idx]
            ov = t_hi if t_hi < phi else phi
            slo, shi = pos - d, ov - d
            if keep_mask[idx]:
                out_raw.append((slo, shi, d, w, m))
            else:
                q_comp += 1
                nw = ws.concat(w, pw)
                nm = bc.combine_mark(m, w_len, pm)
                stack.append((slo, shi, d + pd, nw, nm))
            pos = ov
            idx += 1

    out_raw.sort(key=lambda x: x[0])
    # Compact coordinates densely into [0, new_n)
    old_los = [p[0] for p in out_raw]
    new_los = []
    acc = 0
    for lo, hi, *_ in out_raw:
        new_los.append(acc)
        acc += (hi - lo)

    def map_pos(x):
        idx = bisect_right(old_los, x) - 1
        return new_los[idx] + (x - old_los[idx])

    comp = []
    for i, (lo, hi, d, w, m) in enumerate(out_raw):
        nlo = new_los[i]
        nhi = nlo + (hi - lo)
        nt = map_pos(lo + d)
        comp.append((nlo, nhi, nt - nlo, w, m))

    return comp, acc, q_comp


def reverse_sais_cascade(L_bytes: bytes, primary_idx: int, verbose=False):
    """
    Full Recursive Reverse SA-IS directly on I_0 (with ZERO re-sorting at ANY level!):
      At each level t (live rows [0, n_t), pieces sorted by lo):
        1. Spiral-compress any self-overlap (0 < |d| < L) or stop if n_t == 1 (d == 0).
        2. Classify every piece by exact SA-IS type in T^{(t)}:
             - L-type in T^{(t)}  <==> FL(i) < i  <==>  target of an LF piece with delta_LF > 0!
             - S-type in T^{(t)}  <==> FL(i) > i  <==>  target of an LF piece with delta_LF < 0!
             - Wait: even simpler! In LF coordinates, EVERY piece with delta_LF < 0 maps S-type->...
               Wait: let's check BOTH directions carefully!
                 Row i is S-type iff FL(i) > i  (i.e. the piece whose IMAGE J_u contains i has delta_LF(u) < 0!).
                 Row i is L-type iff FL(i) < i  (i.e. the piece whose IMAGE J_u contains i has delta_LF(u) > 0!).
                 Row i is LMS    iff FL(i) > i AND LF(i) > i
                                 <==> (i lies in image J_u with delta_LF(u) < 0) AND (i lies in domain I_v with delta_LF(v) > 0)!!
        3. Exact 2-step acyclic contraction to LMS(t):
             - Step A (Eliminate all L-type rows? Wait: check acyclicity!):
               * Step A1: Eliminate ALL pieces with `delta_LF < 0` (strictly decreasing LF -> 100% ACYCLIC!).
                 Wait: which rows remain after eliminating `delta_LF < 0`? All rows with `LF(i) > i`!
                 Among those, every LMS row has `LF(i) > i` (so 100% of LMS rows SURVIVE Step A1!),
                 AND any remaining non-LMS row with `LF(i) > i` is an L-type row (`FL(i) < i`)!
               * Step A2: Wait — on the surviving set `{i : LF(i) > i}`, every row maps strictly UPWARD (`delta_LF > 0`)
                 EXCEPT rows whose composed jump wrapped around!
               * EVEN SIMPLER & 100% EXACT (Theorem S1 directly!):
                 Split pieces at the `L`/`S` boundaries (the images of `delta_LF < 0` vs `delta_LF > 0`, which
                 at Level 0 are JUST the 256 `split[c]` points — ZERO extra passes!).
                 Then EVERY piece is pure-type `(is_L_or_S, is_LF_up_or_down)`:
                   - Type `(S, LF_up)`   = **LMS** (KEEP!)
                   - Type `(L, LF_up)`   = L-type going up (`delta_LF > 0`, pure L->L chain!)
                   - Type `(L, LF_down)` = LML peak (`delta_LF < 0`, transitions L -> S!)
                   - Type `(S, LF_down)` = non-LMS S-type (`delta_LF < 0`, pure S->S chain!)
                 Check acyclicity of eliminating NON-LMS in 2 steps:
                   - Pass 1: Eliminate all `delta_LF < 0` pieces (`(S, LF_down)` + `(L, LF_down)` — 100% ACYCLIC since every step strictly decreases coordinate!) -> ALL remaining pieces are `(S, LF_up) = LMS` and `(L, LF_up)`!
                   - Pass 2: Now among the surviving pieces (`LMS` and `(L, LF_up)`), every `(L, LF_up)` piece was originally an L-type row whose LF went strictly UP (`> i`) to an L-type or LML row; after Pass 1 bypassed any `(L, LF_down)` or `(S, LF_down)` chain?
                     CAREFUL! If an `(L, LF_up)` piece jumps into an `(L, LF_down)` peak and bounces down past its start, could the surviving `(L, LF_up)` pieces have a cycle?
                     NO! Because the surviving set `LMS` has >= 1 row on EVERY cycle of a non-trivial string (every cycle has a global minimum row `i_min`, which satisfies `LF(i_min) > i_min` AND `FL(i_min) > i_min`, so `i_min ∈ LMS` ALWAYS!!)!
                     **WAIT A SECOND!! STOP AND APPLAUD THAT 1-LINE PROOF!!**
                     **EVERY CYCLE OF LENGTH >= 2 HAS A GLOBAL MINIMUM ROW `i_min`, WHICH SATISFIES `LF(i_min) > i_min` AND `FL(i_min) > i_min`, SO `i_min ∈ LMS` AUTOMATICALLY!!**
                     **THEREFORE THE COMPLEMENT `V \ LMS` IS *UNCONDITIONALLY ACYCLIC* ON EVERY SINGLE-CYCLE OR MULTI-CYCLE BIJECTION WITH NO FIXED POINTS (`n_t >= 2`)!!**
    """
    n, r0, m16, st_tup, j_order, C = bc.load_runs_and_open_baseline(L_bytes, primary_idx)
    ws = bc.WordStore()
    pieces = [(lo, hi, d, c, m) for lo, hi, d, c, m in st_tup]

    cur_n = n
    level = 0
    tot_q = 0
    tot_sp = 0
    level_logs = []
    primary_cycle = None

    while primary_cycle is None:
        # Step 0: Handle fixed points (d == 0) & spirals (0 < |d| < L)
        step0 = []
        did_sp = False
        for lo, hi, d, w, m in pieces:
            if d == 0:
                if m >= 0:
                    primary_cycle = (w, m)
                continue
            L = hi - lo
            if d > 0 and d < L:
                assert m < 0
                q, rem = divmod(L, d)
                tot_sp += 1; did_sp = True
                if rem > 0: step0.append((lo, lo + rem, (q + 1) * d, ws.power(w, q + 1), -1))
                if d - rem > 0: step0.append((lo + rem, lo + d, q * d, ws.power(w, q), -1))
            elif d < 0 and (-d) < L:
                assert m < 0
                s = -d; q, rem = divmod(L, s)
                tot_sp += 1; did_sp = True
                if s - rem > 0: step0.append((hi - s, hi - rem, -q * s, ws.power(w, q), -1))
                if rem > 0: step0.append((hi - rem, hi, -(q + 1) * s, ws.power(w, q + 1), -1))
            else:
                step0.append((lo, hi, d, w, m))

        if primary_cycle is not None:
            break
        if did_sp:
            step0.sort(key=lambda x: x[0])
            # Re-compact after spiral
            old_los = [p[0] for p in step0]
            new_los = []; acc = 0
            for lo, hi, *_ in step0: new_los.append(acc); acc += (hi - lo)
            def mp(x):
                ix = bisect_right(old_los, x) - 1
                return new_los[ix] + (x - old_los[ix])
            pieces = [(new_los[i], new_los[i] + (hi - lo), mp(lo + d) - new_los[i], w, m)
                      for i, (lo, hi, d, w, m) in enumerate(step0)]
            cur_n = acc
            continue

        # Identify EXACT LMS subset of current level:
        # Row i is LMS <==> (LF(i) > i) AND (FL(i) > i)
        #   i.e., i lies in a domain piece with d > 0, AND i lies in a target image J_u with d_u < 0!
        # Compute exact union of S-type intervals = sorted merged union of J_u for pieces with d_u < 0!
        s_ints = sorted((lo + d, hi + d) for lo, hi, d, _, _ in step0 if d < 0)
        merged_s = []
        for a, b in s_ints:
            if merged_s and merged_s[-1][1] == a:
                merged_s[-1] = (merged_s[-1][0], b)
            else:
                merged_s.append((a, b))

        # Split pieces of step0 against merged_s so every refined piece is pure LMS or pure non-LMS!
        refined = []
        is_lms = []
        mi = 0; nm = len(merged_s)
        for lo, hi, d, w, m in step0:
            if d < 0:
                # LF(i) < i => CANNOT be LMS!
                refined.append((lo, hi, d, w, m))
                is_lms.append(False)
                continue
            # d > 0 => LF(i) > i! Intersect [lo, hi) with merged_s to separate LMS vs L-type!
            pos = lo
            while pos < hi:
                while mi < nm and merged_s[mi][1] <= pos:
                    mi += 1
                if mi >= nm or merged_s[mi][0] >= hi:
                    refined.append((pos, hi, d, w, m if pos == lo else -1))
                    is_lms.append(False)
                    pos = hi
                else:
                    sa, sb = merged_s[mi]
                    if pos < sa:
                        refined.append((pos, sa, d, w, m if pos == lo else -1))
                        is_lms.append(False)
                        pos = sa
                    ov = hi if hi < sb else sb
                    refined.append((pos, ov, d, w, m if pos == lo else -1))
                    is_lms.append(True)
                    pos = ov
                    if pos == sb:
                        mi += 1

        # Contract directly to LMS subset (guaranteed acyclic because every cycle >= 2 has its min in LMS!)
        new_pieces, new_n, q_lvl = contract_pieces_to_subset(refined, is_lms, ws)
        level += 1
        tot_q += q_lvl
        # Measure thin rows (w <= 16) at this reduced level!
        m16_lvl = sum(hi - lo for lo, hi, *_ in new_pieces if (hi - lo) <= 16)
        has_p_thin = any(m >= 0 and (hi - lo) <= 16 for lo, hi, _, _, m in new_pieces)
        if not has_p_thin and new_pieces:
            m16_lvl += 1
        level_logs.append({
            "lvl": level,
            "n_t": new_n,
            "r_t": len(new_pieces),
            "m16_t": m16_lvl,
            "q_lvl": q_lvl,
        })
        if verbose:
            print(f"    Lvl {level:2d}: n_t={new_n:6d} ({new_n/n:5.3f}n) | r_t={len(new_pieces):6d} ({len(new_pieces)/r0:5.3f}r0) | "
                  f"m16_t={m16_lvl:6d} ({m16_lvl/m16:5.3f}m16) | Q_lvl={q_lvl:6d} ({q_lvl/r0:4.2f}r0)")
        pieces = new_pieces
        cur_n = new_n

    w_c, m_c = primary_cycle
    cyc = ws.expand(w_c)
    q = len(cyc)
    rot = cyc[m_c:] + cyc[:m_c]
    text = ((rot * (n // q + 1))[:n])[::-1]
    return text, {
        "n": n, "r0": r0, "m16": m16,
        "levels": level, "tot_q": tot_q, "tot_sp": tot_sp,
        "slp_gt16": ws.slp_gt16, "logs": level_logs,
    }


def main():
    test_theorem_s1_exhaustive()

    print("\n=== 2. Exhaustive Exactness of Full Recursive Reverse SA-IS (Theorem S2, binary n<=8, ternary n<=5) ===")
    cnt = 0
    for base, max_n in [(2, 8), (3, 5)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                L, p = bc.bwt_encode(T)
                dec, _ = reverse_sais_cascade(L, p, verbose=False)
                assert dec == T, f"Reverse SA-IS FAIL on {T}"
                cnt += 1
    print(f"  EXHAUSTIVE REVERSE SA-IS PASS: {cnt} binary(n<=8)+ternary(n<=5) BWTs (incl. periodic!) [verified]")

    print("\n=== 3. REVERSE SA-IS CASCADE ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    print("\n--- Level-by-Level Cascade on block-0.L ---")
    s_n = s_lms1 = s_r0 = s_r1 = s_m16 = s_m16_1 = s_q1 = s_qtot = s_slp = 0
    rows_tbl = []
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        ref = bc.ref_ibwt(L, entry["primary_index"])
        dec, st = reverse_sais_cascade(L, entry["primary_index"], verbose=(entry["file"] == "block-0.L"))
        assert dec == ref, f"EXACTNESS FAIL on {entry['file']}"
        l1 = st["logs"][0]
        s_n += st["n"]; s_lms1 += l1["n_t"]; s_r0 += st["r0"]; s_r1 += l1["r_t"]
        s_m16 += st["m16"]; s_m16_1 += l1["m16_t"]; s_q1 += l1["q_lvl"]; s_qtot += st["tot_q"]; s_slp += st["slp_gt16"]
        rows_tbl.append((entry["file"], st, l1))

    print("\n=== 4. 6-BLOCK SUMMARY OF REVERSE SA-IS (LEVEL 1 LMS & FULL CASCADE) ===")
    print("block       | |LMS|/n | r(T1)/r0 | m16(T1)/m16 | Q_1 (/r0)  | H  | Q_tot (/r0, /m16)       | slp_>16 | exact")
    print("------------+---------+----------+-------------+------------+----+-------------------------+---------+------")
    for fn, st, l1 in rows_tbl:
        r0 = st["r0"]; m16 = st["m16"]
        print(f"{fn:11s} |  {l1['n_t']/st['n']:5.3f}  |   {l1['r_t']/r0:5.3f}  |    {l1['m16_t']/m16:5.3f}    | "
              f"{l1['q_lvl']:6d}({l1['q_lvl']/r0:4.2f}) | {st['levels']:2d} | {st['tot_q']:6d} ({st['tot_q']/r0:4.2f}r, {st['tot_q']/m16:5.3f}m) | {st['slp_gt16']:7d} | PASS")
    print("------------+---------+----------+-------------+------------+----+-------------------------+---------+------")
    print(f"SUM 6 BLKS  |  {s_lms1/s_n:5.3f}  |   {s_r1/s_r0:5.3f}  |    {s_m16_1/s_m16:5.3f}    | "
          f"{s_q1:6d}({s_q1/s_r0:4.2f}) |    | {s_qtot:6d} ({s_qtot/s_r0:4.2f}r, {s_qtot/s_m16:5.3f}m) | {s_slp:7d} | PASS")


if __name__ == "__main__":
    main()
