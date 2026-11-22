#!/tmp/solver-venv/bin/python
"""
FCP Cycle 1 — Part 2: Exact Compositional Contraction Discovery (Arbitrary Subset Theorem,
Bigram-DAG Zero-Bucket-Split Law, 2-Step BDMR Locality, & Wikidata Block Measurements).
"""
import hashlib
import json
import os
import random
import sys
import time
from collections import Counter, defaultdict

from ortools.sat.python import cp_model

sys.path.insert(0, "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-neon-teamwork-paradigm/beta")
import deep_analysis as db


def bwt_of_word(T):
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    p = rots.index(0)
    L_raw = [T[(i - 1) % n] for i in rots]
    uniq = sorted(set(L_raw))
    mp = {c: idx for idx, c in enumerate(uniq)}
    L = [mp[c] for c in L_raw]
    F = sorted(L)
    s = len(uniq)
    C = [0] * (s + 1)
    for c in L:
        C[c + 1] += 1
    for i in range(s):
        C[i + 1] += C[i]
    occ = C[:-1].copy()
    LF = [0] * n
    for i, c in enumerate(L):
        LF[i] = occ[c]
        occ[c] += 1
    FL = [0] * n
    for i, j in enumerate(LF):
        FL[j] = i
    return L, F, LF, FL, C, p, s


