#!/usr/bin/env python3
"""
Exploratory mathematical laboratory for Compositional BWT-Space Contraction:
  Angle 1: Bucket-Respecting Piecewise-Translation (IET) Schur Elimination in PT_n
  Tests:
    1. Exactness & Universal Piece-Monotonicity on exhaustive small cases
       (binary n<=8, ternary n<=5, all primary marks p, plus multi-cycle L)
    2. Adversarial families: Symmetric-Middle-Run, Fib_{2m}+$, Phi_BWT
    3. All 6 authenticated Wikidata blocks in /tmp/mibz2-blocks/
"""
import hashlib
import json
import os
import sys
from collections import defaultdict

BLOCKS_DIR = "/tmp/mibz2-blocks"


def bwt_encode(T):
    n = len(T)
    order = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = [T[(i - 1) % n] for i in order]
    p = order.index(0)
    return L, p


def decode_lf_reference(L, p):
    """Exact reference inverse BWT via LF walk (handles single-cycle and reports cycle len)."""
    n = len(L)
    counts = defaultdict(int)
    for c in L:
        counts[c] += 1
    alpha = sorted(counts.keys())
    C = {}
    acc = 0
    for c in alpha:
        C[c] = acc
        acc += counts[c]
    occ = defaultdict(int)
    LF = [0] * n
    for j, c in enumerate(L):
        LF[j] = C[c] + occ[c]
        occ[c] += 1
    out = []
    cur = p
    for _ in range(n):
        out.append(L[cur])
        cur = LF[cur]
        if cur == p:
            break
    out.reverse()
    return out, len(out), LF, C, counts


def build_initial_iet(L):
    """
    Build the bucket-respecting piecewise-translation bijection f0 = LF from L (or runs).
    Each piece is a tuple: (src_start, length, dst_start, src_bucket, dst_bucket, word_id)
    where word_id references an SLP table:
      slp[word_id] = ('lit', byte) or ('cat', left_id, right_id) or ('rep', base_id, k)
    Note: A piece represents mapping x -> x - src_start + dst_start for x in [src_start, src_start + length),
    and in backward LF direction, stepping from x to LF(x) emits the symbol L[x] = dst_bucket!
    Wait: let's track forward-text string emitted along the LF step!
    Along LF: x -> LF(x) moves BACKWARD in text (from T[i] to T[i-1]), emitting L[x] = T[i-1].
    If we compose step1 (x -> y, emits w1) followed by step2 (y -> z, emits w2),
    in forward text order, w2 precedes w1! So the combined forward-text word is cat(w2, w1).
    Let's verify:
      start at p: F[p] = T[0].
      step 1: p -> LF(p) emits L[p] = T[n-1].
      step 2: LF(p) -> LF^2(p) emits L[LF(p)] = T[n-2].
      Combined forward-text word for p -> LF^2(p) is T[n-2] T[n-1] = cat(w2, w1).
    YES!
    """
    n = len(L)
    # First compress to maximal runs (simulating I0)
    runs = []
    if n > 0:
        cur_c = L[0]
        cur_l = 1
        for i in range(1, n):
            if L[i] == cur_c:
                cur_l += 1
            else:
                runs.append((cur_c, cur_l))
                cur_c = L[i]
                cur_l = 1
        runs.append((cur_c, cur_l))
    return build_initial_iet_from_runs(runs, n)


def build_initial_iet_from_runs(runs, n):
    counts = defaultdict(int)
    for c, l in runs:
        counts[c] += l
    alpha = sorted(counts.keys())
    C = {}
    acc = 0
    for c in alpha:
        C[c] = acc
        acc += counts[c]
    bucket_end = {c: C[c] + counts[c] for c in alpha}

    slp = []
    lit_id = {}
    for c in alpha:
        lit_id[c] = len(slp)
        slp.append(('lit', c))

    # Split runs at F-bucket boundaries so every piece has a single src_bucket and single dst_bucket
    # Store pieces grouped by src_bucket (sorted by src_start) and dst_bucket?
    # Better: store pieces in a dict by_src[b] (sorted by src_start), and maintain by_dst on demand or incrementally.
    pieces_by_src = {c: [] for c in alpha}
    occ = {c: 0 for c in alpha}

    # Pointer into alpha for current F-bucket
    bi = 0
    pos = 0
    for c, l in runs:
        rem = l
        while rem > 0:
            sb = alpha[bi]
            avail = bucket_end[sb] - pos
            take = rem if rem < avail else avail
            dst_start = C[c] + occ[c]
            pieces_by_src[sb].append((pos, take, dst_start, sb, c, lit_id[c]))
            occ[c] += take
            pos += take
            rem -= take
            if pos == bucket_end[sb] and bi + 1 < len(alpha):
                bi += 1

    return alpha, C, counts, pieces_by_src, slp, len(runs)


