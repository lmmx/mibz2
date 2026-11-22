#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 2B: Exact Run-Space Compositional Evaluation of Ascent-DAG / Descent-DAG
(Checking for Disguised Walks & Unpaid Acquisition!) on the 6 Authenticated Wikidata Blocks
+ Multi-Round Alternating Ascent/Descent-DAG Contraction vs BDMR!
"""
import hashlib
import json
import os
import time
from collections import defaultdict


def load_block(path):
    data = open(path, "rb").read()
    n = len(data)
    # Build uncapped maximal runs of L
    runs = []
    prev = data[0]
    cnt = 1
    for b in data[1:]:
        if b == prev:
            cnt += 1
        else:
            runs.append((prev, cnt))
            prev = b
            cnt = 1
    runs.append((prev, cnt))
    return data, n, runs


def run_space_ascent_dag_step(runs, n, s, mode="asc"):
    """
    EXACT RUN-SPACE (ZERO LF-WALK!) evaluation of 1-shot Ascent-DAG (L < F) or Descent-DAG (L > F)!
    Measures EXACTLY:
      - How many interval pullback pieces are created in reverse topological order!
      - Whether pullback piece count explodes vs r_0 (checking for disguised walk!)
      - Exact output runs (L_S, C_S), verifying 100% parity with exact row-level first-return!
    Algorithm (reverse topological DP on DAG):
      In Ascent-DAG (a < b): deleted rows are (L=a, F=b) with a < b.
      Surviving rows in F_b are those with L >= b.
      Every row y in F_a has FL(y) in some F_b (namely the j-th occurrence of a in L!).
      - If b <= a: FL(y) is in S (survives!), so y's emitted word stops at (a,).
      - If b > a: FL(y) is in D (deleted!), so y's emitted word is (a,) + u_S(FL(y))!
      Now look at the partition of ALL rows of F_b (BOTH surviving and deleted!) by their
      forward-suffix emitted word u_ext(z) = F[z] + u_ext(FL(z)) while continuing along a < b!
      WAIT A SECOND!
      Every row z in F_b has an extended word u_ext(z) starting at b!
      By our Zero-Bucket-Split Theorem, u_ext(z) partitions ALL of F_b into CONTIGUOUS INTERVALS
      (NOT just surviving rows — ALL rows of F_b!! Because u_ext(z) depends ONLY on the suffix
      starting at z, and the BWT rows of F_b are sorted lexicographically by their suffix!!).
      Let P[b] = list of (word_id, width) contiguous intervals partitioning F_b!
      Processing a = s-1 down to 0:
        We scan the runs of L = a across all F-buckets b = 0..s-1 (i.e. FL(F_a) in exact order!).
        - For each run (a, w) lying inside F_b with b <= a:
            All w rows in F_a map to FL in S -> they all have u_ext = (a,) (stopping at a!).
        - For each run (a, w) lying inside F_b with b > a:
            This run occupies a contiguous sub-interval [lo, lo + w) inside F_b!
            We intersect [lo, lo + w) with the ALREADY-COMPUTED partition P[b] of F_b!
            Each intersecting piece (word_id_b, take) maps to (word_id_new = (a, word_id_b), take) in F_a!
      WAIT A SECOND!!
      CHECK THE WORK OF THIS ALGORITHM!!
      1. It NEVER chases LF row-by-row!!
      2. Every pullback piece inside F_b corresponds to a distinct (a, word_id_b) segment inside F_a!
      3. Once P[0..s-1] are computed (total intervals = sum_a |P[a]|), what is L_S?
         For each surviving run (L=a, w) inside F_b (so a >= b):
           Its LF-image is a contiguous sub-interval [occ_a, occ_a + w) inside F_a!
           WAIT: each row i in that surviving run has LF(i) in F_a.
           Wait: does LF(i) survive, or can LF(i) itself be DELETED?!
           AHAHA!! STOP AND CHECK!!
           LF(i) is in F_a, and LF(i) is DELETED iff L[LF(i)] < a!
           If LF(i) is deleted, then LF_S(i) = LF^k(i) goes BACKWARDS through the DAG until hitting S!
           SO computing L_S requires **FORWARD** topological DP on the backward-emitted word (or equivalently,
           partitioning each F_b's surviving rows by their incoming backward chain w_S(i) = u_S(LF_S(i)))!
           WAIT A MINUTE! Let's check BOTH directions:
           - Direction 1 (F-space / forward DP, a = s-1..0): builds the exact partition P[a] of F_a into u_ext words (exact bucket boundaries C_S AND exact SLP grammar rules for every new symbol!).
           - Direction 2 (L-space / backward DP, a = 0..s-1): builds the exact partition Q[a] of L_S across surviving rows!
             For a = 0..s-1: inside F_a, surviving rows have L >= a, deleted rows have L < a.
             Wait: for a deleted row z in F_a (with L[z] = c < a), its LF(z) lands in F_c (c < a!), whose backward-source partition Q[c] is ALREADY KNOWN!
             SO pushing Q[c] forward through the runs of (L=c) inside F_a gives the EXACT L_S-runs on ALL rows of F_a in topological order a = 0..s-1!!
    HOLD EVERYTHING!! DID WE JUST INVENT A 100% RUN-SPACE, ZERO-LF-WALK, 2-SWEEP TOPOLOGICAL DP ALGORITHM FOR ARBITRARY-CHAIN-LENGTH ASCENT-DAG CONTRACTION DIRECTLY FROM I_0?!
    LET'S IMPLEMENT IT AND MEASURE ITS EXACT PIECE COUNT (IS IT O(r_0) OR DOES IT EXPLODE?!) ON ALL 6 WIKIDATA BLOCKS RIGHT NOW!!
    """
    # Build C and bucket-runs br[b] = [(c, w, off_in_b)]
    cnt = [0] * s
    for c, w in runs:
        cnt[c] += w
    C = [0] * (s + 1)
    for c in range(s):
        C[c + 1] = C[c] + cnt[c]

    br = [[] for _ in range(s)]
    pos = 0
    fb = 0
    for c, w in runs:
        rem = w
        cp = pos
        while rem > 0:
            while C[fb + 1] <= cp:
                fb += 1
            tk = min(rem, C[fb + 1] - cp)
            br[fb].append((c, tk, cp - C[fb]))
            cp += tk
            rem -= tk
        pos += w

    # Map order so "asc" deletes L < F (so a -> b with a < b), "desc" deletes L > F (a > b)
    # In general, edge (a -> b) is deleted iff is_del(a, b).
    if mode == "asc":
        fwd_order = list(range(s - 1, -1, -1))   # reverse topo: sinks first (s-1 down to 0)
        bwd_order = list(range(s))               # topo: sources first (0 up to s-1)
        is_del = lambda l, f: l < f
    else:
        fwd_order = list(range(s))               # sinks first (0 up to s-1)
        bwd_order = list(range(s - 1, -1, -1))   # sources first (s-1 down to 0)
        is_del = lambda l, f: l > f

    # Pre-index runs of L=a in order of appearance across buckets b:
    # For each a, list of (b, off_in_b, width)
    runs_of_L = [[] for _ in range(s)]
    for b in range(s):
        for c, w, off_b in br[b]:
            runs_of_L[c].append((b, off_b, w))

    # --- SWEEP 1 (Reverse Topological DP): Compute P[a] = [(word_id, width)] for ALL rows of F_a ---
    # Also track prefix sums of P[b] for fast interval slicing!
    import bisect
    P = [None] * s
    P_starts = [None] * s
    word_id_map = {}  # (a, child_wid_or_None) -> wid
    next_wid = 0
    fwd_pieces_total = 0

    def get_wid(key):
        nonlocal next_wid
        wid = word_id_map.get(key)
        if wid is None:
            wid = next_wid
            word_id_map[key] = wid
            next_wid += 1
        return wid

    for a in fwd_order:
        if cnt[a] == 0:
            P[a] = []
            P_starts[a] = [0]
            continue
        segs = []  # (wid, width)
        for b, off_b, w in runs_of_L[a]:
            if not is_del(a, b):
                # Stops at a!
                wid = get_wid((a, None))
                if segs and segs[-1][0] == wid:
                    segs[-1] = (wid, segs[-1][1] + w)
                else:
                    segs.append((wid, w))
                fwd_pieces_total += 1
            else:
                # Intersect [off_b, off_b + w) with P[b]!
                pb = P[b]
                ps = P_starts[b]
                idx = bisect.bisect_right(ps, off_b) - 1
                cur = off_b
                end = off_b + w
                while cur < end:
                    cwid, cw = pb[idx]
                    seg_end = ps[idx + 1]
                    tk = min(end, seg_end) - cur
                    wid = get_wid((a, cwid))
                    if segs and segs[-1][0] == wid:
                        segs[-1] = (wid, segs[-1][1] + tk)
                    else:
                        segs.append((wid, tk))
                    fwd_pieces_total += 1
                    cur += tk
                    idx += 1
        P[a] = segs
        st = [0]
        acc = 0
        for _, sw in segs:
            acc += sw
            st.append(acc)
        P_starts[a] = st

    # --- SWEEP 2 (Topological DP): Compute Q[b] = [(src_wid, width)] for ALL rows of F_b ---
    # Here src_wid(z) for row z in F_b is the word_id of the surviving row y = LF_S(z)
    # Wait:
    #   - If z is SURVIVING in F_b (i.e. L[z] >= b in asc): its backward step goes to LF(z) in F_{L[z]},
    #     WAIT: we want L_S[z] = u_S(LF_S(z))!
    #     Look at LF(z): LF(z) lies in F_c where c = L[z].
    #     - If LF(z) SURVIVES, then LF_S(z) = LF(z), and u_S(LF(z)) = P[c] at offset occ_c!
    #     - If LF(z) is DELETED (meaning L[LF(z)] < c), then LF_S(z) = LF_S(LF(z))!
    #   SO DEFINE for EVERY row x in F_c (both surviving AND deleted!):
    #     back_sym(x) = u_ext(x)       if x SURVIVES (not is_del(L[x], c))
    #                 = back_sym(LF(x)) if x is DELETED (is_del(L[x], c))!!
    #   CHECK THAT RECURRENCE!!
    #   For a deleted row x in F_c: L[x] = d with is_del(d, c) => d precedes c in TOPOLOGICAL order!!
    #   So LF(x) lies in F_d, where back_sym(F_d) = Q[d] is ALREADY COMPUTED!!
    #   And for a surviving row x in F_c: back_sym(x) = P[c] at x (ALREADY COMPUTED in Sweep 1)!!
    #   AND FINALLY, for every SURVIVING row z in F_b with L[z] = c:
    #     L_S[z] = back_sym(LF(z)) = Q[c] at the LF-offset of z inside F_c!!
    #   NO LF-CHASES! PURE TOPOLOGICAL INTERVAL SLICING!
    Q = [None] * s
    Q_starts = [None] * s
    bwd_pieces_total = 0
    # Track cumulative offset of L=c across all buckets (gives exact LF-offset in F_c!)
    occ_in_F = [0] * s
    # Wait: we need occ_in_F for each run (c, w, off_b) in br[b]!
    # Precompute lf_off for every entry in br[b]
    br_with_lf = [[] for _ in range(s)]
    for b in range(s):
        for c, w, off_b in br[b]:
            br_with_lf[b].append((c, w, off_b, occ_in_F[c]))
            occ_in_F[c] += w

    for c in bwd_order:
        if cnt[c] == 0:
            Q[c] = []
            Q_starts[c] = [0]
            continue
        segs = []
        pc = P[c]
        pcs = P_starts[c]
        for l_sym, w, off_c, lf_off in br_with_lf[c]:
            if not is_del(l_sym, c):
                # These w rows in F_c SURVIVE -> back_sym = P[c] sliced on [off_c, off_c + w)
                idx = bisect.bisect_right(pcs, off_c) - 1
                cur = off_c
                end = off_c + w
                while cur < end:
                    wid, _ = pc[idx]
                    tk = min(end, pcs[idx + 1]) - cur
                    if segs and segs[-1][0] == wid:
                        segs[-1] = (wid, segs[-1][1] + tk)
                    else:
                        segs.append((wid, tk))
                    bwd_pieces_total += 1
                    cur += tk
                    idx += 1
            else:
                # These w rows in F_c are DELETED -> LF maps them to [lf_off, lf_off + w) in F_{l_sym}!
                qd = Q[l_sym]
                qds = Q_starts[l_sym]
                idx = bisect.bisect_right(qds, lf_off) - 1
                cur = lf_off
                end = lf_off + w
                while cur < end:
                    wid, _ = qd[idx]
                    tk = min(end, qds[idx + 1]) - cur
                    if segs and segs[-1][0] == wid:
                        segs[-1] = (wid, segs[-1][1] + tk)
                    else:
                        segs.append((wid, tk))
                    bwd_pieces_total += 1
                    cur += tk
                    idx += 1
        Q[c] = segs
        st = [0]
        acc = 0
        for _, sw in segs:
            acc += sw
            st.append(acc)
        Q_starts[c] = st

    # --- SWEEP 3: Emit contracted runs L_S on surviving rows only! ---
    out_runs = []
    emit_pieces = 0
    m = 0
    for b in range(s):
        for l_sym, w, off_b, lf_off in br_with_lf[b]:
            if is_del(l_sym, b):
                continue
            m += w
            # L_S on this surviving run is Q[l_sym] sliced at [lf_off, lf_off + w)!
            qc = Q[l_sym]
            qcs = Q_starts[l_sym]
            idx = bisect.bisect_right(qcs, lf_off) - 1
            cur = lf_off
            end = lf_off + w
            while cur < end:
                wid, _ = qc[idx]
                tk = min(end, qcs[idx + 1]) - cur
                if out_runs and out_runs[-1][0] == wid:
                    out_runs[-1] = (wid, out_runs[-1][1] + tk)
                else:
                    out_runs.append((wid, tk))
                emit_pieces += 1
                cur += tk
                idx += 1

    # Remap surviving wid's in exact lexicographic order of P[a] across surviving rows!
    # Wait: the surviving rows of F_b are a subset of F_b, so the order of buckets in S
    # is simply the order in which distinct wid's appear in P[0], P[1], ..., P[s-1] (or restricted to surviving)!
    # Let's assign canonical sorted IDs 0..new_s-1 by first appearance in surviving rows of F_0..F_{s-1}
    wid_order = {}
    new_s = 0
    for b in range(s):
        pb = P[b]
        pbs = P_starts[b]
        for l_sym, w, off_b, _ in br_with_lf[b]:
            if is_del(l_sym, b):
                continue
            idx = bisect.bisect_right(pbs, off_b) - 1
            cur = off_b
            end = off_b + w
            while cur < end:
                wid, _ = pb[idx]
                if wid not in wid_order:
                    wid_order[wid] = new_s
                    new_s += 1
                cur = min(end, pbs[idx + 1])
                idx += 1

    # Remap out_runs to canonical 0..new_s-1
    canon_runs = []
    for wid, w in out_runs:
        cid = wid_order[wid]
        if canon_runs and canon_runs[-1][0] == cid:
            canon_runs[-1] = (cid, canon_runs[-1][1] + w)
        else:
            canon_runs.append((cid, w))

    p_sum = sum(len(x) for x in P if x)
    q_sum = sum(len(x) for x in Q if x)
    stats = {
        "n": n, "m": m, "del_pct": 100.0 * (n - m) / n,
        "r0": len(runs), "r_S": len(canon_runs), "r_ratio": len(canon_runs) / len(runs),
        "new_s": new_s, "P_segs": p_sum, "Q_segs": q_sum,
        "fwd_pieces": fwd_pieces_total, "bwd_pieces": bwd_pieces_total, "emit_pieces": emit_pieces,
        "total_pieces_per_r0": (fwd_pieces_total + bwd_pieces_total + emit_pieces) / len(runs),
    }
    return canon_runs, m, new_s, stats


def verify_against_row_level():
    print("=== 1. EXACT PARITY CHECK OF 2-SWEEP TOPOLOGICAL RUN-SPACE DP vs ROW-LEVEL FIRST-RETURN ===")
    import sys
    sys.path.insert(0, "/tmp/fcp-ref")
    import p2_compositional_sat as p2
    checked = 0
    for n in range(3, 8):
        for k in ([2, 3] if n <= 6 else [2]):
            for w in p2.gen_lyndon(n, k):
                if len(set(w)) < 2:
                    continue
                L, F, LF, FL, C, p, s = p2.bwt_of_word(w)
                runs = []
                for c in L:
                    if runs and runs[-1][0] == c:
                        runs[-1] = (c, runs[-1][1] + 1)
                    else:
                        runs.append((c, 1))
                for mode, pred in [("asc", lambda l, f: l < f), ("desc", lambda l, f: l > f)]:
                    S_mask = [not pred(L[i], F[i]) for i in range(n)]
                    ok, qb, nw, splits, r_new, _, L_S_true = p2.subset_contraction(L, F, LF, FL, p, S_mask)
                    assert ok and splits == 0
                    c_runs, m, new_s, _ = run_space_ascent_dag_step(runs, n, s, mode=mode)
                    L_S_dp = []
                    for c, wt in c_runs:
                        L_S_dp.extend([c] * wt)
                    assert L_S_dp == L_S_true, f"MISMATCH on w={w} mode={mode}: dp={L_S_dp} vs true={L_S_true}"
                    checked += 1
    print(f"  [VERIFIED] {checked} exact parity tests (n=3..7, Ascent + Descent): 100.0% BIT-EXACT MATCH!")


def run_wikidata_benchmark():
    print("\n=== 2. MEASURING 2-SWEEP TOPOLOGICAL DAG-DP ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    mpath = "/tmp/mibz2-blocks/manifest.json"
    manifest_txt = open(mpath).read()
    block_files = sorted(f for f in os.listdir("/tmp/mibz2-blocks") if f.endswith(".L"))

    for fname in block_files:
        fpath = os.path.join("/tmp/mibz2-blocks", fname)
        data, n, runs = load_block(fpath)
        sha_ok = hashlib.sha256(data).hexdigest() in manifest_txt
        t0 = time.time()
        c_runs_a, m_a, s_a, st_a = run_space_ascent_dag_step(runs, n, 256, mode="asc")
        dt_a = time.time() - t0
        print(f"  {fname:12s} sha={sha_ok} | r0={st_a['r0']:6d} -> r_S={st_a['r_S']:6d} ({st_a['r_ratio']:.3f}x) | "
              f"del={st_a['del_pct']:5.2f}% | s'={s_a:5d} | "
              f"P={st_a['P_segs']:6d} Q={st_a['Q_segs']:6d} | "
              f"pieces(F/B/E)=({st_a['fwd_pieces']}/{st_a['bwd_pieces']}/{st_a['emit_pieces']}) "
              f"= {st_a['total_pieces_per_r0']:.2f} r0 ({dt_a:.2f}s)")

    # Multi-round alternating Ascent/Descent DAG contraction on block-0.L (with self-run when needed)!
    print("\n=== 3. FULL MULTI-ROUND ALTERNATING ASCENT/DESCENT DAG CONTRACTION ON block-0.L & block-143.L ===")
    for fname in ["block-0.L", "block-143.L"]:
        data, n, runs = load_block(os.path.join("/tmp/mibz2-blocks", fname))
        r0 = len(runs)
        cur_runs, cur_n, cur_s = runs, n, 256
        tot_r = 0
        tot_pieces = 0
        tot_P = 0
        tot_Q = 0
        rnd = 0
        print(f"  --- {fname} (n0={n}, r0={r0}) ---")
        while cur_n > 4096 and rnd < 20:
            mode = "asc" if (rnd % 2 == 0) else "desc"
            cur_runs, cur_n, cur_s, st = run_space_ascent_dag_step(cur_runs, cur_n, cur_s, mode=mode)
            tot_r += st["r0"]
            tot_pieces += st["fwd_pieces"] + st["bwd_pieces"] + st["emit_pieces"]
            tot_P += st["P_segs"]
            tot_Q += st["Q_segs"]
            if rnd < 8 or cur_n <= 4096:
                print(f"    rnd {rnd:2d} ({mode}): n={st['n']:6d}->{cur_n:6d} ({st['del_pct']:5.2f}% del), "
                      f"r={st['r0']:6d}->{st['r_S']:6d} ({st['r_S']/r0:.3f} r0), s'={cur_s:5d}, "
                      f"pieces={st['fwd_pieces']+st['bwd_pieces']+st['emit_pieces']:7d} ({(st['fwd_pieces']+st['bwd_pieces']+st['emit_pieces'])/st['r0']:.2f} r_t)")
            rnd += 1
            if st["del_pct"] < 1.0:
                print("    (stalled on self-runs)")
                break
        print(f"  SUMMARY {fname} to T<=4096: {rnd} rounds | sum(r_t)/r0 = {tot_r/r0:.3f}x | total_pieces/r0 = {tot_pieces/r0:.3f}x | final n={cur_n}, r={len(cur_runs)}")


if __name__ == "__main__":
    t0 = time.time()
    verify_against_row_level()
    run_wikidata_benchmark()
    print(f"\nDone in {time.time()-t0:.2f}s")