def is_primitive(T):
    n = len(T)
    for d in range(1, n):
        if n % d == 0 and T == T[:d] * (n // d):
            return False
    return True


def gen_lyndon(n, k):
    w = [-1]
    while w:
        w[-1] += 1
        m = len(w)
        if m == n:
            yield tuple(w)
        while len(w) < n:
            w.append(w[-m])
        while w and w[-1] == k - 1:
            w.pop()


def subset_contraction(L, F, LF, FL, p, S_mask):
    """
    Exact first-return contraction on arbitrary nonempty subset S = {i : S_mask[i]}.
    Returns:
      - ok_zero_reorder: bool (exact BWT match with zero row reordering on maximal contiguous u_S buckets)
      - q_buckets: number of contiguous buckets (new alphabet size)
      - n_words: number of distinct emitted words u_S
      - bucket_splits: q_buckets - n_words (0 iff ZERO bucket splits!)
      - r_new: number of L-runs in contracted BWT L_S
      - max_chain: max deleted chain length (k - 1)
      - L_S: contracted BWT column
    """
    n = len(L)
    S = [i for i in range(n) if S_mask[i]]
    m = len(S)
    assert m >= 1
    pos_in_S = {row: idx for idx, row in enumerate(S)}

    # For each y in S, compute forward emitted word u_S(y) and FL_S(y)
    u_S = []
    FL_S = [0] * m
    LF_S = [0] * m
    max_chain = 0
    for idx, y in enumerate(S):
        word = [F[y]]
        cur = FL[y]
        steps = 1
        while not S_mask[cur]:
            word.append(F[cur])
            cur = FL[cur]
            steps += 1
        max_chain = max(max_chain, steps - 1)
        u_S.append(tuple(word))
        nxt_idx = pos_in_S[cur]
        FL_S[idx] = nxt_idx
        LF_S[nxt_idx] = idx

    # Partition S into maximal contiguous intervals of constant u_S
    bucket_id = [0] * m
    b_words = [u_S[0]]
    cur_b = 0
    for idx in range(1, m):
        if u_S[idx] != u_S[idx - 1]:
            cur_b += 1
            b_words.append(u_S[idx])
        bucket_id[idx] = cur_b
    q_buckets = cur_b + 1
    n_words = len(set(u_S))
    bucket_splits = q_buckets - n_words

    # Define L_S[i] = bucket_id[LF_S[i]]
    L_S = [bucket_id[LF_S[i]] for i in range(m)]
    # Check: is (L_S, 0..q_buckets-1) the EXACT circular BWT of the rewritten text T_S?
    # Reconstruct T_S by following FL_S from some start (say index 0)
    T_S = []
    cur = 0
    for _ in range(m):
        T_S.append(bucket_id[cur])
        cur = FL_S[cur]
    assert cur == 0, "LF_S is not a single cycle!"
    # Compute true BWT of T_S independently!
    T_S_tup = tuple(T_S)
    rots = sorted(range(m), key=lambda i: T_S_tup[i:] + T_S_tup[:i])
    L_true = [T_S_tup[(i - 1) % m] for i in rots]
    ok_zero_reorder = (L_S == L_true)
    # Also check fiber monotonicity directly
    mono_ok = True
    last_seen = {}
    for idx in range(m):
        w = u_S[idx]
        if w in last_seen and FL_S[idx] <= last_seen[w]:
            mono_ok = False
        last_seen[w] = FL_S[idx]

    r_new = 1 + sum(1 for i in range(1, m) if L_S[i] != L_S[i - 1])
    return ok_zero_reorder and mono_ok, q_buckets, n_words, bucket_splits, r_new, max_chain, L_S


def test_theorem_A():
    print("=== 1. VERIFYING UNIVERSAL ZERO-ROW-REORDERING THEOREM (ARBITRARY SUBSETS S) ===")
    rng = random.Random(42)
    tested = 0
    for n in range(3, 8):
        for k in ([2, 3] if n <= 6 else [2]):
            for w in gen_lyndon(n, k):
                if len(set(w)) < 2:
                    continue
                L, F, LF, FL, C, p, s = bwt_of_word(w)
                # Test ALL nonempty subsets for n<=6, random 30 subsets for n=7
                if n <= 6:
                    masks = range(1, 1 << n)
                else:
                    masks = [rng.randint(1, (1 << n) - 1) for _ in range(30)]
                for mask in masks:
                    S_mask = [bool((mask >> i) & 1) for i in range(n)]
                    ok, _, _, _, _, _, _ = subset_contraction(L, F, LF, FL, p, S_mask)
                    assert ok, f"FAILED on w={w}, mask={mask}"
                    tested += 1
    print(f"  [VERIFIED] {tested} (valid BWT, arbitrary nonempty subset S) pairs: 100.0% EXACT zero-row-reordering!")


def is_interval_complement(out_set, s):
    """Check if Sigma \\ out_set is a contiguous interval (or empty) in 0..s-1."""
    comp = sorted(c for c in range(s) if c not in out_set)
    if len(comp) <= 1:
        return True
    return comp[-1] - comp[0] + 1 == len(comp)


def is_dag(edges, s):
    """Check if directed graph (0..s-1, edges) is acyclic."""
    adj = defaultdict(list)
    indeg = [0] * s
    for a, b in edges:
        adj[a].append(b)
        indeg[b] += 1
    q = [v for v in range(s) if indeg[v] == 0]
    vis = 0
    while q:
        u = q.pop()
        vis += 1
        for v in adj[u]:
            indeg[v] -= 1
            if indeg[v] == 0:
                q.append(v)
    return vis == s


def test_bigram_dag_zero_split():
    print("\n=== 2. EXACT CHARACTERIZATION OF ZERO-BUCKET-SPLIT BIGRAM-DAG CONTRACTIONS ===")
    # Test ALL acyclic subsets E_del of Sigma x Sigma for s = 3 (2^6 = 64? No, 3*3=9 pairs, no self-loops -> 6 pairs -> 25 DAGs!)
    # Wait: include self-loops? A self-loop (a, a) is cyclic, so NOT a DAG!
    # Wait: what about s=3 (all 25 DAGs) and s=4 (all 543 DAGs)?
    for s in [3, 4]:
        non_diag = [(a, b) for a in range(s) for b in range(s) if a != b]
        dags = []
        if s == 3:
            for mask in range(1, 1 << len(non_diag)):
                edges = [non_diag[i] for i in range(len(non_diag)) if (mask >> i) & 1]
                if is_dag(edges, s):
                    dags.append(tuple(edges))
        else:
            # All 543 DAGs on 4 vertices (2^12 = 4096 masks)
            for mask in range(1, 1 << len(non_diag)):
                edges = [non_diag[i] for i in range(len(non_diag)) if (mask >> i) & 1]
                if is_dag(edges, s):
                    dags.append(tuple(edges))

        # Build test suite of valid BWTs with alphabet size s
        bwt_suite = []
        max_n = 7 if s == 3 else 6
        for n in range(s, max_n + 1):
            for w in gen_lyndon(n, s):
                if len(set(w)) == s:
                    bwt_suite.append((w, bwt_of_word(w)))
        rng = random.Random(1234 + s)
        for n in [8, 9, 10, 12]:
            for _ in range(40):
                w = tuple(rng.randrange(s) for _ in range(n))
                if len(set(w)) == s and is_primitive(w):
                    bwt_suite.append((w, bwt_of_word(w)))

        # Check our conjectured condition:
        # Cond C1: for every a in 0..s-1, Sigma \ Out(a) is a contiguous interval!
        # Wait: is there ANY subtlety when a path has length >= 2?! Let's check!
        # Suppose a -> b is in E_del, and we have u1 = a b (stops at b) and u2 = a b c (continues from b).
        # What about u1 = a (stops at a, next letter c1 in Sigma \ Out(a)) vs u2 = a b (next letter c2)?
        # Wait: if b in Out(a), then b is OUTSIDE the interval Sigma \ Out(a), so either b < all(Sigma \ Out(a)) or b > all(Sigma \ Out(a))!
        # So EVERY word starting with ab is on ONE side of a, and EVERY word starting with a b' (b' != b) is separated at position 1!
        # So Cond C1 should be SUFFICIENT! Is it NECESSARY? Let's test EVERY SINGLE DAG!
        match_c1 = 0
        c1_suff_violations = []
        c1_nec_violations = []
        ascent_stats = []
        for edges in dags:
            out_map = defaultdict(set)
            for a, b in edges:
                out_map[a].add(b)
            c1 = all(is_interval_complement(out_map[a], s) for a in range(s))
            edge_set = set(edges)

            # Test against bwt_suite
            has_split = False
            split_wit = None
            max_r_ratio = 0.0
            r_inc_wit = None
            for w, (L, F, LF, FL, C, p, sig) in bwt_suite:
                n = len(L)
                S_mask = [(L[i], F[i]) not in edge_set for i in range(n)]
                if not any(S_mask):
                    continue
                ok, qb, nw, splits, r_new, max_ch, _ = subset_contraction(L, F, LF, FL, p, S_mask)
                assert ok
                r0 = 1 + sum(1 for i in range(1, n) if L[i] != L[i - 1])
                if r_new / r0 > max_r_ratio:
                    max_r_ratio = r_new / r0
                    if r_new > r0 and r_inc_wit is None:
                        r_inc_wit = (w, r0, r_new, n, sum(S_mask))
                if splits > 0:
                    has_split = True
                    split_wit = (w, qb, nw, splits)
                    break

            if c1 and not has_split:
                match_c1 += 1
            elif (not c1) and has_split:
                match_c1 += 1
            elif c1 and has_split:
                c1_suff_violations.append((edges, split_wit))
            else:
                c1_nec_violations.append(edges)

            # Track canonical Ascent DAG {(a,b): a < b}
            if set(edges) == {(a, b) for a in range(s) for b in range(s) if a < b}:
                ascent_stats.append((s, has_split, max_r_ratio, r_inc_wit))

        print(f"  s={s}: {len(dags)} non-empty DAGs tested on {len(bwt_suite)} valid BWTs:")
        print(f"    Cond C1 (forall a, Sigma\\Out(a) contiguous interval) matches {match_c1}/{len(dags)}!")
        print(f"    Sufficiency violations: {len(c1_suff_violations)} | Necessity violations: {len(c1_nec_violations)}")
        if c1_nec_violations:
            print(f"    (Sample nec violation: {c1_nec_violations[:3]})")
        if ascent_stats:
            print(f"    Ascent-DAG (a<b) s={s}: has_split={ascent_stats[0][1]}, max r_new/r0={ascent_stats[0][2]:.4f}, r_inc_wit={ascent_stats[0][3]}")


def test_ascent_dag_run_monotonicity_and_2step():
    print("\n=== 3. ANALYZING ASCENT-DAG (a<b), DESCENT-DAG (a>b), AND 2-STEP BDMR COMPOSITIONS ===")
    # Check if Ascent-DAG (a<b) ALWAYS preserves r_new <= r0, or find minimal counterexample!
    for name, pred in [("Ascent (a<b)", lambda a, b: a < b), ("Descent (a>b)", lambda a, b: a > b)]:
        r_inc = []
        max_ratio = 0.0
        total = 0
        for n in range(3, 9):
            for k in ([2, 3] if n <= 7 else [2]):
                for w in gen_lyndon(n, k):
                    if len(set(w)) < 2:
                        continue
                    L, F, LF, FL, C, p, s = bwt_of_word(w)
                    r0 = 1 + sum(1 for i in range(1, n) if L[i] != L[i - 1])
                    S_mask = [not pred(L[i], F[i]) for i in range(n)]
                    ok, qb, nw, splits, r_new, max_ch, _ = subset_contraction(L, F, LF, FL, p, S_mask)
                    assert ok and splits == 0
                    total += 1
                    if r_new > r0:
                        r_inc.append((n, s, w, r0, r_new, n - sum(S_mask), max_ch))
                    max_ratio = max(max_ratio, r_new / r0)
        print(f"  {name:15s} over {total} BWTs: zero splits=100%, r_inc count={len(r_inc)}, max r_new/r0={max_ratio:.4f}")
        if r_inc:
            r_inc.sort(key=lambda x: (x[0], x[1]))
            print(f"    Minimal r_new > r0 witness: n={r_inc[0][0]}, s={r_inc[0][1]}, T={r_inc[0][2]}, r:{r_inc[0][3]}->{r_inc[0][4]}, del={r_inc[0][5]}, max_chain={r_inc[0][6]}")

    # Check 2-step BDMR compositions C_2 o C_1 on valid BWTs n=5..9:
    # How often is D_12 = D_1 U D_2 local to (L[i], F[i])? What is max chain length?
    print("\n  Checking 2-step BDMR compositions C_2 o C_1 (pair -> pair) on valid BWTs n=5..8:")
    rng = random.Random(77)
    total_2step = 0
    local_LF_cnt = 0
    zero_split_cnt = 0
    chain_hist = Counter()
    shrink_sum = 0.0
    for n in [5, 6, 7, 8]:
        ws = [w for w in gen_lyndon(n, 3) if len(set(w)) == 3]
        sample = ws if len(ws) <= 40 else rng.sample(ws, 40)
        for w in sample:
            L, F, LF, FL, C, p, s = bwt_of_word(w)
            # Pick a pair step C1 (max-cut 2-coloring)
            best_c1 = None
            for mask1 in range(1, (1 << s) - 1):
                inL1 = [bool((mask1 >> c) & 1) for c in range(s)]
                D1 = [inL1[L[i]] and (not inL1[F[i]]) for i in range(n)]
                dc = sum(D1)
                if best_c1 is None or dc > best_c1[0]:
                    best_c1 = (dc, inL1, D1)
            dc1, inL1, D1 = best_c1
            if dc1 == 0 or dc1 >= n - 2:
                continue
            S1_mask = [not d for d in D1]
            ok1, q1, nw1, sp1, r1, _, L1 = subset_contraction(L, F, LF, FL, p, S1_mask)
            if q1 < 2 or len(L1) < 3:
                continue
            # Now pick max-cut pair step C2 on L1!
            n1 = len(L1)
            F1 = sorted(L1)
            # Sample up to 15 colorings on q1
            best_c2 = None
            if q1 <= 6:
                c2_masks = range(1, (1 << q1) - 1)
            else:
                c2_masks = [rng.randint(1, (1 << q1) - 2) for _ in range(30)]
            for mask2 in c2_masks:
                inL2 = [bool((mask2 >> c) & 1) for c in range(q1)]
                D2 = [inL2[L1[i]] and (not inL2[F1[i]]) for i in range(n1)]
                dc2 = sum(D2)
                if best_c2 is None or dc2 > best_c2[0]:
                    best_c2 = (dc2, inL2, D2)
            dc2, inL2, D2 = best_c2
            if dc2 == 0 or dc2 >= n1 - 1:
                continue
            # Map D2 back to original indices of I_0!
            S1_rows = [i for i in range(n) if S1_mask[i]]
            S12_mask = S1_mask.copy()
            for idx1, d in enumerate(D2):
                if d:
                    S12_mask[S1_rows[idx1]] = False
            # Check if D12 = not S12_mask is a function of (L[i], F[i]) alone!
            lf_to_del = defaultdict(set)
            for i in range(n):
                lf_to_del[(L[i], F[i])].add(not S12_mask[i])
            is_local = all(len(v) == 1 for v in lf_to_del.values())
            ok12, q12, nw12, sp12, r12, max_ch, _ = subset_contraction(L, F, LF, FL, p, S12_mask)
            assert ok12
            total_2step += 1
            if is_local:
                local_LF_cnt += 1
            if sp12 == 0:
                zero_split_cnt += 1
            chain_hist[max_ch] += 1
            shrink_sum += (n - sum(S12_mask)) / n

    print(f"    Tested {total_2step} max-cut 2-step compositions C_2 o C_1:")
    print(f"    Avg deleted fraction |D_12|/n = {shrink_sum / max(1, total_2step):.4f}")
    print(f"    D_12 purely local to (L, F) in I_0: {local_LF_cnt}/{total_2step} ({100*local_LF_cnt/max(1,total_2step):.1f}%)")
    print(f"    Zero bucket splits on u_S12:        {zero_split_cnt}/{total_2step} ({100*zero_split_cnt/max(1,total_2step):.1f}%)")
    print(f"    Max deleted chain length histogram: {dict(sorted(chain_hist.items()))}")


def measure_wikidata_ascent_dag():
    print("\n=== 4. MEASURING 1-SHOT ASCENT-DAG (a<b) & DESCENT-DAG (a>b) ON 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    mpath = "/tmp/mibz2-blocks/manifest.json"
    with open(mpath) as fp:
        manifest = json.load(fp)
    # Check manifest structure
    entries = manifest if isinstance(manifest, list) else manifest.get("blocks", list(manifest.values()))
    if isinstance(manifest, dict) and "blocks" not in manifest:
        # Handle dict mapping filename -> info or similar
        items = [(k, v) for k, v in manifest.items() if isinstance(v, dict)]
    else:
        items = [(e.get("file", e.get("name", f"blk_{i}")), e) for i, e in enumerate(entries)]

    # Verify SHA-256 of all 6 blocks
    block_files = sorted(f for f in os.listdir("/tmp/mibz2-blocks") if f.endswith(".L"))
    print(f"  Manifest keys: {list(manifest.keys())[:6] if isinstance(manifest, dict) else 'list'}")
    for fname in block_files:
        fpath = os.path.join("/tmp/mibz2-blocks", fname)
        data = open(fpath, "rb").read()
        sha = hashlib.sha256(data).hexdigest()
        # Verify against manifest
        matched = sha in json.dumps(manifest)
        n = len(data)
        # Build F and runs directly (fast, O(n) in Python using bytearray/counts)
        cnt = [0] * 256
        for b in data:
            cnt[b] += 1
        C = [0] * 257
        for c in range(256):
            C[c + 1] = C[c] + cnt[c]
        # Compute runs of L and (L,F)-runs
        r0 = 1
        prev = data[0]
        for b in data[1:]:
            if b != prev:
                r0 += 1
                prev = b

        # Evaluate 1-shot Ascent-DAG (L < F), Descent-DAG (L > F), and optimal 2-coloring proxy (byte < 128)
        # Count |D| directly from (L, F) in O(n) without building LF!
        del_asc = 0
        del_desc = 0
        del_self = 0
        # Also compute exact 1-shot surviving runs r_S ONLY on block-0 or via fast LF?
        # Wait: 900k LF in Python takes ~0.3s! Let's compute exact (del, max_chain, q_words, r_S) on ALL 6 blocks!
        # Wait: let's do fast O(n) LF on block-0 and block-2 first (or all 6 if <1s each)!
        for c in range(256):
            lo, hi = C[c], C[c + 1]
            if lo == hi:
                continue
            seg = data[lo:hi]
            for b in seg:
                if b < c:
                    del_asc += 1
                elif b > c:
                    del_desc += 1
                else:
                    del_self += 1
        print(f"  {fname:14s} sha_ok={matched} n={n:6d} r0={r0:6d} | Asc(L<F): del={100*del_asc/n:5.2f}% | Desc(L>F): del={100*del_desc/n:5.2f}% | Self(L=F): {100*del_self/n:5.2f}%")

    # Detailed 1-shot Ascent-DAG & Descent-DAG structural stats on block-0.L (and block-143.L)
    for fname in ["block-0.L", "block-143.L"]:
        data = open(os.path.join("/tmp/mibz2-blocks", fname), "rb").read()
        n = len(data)
        cnt = [0] * 256
        for b in data:
            cnt[b] += 1
        C = [0] * 257
        for c in range(256):
            C[c + 1] = C[c] + cnt[c]
        F = bytearray(n)
        for c in range(256):
            F[C[c]:C[c + 1]] = bytes([c]) * cnt[c]
        occ = C[:256].copy()
        LF = [0] * n
        for i, b in enumerate(data):
            LF[i] = occ[b]
            occ[b] += 1
        FL = [0] * n
        for i, j in enumerate(LF):
            FL[j] = i

        r0 = 1 + sum(1 for i in range(1, n) if data[i] != data[i - 1])
        for mode_name, is_del_fn in [("Ascent(L<F)", lambda l, f: l < f), ("Descent(L>F)", lambda l, f: l > f)]:
            S_mask = [not is_del_fn(data[i], F[i]) for i in range(n)]
            S = [i for i in range(n) if S_mask[i]]
            m = len(S)
            # Compute emitted words u_S(y) and verify zero bucket splits + compute r_S and chain length distribution!
            chain_lens = Counter()
            word_to_id = {}
            bucket_id = [0] * n  # indexed by original row in S
            b_cnt = 0
            prev_w = None
            n_words_seen = set()
            for y in S:
                w = [F[y]]
                cur = FL[y]
                while not S_mask[cur]:
                    w.append(F[cur])
                    cur = FL[cur]
                wt = tuple(w)
                chain_lens[len(wt)] += 1
                if wt != prev_w:
                    b_cnt += 1
                    prev_w = wt
                n_words_seen.add(wt)
                bucket_id[y] = b_cnt - 1
            # Compute L_S[idx] = bucket_id[LF_S[y]]
            r_S = 0
            prev_ls = -1
            for y in S:
                cur = LF[y]
                while not S_mask[cur]:
                    cur = LF[cur]
                ls = bucket_id[cur]
                if ls != prev_ls:
                    r_S += 1
                    prev_ls = ls
            splits = b_cnt - len(n_words_seen)
            top_lens = sorted(chain_lens.items())[:8]
            max_len = max(chain_lens.keys())
            avg_len = sum(k * v for k, v in chain_lens.items()) / m
            print(f"  [DETAIL {fname} {mode_name}] n={n}->m={m} ({100*(n-m)/n:.2f}% del), r0={r0}->r_S={r_S} ({r_S/r0:.4f}x), "
                  f"uniq_words={len(n_words_seen)}, buckets={b_cnt}, splits={splits}, max_word_len={max_len}, avg_len={avg_len:.2f}, len_hist[:8]={top_lens}")


if __name__ == "__main__":
    t0 = time.time()
    test_theorem_A()
    test_bigram_dag_zero_split()
    test_ascent_dag_run_monotonicity_and_2step()
    measure_wikidata_ascent_dag()
    print(f"\nDone in {time.time()-t0:.2f}s")