def is_monotone_pieces(self_pieces):
    """Check if self-loop pieces (sorted by src_start) are strictly monotone increasing in dst_start."""
    prev_dst = -1
    for s, l, d, sb, db, wid in self_pieces:
        if d < prev_dst:
            return False
        prev_dst = d + l
    return True


def eliminate_bucket(e, pieces_by_src, slp, use_spiral=True):
    """
    Compute exact Schur complement Eliminating bucket e from the piecewise-translation map.
    Returns stats dict:
      r_in, r_out, r_self, is_mono, transitions, spiral_jumps, new_pieces_count, delta_r, closed_cycles_in_e
    """
    out_all = pieces_by_src.pop(e)
    self_pieces = []
    exit_pieces = []
    for p in out_all:
        if p[4] == e:
            self_pieces.append(p)
        else:
            exit_pieces.append(p)

    # Collect incoming pieces from other buckets
    in_pieces_by_src = {}
    r_in = 0
    for b, plist in pieces_by_src.items():
        inc = []
        keep = []
        for p in plist:
            if p[4] == e:
                inc.append(p)
            else:
                keep.append(p)
        if inc:
            in_pieces_by_src[b] = inc
            pieces_by_src[b] = keep
            r_in += len(inc)

    r_out = len(exit_pieces)
    r_self = len(self_pieces)
    mono = is_monotone_pieces(self_pieces)

    # Build sorted lookup over all outgoing pieces of e (self + exit), sorted by src_start
    # They partition F_e completely!
    out_sorted = out_all  # already sorted by src_start!
    # Quick binary search helper
    out_starts = [p[0] for p in out_sorted]

    import bisect

    def slp_cat(left_id, right_id):
        idx = len(slp)
        slp.append(('cat', left_id, right_id))
        return idx

    def slp_rep(base_id, k):
        if k == 1:
            return base_id
        idx = len(slp)
        slp.append(('rep', base_id, k))
        return idx

    transitions = 0
    spiral_jumps = 0
    new_pieces_count = 0

    # Active intervals currently inside F_e to be traced to exit:
    # Each active item: (orig_src_start, length, cur_pos_in_e, orig_src_bucket, cur_word_id)
    # Wait: if r_self == 0, we can do a pure 2-pointer merge!
    if r_self == 0:
        # Gather all incoming pieces sorted by dst_start
        all_in = []
        for b, inc in in_pieces_by_src.items():
            all_in.extend(inc)
        all_in.sort(key=lambda p: p[2])

        oi = 0
        o_len = len(out_sorted)
        for s, l, d, sb, db, wid in all_in:
            cur_s = s
            cur_d = d
            rem = l
            while rem > 0:
                while oi < o_len and out_sorted[oi][0] + out_sorted[oi][1] <= cur_d:
                    oi += 1
                os_, ol_, od_, _, odb_, owid_ = out_sorted[oi]
                off = cur_d - os_
                take = min(rem, ol_ - off)
                new_wid = slp_cat(owid_, wid)
                pieces_by_src[sb].append((cur_s, take, od_ + off, sb, odb_, new_wid))
                new_pieces_count += 1
                transitions += 1
                cur_s += take
                cur_d += take
                rem -= take
    else:
        # General self-loop resolution with closed-form single-piece spiral acceleration!
        # Queue of segments (orig_src, length, cur_pos, orig_src_bucket, cur_wid)
        stack = []
        for b, inc in in_pieces_by_src.items():
            for s, l, d, sb, db, wid in inc:
                stack.append((s, l, d, sb, wid))

        while stack:
            orig_s, l, cur_pos, sb, wid = stack.pop()
            # Find out_sorted piece containing cur_pos
            oi = bisect.bisect_right(out_starts, cur_pos) - 1
            os_, ol_, od_, _, odb_, owid_ = out_sorted[oi]
            off = cur_pos - os_
            take = min(l, ol_ - off)
            if take < l:
                stack.append((orig_s + take, l - take, cur_pos + take, sb, wid))

            transitions += 1
            if odb_ != e:
                # Exited e!
                new_wid = slp_cat(owid_, wid)
                pieces_by_src[sb].append((orig_s, take, od_ + off, sb, odb_, new_wid))
                new_pieces_count += 1
            else:
                # Self-loop inside piece oi: maps x -> x + delta on [os_, os_ + ol_)
                delta = od_ - os_
                if delta == 0:
                    # Fixed points! Impossible if entered from outside (by bijection),
                    # since fixed points are their own closed cycles.
                    raise RuntimeError("Bijection violation: entered fixed point from outside")
                if use_spiral:
                    # Closed-form spiral out of [os_, os_ + ol_)!
                    # Let piece interval be [A, B) = [os_, os_ + ol_).
                    A = os_
                    B = os_ + ol_
                    # Current subinterval is [u, v) = [cur_pos, cur_pos + take) subset [A, B).
                    u = cur_pos
                    v = cur_pos + take
                    if delta > 0:
                        # For x in [u, v), k(x) = ceil((B - x) / delta) = (B - 1 - x) // delta + 1
                        k_max = (B - 1 - u) // delta + 1
                        k_min = (B - 1 - (v - 1)) // delta + 1
                        if k_max == k_min:
                            k = k_max
                            rep_w = slp_rep(owid_, k)
                            new_wid = slp_cat(rep_w, wid)
                            stack.append((orig_s, take, u + k * delta, sb, new_wid))
                            if k > 1:
                                spiral_jumps += 1
                        else:
                            # Split at the boundary where k changes:
                            # x has k = k_max iff B - 1 - x >= (k_max - 1)*delta
                            # iff x <= B - 1 - (k_max - 1)*delta
                            split_x = B - (k_max - 1) * delta
                            # [u, split_x) has k = k_max; [split_x, v) has k <= k_max - 1
                            len1 = split_x - u
                            if len1 < take:
                                stack.append((orig_s + len1, take - len1, split_x, sb, wid))
                            rep_w = slp_rep(owid_, k_max)
                            new_wid = slp_cat(rep_w, wid)
                            stack.append((orig_s, len1, u + k_max * delta, sb, new_wid))
                            if k_max > 1:
                                spiral_jumps += 1
                    else:
                        # delta < 0, let step = -delta > 0
                        # For x in [u, v), exits < A when x - k*step < A
                        # k(x) = (x - A) // step + 1
                        step = -delta
                        k_min = (u - A) // step + 1
                        k_max = (v - 1 - A) // step + 1
                        if k_min == k_max:
                            k = k_min
                            rep_w = slp_rep(owid_, k)
                            new_wid = slp_cat(rep_w, wid)
                            stack.append((orig_s, take, u - k * step, sb, new_wid))
                            if k > 1:
                                spiral_jumps += 1
                        else:
                            # x has k = k_min iff x - A < k_min * step
                            # iff x < A + k_min * step
                            split_x = A + k_min * step
                            len1 = split_x - u
                            if len1 < take:
                                stack.append((orig_s + len1, take - len1, split_x, sb, wid))
                            rep_w = slp_rep(owid_, k_min)
                            new_wid = slp_cat(rep_w, wid)
                            stack.append((orig_s, len1, u - k_min * step, sb, new_wid))
                            if k_min > 1:
                                spiral_jumps += 1
                else:
                    new_wid = slp_cat(owid_, wid)
                    stack.append((orig_s, take, od_ + off, sb, new_wid))

    # Re-sort affected buckets by src_start and coalesce adjacent pieces with identical shift & word?
    # Note: we do NOT need to coalesce by word for piece count bound, let's check both raw and shift-coalesced!
    for b in in_pieces_by_src.keys():
        pieces_by_src[b].sort(key=lambda p: p[0])

    old_touch = r_in + r_out + r_self
    delta_r = new_pieces_count - old_touch
    return {
        'e': e,
        'r_in': r_in,
        'r_out': r_out,
        'r_self': r_self,
        'mono': mono,
        'transitions': transitions,
        'spiral_jumps': spiral_jumps,
        'new_pieces': new_pieces_count,
        'delta_r': delta_r,
    }


def expand_slp(slp, wid, out_list):
    stack = [wid]
    while stack:
        w = stack.pop()
        node = slp[w]
        tag = node[0]
        if tag == 'lit':
            out_list.append(node[1])
        elif tag == 'cat':
            stack.append(node[2])
            stack.append(node[1])
        elif tag == 'rep':
            base, k = node[1], node[2]
            # Expand base once and extend k times
            tmp = []
            expand_slp(slp, base, tmp)
            out_list.extend(tmp * k)


def reconstruct_from_iet_and_primary(pieces_by_src, slp, L_orig, p_orig, orig_LF=None):
    """
    Given remaining pieces on surviving buckets S, reconstruct the cycle containing p_orig.
    If p_orig was in an eliminated bucket, we just track where p_orig first lands in S (or simpler:
    keep the bucket containing p_orig, OR walk from p_orig in I0 until hitting S).
    Here we ALWAYS keep the bucket containing p_orig (or test both!).
    """
    import bisect
    # Flatten remaining pieces sorted by src_start
    all_p = []
    for b in sorted(pieces_by_src.keys()):
        all_p.extend(pieces_by_src[b])
    all_p.sort(key=lambda x: x[0])
    starts = [x[0] for x in all_p]

    # Walk LF_S from p_orig (assuming p_orig is in S)
    cur = p_orig
    wids = []
    steps = 0
    while True:
        idx = bisect.bisect_right(starts, cur) - 1
        s, l, d, sb, db, wid = all_p[idx]
        assert s <= cur < s + l
        wids.append(wid)
        cur = d + (cur - s)
        steps += 1
        if cur == p_orig:
            break
    # Forward text order is reverse of LF step order!
    out = []
    for wid in reversed(wids):
        expand_slp(slp, wid, out)
    return out, steps


def run_md_iet_schur(L, p, strategy="min_deg", keep_primary_bucket=True):
    """
    Run Bucket IET Schur Elimination down to 1 bucket (containing p if keep_primary_bucket).
    Returns: reconstructed text, stats
    """
    alpha, C, counts, pieces_by_src, slp, r0 = build_initial_iet(L)
    # Find bucket containing p
    p_bucket = None
    for c in alpha:
        if C[c] <= p < C[c] + counts[c]:
            p_bucket = c
            break

    r_init = sum(len(v) for v in pieces_by_src.values())
    r_cur = r_init
    r_max = r_init
    monotone_violation = False
    history = []

    while len(pieces_by_src) > 1:
        # Compute stats for each candidate bucket
        cands = [b for b in pieces_by_src.keys() if not (keep_primary_bucket and b == p_bucket)]
        if not cands:
            break
        # Count r_in, r_out, r_self for each bucket
        r_in_map = defaultdict(int)
        r_out_map = defaultdict(int)
        r_self_map = defaultdict(int)
        for sb, plist in pieces_by_src.items():
            for piece in plist:
                db = piece[4]
                if sb == db:
                    r_self_map[sb] += 1
                else:
                    r_out_map[sb] += 1
                    r_in_map[db] += 1

        if strategy == "min_deg":
            # Prioritize r_self == 0 first? Or pure min degree?
            best = min(cands, key=lambda b: (r_in_map[b] + r_out_map[b] + 2 * r_self_map[b], counts[b], b))
        elif strategy == "zero_self_first":
            best = min(cands, key=lambda b: (0 if r_self_map[b] == 0 else (1 if r_self_map[b] == 1 else 2),
                                             r_in_map[b] + r_out_map[b] + 2 * r_self_map[b], counts[b], b))
        elif strategy == "min_self_deg":
            best = min(cands, key=lambda b: (r_self_map[b], r_in_map[b] + r_out_map[b], counts[b], b))
        elif strategy == "max_symbol":
            best = max(cands)
        else:
            raise ValueError(strategy)

        st = eliminate_bucket(best, pieces_by_src, slp, use_spiral=True)
        r_next = sum(len(v) for v in pieces_by_src.values())
        if r_next > r_cur:
            monotone_violation = True
        r_cur = r_next
        if r_cur > r_max:
            r_max = r_cur
        st['r_after'] = r_cur
        st['bucket_size'] = counts[best]
        history.append(st)

    out, final_steps = reconstruct_from_iet_and_primary(pieces_by_src, slp, L, p)
    return out, {
        'n': len(L),
        'r0': r0,
        'r_init': r_init,
        'r_max': r_max,
        'r_final': r_cur,
        'monotone_violation': monotone_violation,
        'final_steps': final_steps,
        'total_transitions': sum(h['transitions'] for h in history),
        'total_deg': sum(h['r_in'] + h['r_out'] + 2 * h['r_self'] for h in history),
        'slp_size': len(slp),
        'history': history,
    }


def test_exhaustive():
    print("=== 1. EXHAUSTIVE SMALL VERIFICATION ===")
    total_cases = 0
    violations = 0
    # Binary strings n <= 8, all valid BWTs and all p (actually valid BWT has unique p,
    # let's test ALL binary strings n <= 8 with all rotations/p AND all binary L arrays n <= 7 with all p!)
    for n in range(1, 9):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = bwt_encode(T)
            ref, clen, _, _, _ = decode_lf_reference(L, p)
            out, st = run_md_iet_schur(L, p, strategy="min_deg")
            assert out == ref, f"Mismatch binary n={n}, T={T}: {out} != {ref}"
            if st['monotone_violation']:
                violations += 1
            total_cases += 1

    # Ternary strings n <= 5 (all 3^5 + ... = 363 strings, wait: test all p for each L too!)
    for n in range(1, 6):
        for val in range(3 ** n):
            T = []
            x = val
            for _ in range(n):
                T.append(x % 3)
                x //= 3
            L, p = bwt_encode(T)
            for p_try in range(n):
                ref, clen, _, _, _ = decode_lf_reference(L, p_try)
                for strat in ("min_deg", "zero_self_first", "max_symbol"):
                    out, st = run_md_iet_schur(L, p_try, strategy=strat)
                    assert out == ref, f"Mismatch ternary n={n}, L={L}, p={p_try}, strat={strat}"
                    if st['monotone_violation']:
                        violations += 1
                    total_cases += 1

    # Arbitrary binary L arrays n <= 7 (including multi-cycle!) all p
    multi_cycle = 0
    for n in range(1, 8):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            for p_try in range(n):
                ref, clen, _, _, _ = decode_lf_reference(L, p_try)
                if clen < n:
                    multi_cycle += 1
                out, st = run_md_iet_schur(L, p_try, strategy="min_deg")
                assert out == ref, f"Mismatch arb binary L={L}, p={p_try}"
                if st['monotone_violation']:
                    violations += 1
                total_cases += 1

    # Arbitrary 4-symbol L arrays n = 6 (random 2000 cases, multi-cycle + non-BWT!)
    import random
    rng = random.Random(42)
    for _ in range(2000):
        L = [rng.randrange(4) for _ in range(8)]
        p_try = rng.randrange(8)
        ref, clen, _, _, _ = decode_lf_reference(L, p_try)
        for strat in ("min_deg", "zero_self_first"):
            out, st = run_md_iet_schur(L, p_try, strategy=strat)
            assert out == ref, f"Mismatch arb 4-sym L={L}, p={p_try}"
            if st['monotone_violation']:
                violations += 1
            total_cases += 1

    print(f"PASS! {total_cases} total cases (incl. {multi_cycle} multi-cycle & 4000 arb 4-symbol), "
          f"monotone_violations = {violations}")


def test_adversarial():
    print("\n=== 2. ADVERSARIAL FAMILIES ===")
    # 1. Symmetric-Middle-Run: a M^K b (with multiple markers so valid cyclic)
    # Referee's construction: e.g. T = 0 1^K 2 0 1^K 2 ... or unique markers around M^K
    K = 64
    T_smr = []
    for i in range(32):
        T_smr.extend([0, 1 + (i % 4)] + [10] * K + [20 + (i // 4)])
    L, p = bwt_encode(T_smr)
    ref, _, _, _, _ = decode_lf_reference(L, p)
    for strat in ("zero_self_first", "min_deg", "max_symbol"):
        out, st = run_md_iet_schur(L, p, strategy=strat)
        assert out == ref
        print(f"Symmetric-Middle-Run (n={len(L)}, r0={st['r0']}) [{strat}]: "
              f"r_init={st['r_init']}, r_max={st['r_max']}, trans={st['total_transitions']} "
              f"({st['total_transitions']/st['r0']:.2f} r0), slp={st['slp_size']}, final_steps={st['final_steps']}")

    # 2. Fib_{2m} + $
    a, b = [0], [0, 1]
    for _ in range(14):
        a, b = b, b + a
    T_fib = b + [2]  # 2 is sentinel
    L, p = bwt_encode(T_fib)
    ref, _, _, _, _ = decode_lf_reference(L, p)
    for strat in ("zero_self_first", "min_deg", "max_symbol"):
        out, st = run_md_iet_schur(L, p, strategy=strat)
        assert out == ref
        print(f"Fib+$ (n={len(L)}, r0={st['r0']}) [{strat}]: "
              f"r_init={st['r_init']}, r_max={st['r_max']}, trans={st['total_transitions']} "
              f"({st['total_transitions']/st['r0']:.2f} r0), slp={st['slp_size']}, final_steps={st['final_steps']}")

    # 3. Phi_BWT(pi, b=2, K=8) for random single-cycle pi of size M=64
    import random
    rng = random.Random(123)
    M = 64
    perm = list(range(M))
    rng.shuffle(perm)
    # make single cycle
    u = [0] * M
    for i in range(M):
        u[i] = perm[i]
    L_bits = 6
    K_skel = 8
    # Alphabet: #=0, $=1, bit0=2, bit1=3
    T_phi = []
    for x in u:
        T_phi.append(0)
        T_phi.extend([1] * (K_skel - 1))
        for bit in range(L_bits - 1, -1, -1):
            T_phi.append(2 + ((x >> bit) & 1))
    L, p = bwt_encode(T_phi)
    ref, _, _, _, _ = decode_lf_reference(L, p)
    for strat in ("zero_self_first", "min_deg", "max_symbol"):
        out, st = run_md_iet_schur(L, p, strategy=strat)
        assert out == ref
        print(f"Phi_BWT(M={M}, K={K_skel}) (n={len(L)}, r0={st['r0']}) [{strat}]: "
              f"r_init={st['r_init']}, r_max={st['r_max']}, trans={st['total_transitions']} "
              f"({st['total_transitions']/st['r0']:.2f} r0), slp={st['slp_size']}, final_steps={st['final_steps']}")
        for h in st['history']:
            print(f"   elim e={h['e']} (|F_e|={h['bucket_size']}): in={h['r_in']} out={h['r_out']} self={h['r_self']} "
                  f"mono={h['mono']} trans={h['transitions']} spirals={h['spiral_jumps']} r_after={h['r_after']}")


def test_wikidata_blocks():
    print("\n=== 3. AUTHENTICATED WIKIDATA BLOCKS (block-0.L first, then all 6) ===")
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)

    for blk in manifest["blocks"]:
        path = os.path.join(BLOCKS_DIR, blk["file"])
        data = open(path, "rb").read()
        sha = hashlib.sha256(data).hexdigest()
        assert sha == blk["sha256"], f"SHA mismatch on {blk['file']}"
        p = blk["primary_index"]
        n = blk["n"]
        assert len(data) == n

        # Build initial IET and trace elimination curve (stopping BEFORE non-monotone explosion, AND full!)
        alpha, C, counts, pieces_by_src, slp, r0 = build_initial_iet(data)
        p_bucket = None
        for c in alpha:
            if C[c] <= p < C[c] + counts[c]:
                p_bucket = c
                break
        r_init = sum(len(v) for v in pieces_by_src.values())
        print(f"\n--- {blk['file']}: n={n}, r0={r0}, r_init={r_init}, |Sigma|={len(alpha)}, p_bucket={p_bucket} (|F_p|={counts[p_bucket]}) ---")

        # Trace zero_self_first elimination step by step!
        cum_trans = 0
        cum_elim_rows = 0
        cum_deg = 0
        phase_zero_self = None
        phase_mono = None
        phase_ratio_2 = None
        r_cur = r_init
        r_max = r_init
        step = 0

        # Track stats at key checkpoints
        while len(pieces_by_src) > 1:
            cands = [b for b in pieces_by_src.keys() if b != p_bucket]
            r_in_map = defaultdict(int)
            r_out_map = defaultdict(int)
            r_self_map = defaultdict(int)
            self_lists = defaultdict(list)
            for sb, plist in pieces_by_src.items():
                for piece in plist:
                    db = piece[4]
                    if sb == db:
                        r_self_map[sb] += 1
                        self_lists[sb].append(piece)
                    else:
                        r_out_map[sb] += 1
                        r_in_map[db] += 1

            # Pick best candidate: prefer r_self==0, then r_self==1, then monotone, then min_deg
            def key_fn(b):
                rs = r_self_map[b]
                if rs == 0:
                    tier = 0
                elif rs == 1:
                    tier = 1
                elif is_monotone_pieces(self_lists[b]):
                    tier = 2
                else:
                    tier = 3
                deg = r_in_map[b] + r_out_map[b] + 2 * rs
                return (tier, deg, counts[b], b)

            best = min(cands, key=key_fn)
            tier = key_fn(best)[0]
            deg = r_in_map[best] + r_out_map[best] + 2 * r_self_map[best]

            if tier > 0 and phase_zero_self is None:
                phase_zero_self = (step, n - cum_elim_rows, r_cur, cum_trans, cum_deg, len(pieces_by_src))
            if tier > 2 and phase_mono is None:
                phase_mono = (step, n - cum_elim_rows, r_cur, cum_trans, cum_deg, len(pieces_by_src))

            # Safety cap: if tier == 3 and remaining rows is large, still run on block-0 to see exact blowup!
            if tier == 3 and cum_trans > 5_000_000:
                print(f"  [STOPPED at step {step}, rem_buckets={len(pieces_by_src)}, rem_rows={n-cum_elim_rows}, r_cur={r_cur}, cum_trans={cum_trans}]")
                break

            st = eliminate_bucket(best, pieces_by_src, slp, use_spiral=True)
            step += 1
            r_cur = sum(len(v) for v in pieces_by_src.values())
            if r_cur > r_max:
                r_max = r_cur
            cum_trans += st['transitions']
            cum_elim_rows += counts[best]
            cum_deg += deg

            if st['transitions'] > 2 * max(1, deg) and phase_ratio_2 is None:
                phase_ratio_2 = (step, best, st['r_self'], st['mono'], deg, st['transitions'], n - cum_elim_rows, r_cur)

            if blk["file"] == "block-0.L" and (step <= 10 or step % 15 == 0 or tier >= 2 or len(pieces_by_src) <= 15):
                print(f"  step={step:3d} rem_b={len(pieces_by_src):3d} e={best:3d}({chr(best) if 32<=best<127 else '.':1s}) "
                      f"tier={tier} |F_e|={counts[best]:6d} in={st['r_in']:5d} out={st['r_out']:5d} self={st['r_self']:4d} "
                      f"trans={st['transitions']:7d} (t/deg={st['transitions']/max(1,deg):5.2f}) "
                      f"rem_n={n-cum_elim_rows:6d} r_cur={r_cur:5d}")

        print(f"  SUMMARY {blk['file']}:")
        print(f"    End of tier 0 (r_self=0): {phase_zero_self}")
        print(f"    End of tier<=2 (monotone): {phase_mono}")
        print(f"    First trans > 2*deg:       {phase_ratio_2}")
        print(f"    Final: steps={step}, rem_b={len(pieces_by_src)}, rem_n={n-cum_elim_rows} ({(n-cum_elim_rows)/n:.4f}n), "
              f"r_cur={r_cur} (r_max={r_max}, mono_ok={r_max<=r_init}), cum_trans={cum_trans} ({cum_trans/r0:.2f} r0), "
              f"cum_deg={cum_deg} ({cum_deg/r0:.2f} r0)")

        # Verify exactness on block-0.L if we didn't hit safety cap (or even if stopped early, reconstruct from remaining buckets!)
        out, walk_steps = reconstruct_from_iet_and_primary(pieces_by_src, slp, data, p)
        ref_out, ref_len, _, _, _ = decode_lf_reference(data, p)
        assert out == ref_out, f"EXACTNESS FAILURE on {blk['file']}!"
        print(f"    EXACTNESS VERIFIED! walk_steps={walk_steps} ({walk_steps/n:.4f}n), slp_nodes={len(slp)}")


if __name__ == "__main__":
    test_exhaustive()
    test_adversarial()
    test_wikidata_blocks()
