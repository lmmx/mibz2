#!/usr/bin/env python3
"""
Adversarial Referee #3 (Cycle 14) — Independent Audit of Cycle 13 Results:
  1. Theorem L0: Exact Duval Lyndon factorization + exact BBWT on block-0.L & block-143.L
     (with full inverse-BBWT cycle verification!).
  2. Theorem L1: Independent Jez RLSLP (Jez0) exact expansion + g_rl on block-0.L
     and reversal-invariance on Fib_18+$ vs Fib_18^R+$.
  3. Beta-2 Scaling Phase Diagram (/tmp/fcp-b/cycle13_scaling_phase_diagram.py):
     Exact mathematical + sensitivity audit against shipped src/open.rs & measured silicon.

Zero imports from /tmp/fcp-a/ or /home/antigravity/lmmx/mibz2/.
"""
import json
import math
import time
from collections import Counter, defaultdict
from functools import cmp_to_key
import importlib.util


# ============================================================================
# 1. EXACT BWT INVERSION, DUVAL LYNDON FACTORIZATION, & EXACT BBWT
# ============================================================================

def ibwt_exact(L: bytes, primary_idx: int) -> bytes:
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    out = bytearray(n)
    x = primary_idx
    for k in range(n - 1, -1, -1):
        out[k] = L[x]
        x = LF[x]
    assert x == primary_idx, "LF did not close a single cycle at primary_idx!"
    return bytes(out)


def duval_lyndon(s: bytes):
    """Exact O(n) Duval Lyndon factorization returning list of (start, end) slices."""
    n = len(s)
    i = 0
    factors = []
    while i < n:
        j = i + 1
        k = i
        while j < n and s[k] <= s[j]:
            if s[k] < s[j]:
                k = i
            else:
                k += 1
            j += 1
        period = j - k
        while i <= k:
            factors.append((i, i + period))
            i += period
    # Independently verify: (1) covers [0, n), (2) non-increasing w_1 >= w_2 >= ... >= w_k,
    # (3) every factor starts with its minimum byte (necessary condition; full check on small factors).
    assert sum(b - a for a, b in factors) == n
    for idx in range(len(factors) - 1):
        a1, b1 = factors[idx]
        a2, b2 = factors[idx + 1]
        assert b1 == a2
        assert s[a1:b1] >= s[a2:b2], f"Duval non-increasing violated at {idx}"
    for a, b in factors:
        w = s[a:b]
        assert w[0] == min(w), "Factor does not begin with its minimum byte!"
    return factors


def count_runs(seq) -> int:
    if not seq:
        return 0
    r = 1
    prev = seq[0]
    for b in seq[1:]:
        if b != prev:
            r += 1
            prev = b
    return r


def compute_exact_bbwt(T: bytes, factors):
    """
    Exact Bijective BWT (Gil-Scott 2012):
    Sort all cyclic rotations of all Lyndon factors w_i by infinite-repetition omega-order u^omega.
    Fast exact implementation: progressive radix/bucket split by cyclic slices (16 -> 64 -> 256 -> 1024 -> 4096 -> chunked exact omega-compare).
    Verified by exact inverse BBWT cycle decomposition reproducing the exact Lyndon factor multiset!
    """
    n = len(T)
    f_bytes = [T[a:b] for a, b in factors]
    f_lens = [len(w) for w in f_bytes]
    f_dbl = [w + w for w in f_bytes]

    def cyclic_slice(fi, pos, off, length):
        L = f_lens[fi]
        start = (pos + off) % L
        if start + length <= 2 * L:
            return f_dbl[fi][start:start + length]
        reps = (length // L) + 2
        return (f_bytes[fi] * reps)[start:start + length]

    def cmp_omega_from(a, b, start_off):
        fi1, p1 = a
        fi2, p2 = b
        if fi1 == fi2 and p1 == p2:
            return 0
        l1, l2 = f_lens[fi1], f_lens[fi2]
        lim = l1 + l2
        off = start_off
        step = 1024
        while off < lim:
            ln = min(step, lim - off)
            s1 = cyclic_slice(fi1, p1, off, ln)
            s2 = cyclic_slice(fi2, p2, off, ln)
            if s1 != s2:
                return -1 if s1 < s2 else 1
            off += ln
        return 0

    def sort_bucket(items, off, widths):
        if len(items) <= 1:
            return items
        if not widths:
            items.sort(key=cmp_to_key(lambda a, b: cmp_omega_from(a, b, off)))
            return items
        w_cur = widths[0]
        sub = defaultdict(list)
        for fi, pos in items:
            sub[cyclic_slice(fi, pos, off, w_cur)].append((fi, pos))
        out = []
        for k in sorted(sub.keys()):
            out.extend(sort_bucket(sub[k], off + w_cur, widths[1:]))
        return out

    # Top-level bucket by 16-byte cyclic prefix
    top = defaultdict(list)
    for fi, L in enumerate(f_lens):
        if L >= 16:
            dbl = f_dbl[fi]
            for pos in range(L):
                top[dbl[pos:pos + 16]].append((fi, pos))
        else:
            for pos in range(L):
                top[cyclic_slice(fi, pos, 0, 16)].append((fi, pos))

    bbwt = bytearray(n)
    out_i = 0
    for key in sorted(top.keys()):
        items = sort_bucket(top[key], 16, (64, 256, 1024, 4096))
        for fi, pos in items:
            bbwt[out_i] = f_bytes[fi][pos - 1]
            out_i += 1
    assert out_i == n
    bbwt = bytes(bbwt)

    # AIRTIGHT VERIFICATION: Inverse BBWT LF-walk MUST decompose into exact Lyndon factors!
    counts = [0] * 256
    for b in bbwt:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(bbwt):
        LF[i] = cur[b]
        cur[b] += 1
    seen = bytearray(n)
    recovered_lens = []
    recovered_hashes = Counter()
    for start in range(n):
        if not seen[start]:
            cur_i = start
            buf = bytearray()
            while not seen[cur_i]:
                seen[cur_i] = 1
                buf.append(bbwt[cur_i])
                cur_i = LF[cur_i]
            buf.reverse()
            # Find canonical minimal rotation (Lyndon word is unique min rotation!)
            # Quick check: in BBWT inverse, starting at the first unvisited index `start`
            # visits the lexicographically smallest rotation of the cycle FIRST, so buf[::-1]
            # shifted by 1 IS the Lyndon word! Wait: verify directly by comparing byte multiset & rotation!
            w_rec = bytes(buf)
            # In LF walk starting at `start` (lex-min rotation of this factor):
            # L[start] is the LAST char of the Lyndon word, L[LF[start]] is second-to-last, ..., so reversed buf IS the exact Lyndon word!
            recovered_lens.append(len(w_rec))
            recovered_hashes[w_rec] += 1

    orig_factors_counter = Counter(f_bytes)
    assert recovered_hashes == orig_factors_counter, (
        f"BBWT INVERSE MISMATCH! cycles={len(recovered_lens)} vs factors={len(f_bytes)}"
    )
    return bbwt


def audit_theorem_l0():
    print("=" * 78)
    print("AUDIT 1: THEOREM L0 (DUVAL LYNDON FACTORIZATION + EXACT BBWT)")
    print("=" * 78)
    manifest = json.load(open("/tmp/mibz2-blocks/manifest.json"))
    by_name = {b["file"]: b for b in manifest["blocks"]}

    for fname in ("block-0.L", "block-143.L"):
        t0 = time.time()
        meta = by_name[fname]
        L = open(f"/tmp/mibz2-blocks/{fname}", "rb").read()
        assert len(L) == meta["n"]
        r_bwt = count_runs(L)
        assert r_bwt == meta["r"]
        T = ibwt_exact(L, meta["primary_index"])
        factors = duval_lyndon(T)
        lens = [b - a for a, b in factors]
        max_len = max(lens)
        max_pct = 100.0 * max_len / len(T)
        first_bytes = Counter(T[a] for a, _ in factors)
        zero_count = T.count(0)
        bbwt = compute_exact_bbwt(T, factors)
        r_bbwt = count_runs(bbwt)
        rel_diff_pct = 100.0 * (r_bbwt - r_bwt) / r_bwt
        dt = time.time() - t0
        print(f"[VERIFIED] {fname} (n={len(T)}, dt={dt:.2f}s):")
        print(f"           k_Lyn = {len(factors)}, max_factor = {max_len} ({max_pct:.2f}%), min_factor = {min(lens)}")
        print(f"           Factor start bytes: {dict(first_bytes)} (total 0x00 bytes in T = {zero_count})")
        print(f"           r(BWT) = {r_bwt}, r(BBWT) = {r_bbwt}, delta = {r_bbwt - r_bwt:+d} ({rel_diff_pct:+.4f}%) | inverse-BBWT == Lyndon factors: EXACT")


# ============================================================================
# 2. THEOREM L1: INDEPENDENT JEZ RLSLP (Jez0) & REVERSAL INVARIANCE
# ============================================================================

def jez_rlslp(seq, pair_mode="greedy_maxcut"):
    """
    Independent Jez Local Recompression RLSLP (Jez 2015 / Jez0):
      Each round:
        1. Block compression: replace every maximal run c^l (l >= 2) by fresh symbol R(c, l).
        2. Pair compression: partition active alphabet into (Left, Right) and replace every
           adjacent (a, b) with a in Left, b in Right by fresh symbol P(a, b).
    Returns (g_rl_rules, root_sym, rules_dict, rounds).
    Verifies 100% exact expansion back to `seq`!
    """
    arr = list(seq)
    next_id = max(arr) + 1 if arr else 256
    # Map (kind, a, b) -> sym_id; kind='run' (a=sym, b=len>=2) or kind='pair' (a=left, b=right)
    rule_to_id = {}
    id_to_rule = {}

    def get_run_id(sym, length):
        nonlocal next_id
        key = ("run", sym, length)
        if key not in rule_to_id:
            rule_to_id[key] = next_id
            id_to_rule[next_id] = key
            next_id += 1
        return rule_to_id[key]

    def get_pair_id(a, b):
        nonlocal next_id
        key = ("pair", a, b)
        if key not in rule_to_id:
            rule_to_id[key] = next_id
            id_to_rule[next_id] = key
            next_id += 1
        return rule_to_id[key]

    rounds = 0
    while len(arr) > 1:
        rounds += 1
        # 1. Block compression (runs of length >= 2)
        nxt = []
        i = 0
        m = len(arr)
        while i < m:
            c = arr[i]
            j = i + 1
            while j < m and arr[j] == c:
                j += 1
            if j - i >= 2:
                nxt.append(get_run_id(c, j - i))
            else:
                nxt.append(c)
            i = j
        arr = nxt
        if len(arr) <= 1:
            break

        # 2. Pair compression: deterministic partition (Sigma_L, Sigma_R)
        # Count adjacent distinct pairs (since runs are compressed, arr[i] != arr[i+1])
        if pair_mode == "greedy_maxcut":
            # Deterministic greedy max-cut over active symbols in order of descending degree
            adj_out = defaultdict(Counter)
            adj_in = defaultdict(Counter)
            deg = Counter()
            for idx in range(len(arr) - 1):
                u, v = arr[idx], arr[idx + 1]
                adj_out[u][v] += 1
                adj_in[v][u] += 1
                deg[u] += 1
                deg[v] += 1
            side = {}  # sym -> 0 (Left) or 1 (Right)
            for sym in sorted(deg.keys(), key=lambda x: (-deg[x], x)):
                # Gain if placed in Left (0): edges sym -> already-Right
                gain_L = sum(cnt for nb, cnt in adj_out[sym].items() if side.get(nb) == 1)
                # Gain if placed in Right (1): edges already-Left -> sym
                gain_R = sum(cnt for nb, cnt in adj_in[sym].items() if side.get(nb) == 0)
                side[sym] = 0 if gain_L >= gain_R else 1
        elif pair_mode == "greedy_maxcut_sym":
            # Symmetric-tie-break version: order by degree, tiebreak symmetrically? Wait: test both!
            adj = defaultdict(Counter)
            deg = Counter()
            for idx in range(len(arr) - 1):
                u, v = arr[idx], arr[idx + 1]
                adj[u][v] += 1
                adj[v][u] += 1
                deg[u] += 1
                deg[v] += 1
            side = {}
            for sym in sorted(deg.keys(), key=lambda x: (-deg[x], x)):
                gain_L = sum(cnt for nb, cnt in adj[sym].items() if side.get(nb) == 1)
                gain_R = sum(cnt for nb, cnt in adj[sym].items() if side.get(nb) == 0)
                side[sym] = 0 if gain_L >= gain_R else 1
        elif pair_mode == "parity":
            side = {s: (s & 1) for s in set(arr)}
            if len(set(side.values())) < 2:
                # fallback: lowest differing bit
                syms = sorted(set(arr))
                side = {s: (i & 1) for i, s in enumerate(syms)}

        nxt2 = []
        i = 0
        m = len(arr)
        while i < m:
            if i + 1 < m and side[arr[i]] == 0 and side[arr[i + 1]] == 1:
                nxt2.append(get_pair_id(arr[i], arr[i + 1]))
                i += 2
            else:
                nxt2.append(arr[i])
                i += 1
        if len(nxt2) == len(arr):
            # Fallback if directed cut had 0 L->R edges (e.g. all edges went R->L): flip sides!
            nxt2 = []
            i = 0
            while i < m:
                if i + 1 < m and side[arr[i]] == 1 and side[arr[i + 1]] == 0:
                    nxt2.append(get_pair_id(arr[i], arr[i + 1]))
                    i += 2
                else:
                    nxt2.append(arr[i])
                    i += 1
        assert len(nxt2) < len(arr), "Pair compression made zero progress!"
        arr = nxt2

    root = arr[0]
    # Exact expansion verification (iterative stack)
    out = []
    stack = [(root, 1)]
    while stack:
        sym, mult = stack.pop()
        if sym not in id_to_rule:
            out.extend([sym] * mult)
        else:
            kind, a, b = id_to_rule[sym]
            if kind == "run":
                stack.append((a, b * mult))
            else:
                for _ in range(mult):
                    stack.append((b, 1))
                    stack.append((a, 1))
    assert out == list(seq), "RLSLP EXACT EXPANSION MISMATCH!"
    return len(id_to_rule), root, id_to_rule, rounds


def bwt_encode_fast(T: bytes):
    """Fast O(n log n) or simple rotation sort for small n (e.g. Fib_18, n <= 7000)."""
    n = len(T)
    TT = T + T
    # Prefix-bucketed rotation sort for fast exact BWT on n ~ 4182
    rots = sorted(range(n), key=lambda i: TT[i:i + 64])
    # Verify exact order locally? Better: direct key TT[i:i+n] is fast in C for n=4182!
    rots = sorted(range(n), key=lambda i: TT[i:i + n])
    L = bytes(T[(i - 1) % n] for i in rots)
    return L, rots.index(0)


def audit_theorem_l1():
    print("\n" + "=" * 78)
    print("AUDIT 2: THEOREM L1 (JEZ RLSLP vs C_gap & REVERSAL INVARIANCE)")
    print("=" * 78)

    # 1. block-0.L
    manifest = json.load(open("/tmp/mibz2-blocks/manifest.json"))
    b0 = [b for b in manifest["blocks"] if b["file"] == "block-0.L"][0]
    L0 = open("/tmp/mibz2-blocks/block-0.L", "rb").read()
    T0 = ibwt_exact(L0, b0["primary_index"])
    r0 = b0["r"]

    t0 = time.time()
    g_dir, _, _, rnds_dir = jez_rlslp(T0, "greedy_maxcut")
    dt1 = time.time() - t0
    print(f"[VERIFIED] block-0.L (n={len(T0)}, r0={r0}): Jez RLSLP (directed greedy max-cut):")
    print(f"           g_rl = {g_dir} ({g_dir / r0:.4f} r0), rounds = {rnds_dir}, exact expansion = PASS ({dt1:.2f}s)")
    t0 = time.time()
    g_sym, _, _, rnds_sym = jez_rlslp(T0, "greedy_maxcut_sym")
    dt2 = time.time() - t0
    print(f"[VERIFIED] block-0.L: Jez RLSLP (undirected/symmetric greedy max-cut):")
    print(f"           g_rl = {g_sym} ({g_sym / r0:.4f} r0), rounds = {rnds_sym}, exact expansion = PASS ({dt2:.2f}s)")

    # 2. Fib_18 + $ vs Fib_18^R + $
    # Check both common Fibonacci indexings:
    #   F_1 = "0", F_2 = "01" (or "1", "0") -> find which has len ~ 2584 or 4181 and r0: 4 -> 19!
    a, b = [0], [0, 1]
    for k in range(3, 21):
        a, b = b, b + a
        if k in (16, 17, 18, 19):
            T_fwd = bytes([x + 1 for x in b] + [0])  # $=0 smallest
            T_rev = bytes([x + 1 for x in b[::-1]] + [0])
            T_fwd_hi = bytes(b + [2])                # $=2 largest
            T_rev_hi = bytes(b[::-1] + [2])
            if k in (17, 18):
                L_f, _ = bwt_encode_fast(T_fwd)
                L_r, _ = bwt_encode_fast(T_rev)
                L_fh, _ = bwt_encode_fast(T_fwd_hi)
                L_rh, _ = bwt_encode_fast(T_rev_hi)
                rf, rr = count_runs(L_f), count_runs(L_r)
                rfh, rrh = count_runs(L_fh), count_runs(L_rh)
                gf_dir, _, _, _ = jez_rlslp(T_fwd, "greedy_maxcut")
                gr_dir, _, _, _ = jez_rlslp(T_rev, "greedy_maxcut")
                gf_sym, _, _, _ = jez_rlslp(T_fwd, "greedy_maxcut_sym")
                gr_sym, _, _, _ = jez_rlslp(T_rev, "greedy_maxcut_sym")
                # Also test purely without sentinel (F_k vs F_k^R) and note reversal-invariance!
                print(f"  Fib_{k} (len(F)={len(b)}, n={len(T_fwd)}):")
                print(f"    [$=min] r0(F+$)={rf}, r0(F^R+$)={rr} | [$=max] r0(F+$)={rfh}, r0(F^R+$)={rrh}")
                print(f"    Jez RLSLP directed:  g_rl(F+$)={gf_dir}, g_rl(F^R+$)={gr_dir} (exact expansion PASS)")
                print(f"    Jez RLSLP symmetric: g_rl(F+$)={gf_sym}, g_rl(F^R+$)={gr_sym} (exact expansion PASS)")
    # Note: Also check Fibonacci indexing F_0=b, F_1=a, F_2=ab, ... (len F_18 = 4181 vs 2584)
    # In our loop above, k=1 is [0] (len 1), k=2 is [0,1] (len 2), so k=17 has len 2584 and k=18 has len 4181!
    # Also check F_1=1, F_2=0, F_3=01 (len F_18 = 2584, F_17 = 1597) — already covered by k=17 above!


# ============================================================================
# 3. AUDIT 3: BETA-2'S SCALING PHASE DIAGRAM SENSITIVITY & CODE-FACT CHECK
# ============================================================================

def audit_phase_diagram():
    print("\n" + "=" * 78)
    print("AUDIT 3: BETA-2 SCALING PHASE DIAGRAM (MATHEMATICAL & SENSITIVITY AUDIT)")
    print("=" * 78)
    spec = importlib.util.spec_from_file_location("pd", "/tmp/fcp-b/cycle13_scaling_phase_diagram.py")
    pd = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(pd)

    # 1. Check baseline sanity at Wikidata scale (n=900,000, rho=10.6 and rho=17.58 = block-0.L)
    print("  1. Calibration sanity check at Wikidata scale (n = 900,000):")
    for rho in (10.6, 17.58, 20.0, 50.0):
        o1 = pd.time_open_cells_ms(900_000, rho, 1, "heavy_tail", True)["ms"]
        b1 = pd.time_b4_grid_flat16_ms(900_000, rho, 1)["ms"]
        d1 = pd.time_s2_dt_heavy_ms(900_000, rho, 1)["ms"]
        print(f"     n=900k, rho={rho:5.2f} (1w): Open={o1:.3f} ms, B4={b1:.3f} ms (B4/Open={b1/o1:.2f}x), S2_dt={d1:.3f} ms (dt/Open={d1/o1:.2f}x)")

    # 2. Sensitivity audit of crossovers n*(rho) at rho = 10, 20, 50!
    # Fix A: Correct edge_bytes threshold from n > 2^20 to actual src/open.rs m16 > 2^20!
    # Fix B: Disable 4 KiB STLB penalty (2 MiB Linux THP for Vecs > 2 MiB)
    # Fix C: Add realistic SLP DAG traversal latency (0.5 * l_b4 / 4 per run leaf = r0 * 0.125 * l_b4) to B4/S2_dt emit!
    def time_open_fixed(n, rho, workers, fix_edge=True, fix_thp=False):
        r0 = n / rho
        m16 = pd.thin_retained_rows(n, rho, "heavy_tail")
        edge_bytes = (4 if m16 <= (1 << 20) else 8) if fix_edge else (4 if n <= (1 << 20) else 8)
        thick_frac = 0.0396 * (rho / 10.6) if rho <= 16.0 else 0.06 + 0.94 * (1.0 - 16.0 / rho)
        ns = min(r0, max(1.0, r0 * thick_frac))

        def rand_ns(ws):
            if ws <= 0:
                return pd.LAT_L1_NS
            l3_eff = pd.L3_TOTAL_B / workers
            p_l1 = min(1.0, pd.L1D_B / ws)
            p_l2 = max(0.0, min(1.0, pd.L2_B / ws) - p_l1)
            p_l3 = max(0.0, min(1.0, l3_eff / ws) - min(1.0, pd.L2_B / ws))
            p_dram = max(0.0, 1.0 - max(pd.L2_B, l3_eff) / ws)
            lat_dram = pd.LAT_DRAM_1T_NS if workers == 1 else pd.LAT_DRAM_10W_NS
            tlb = 0.0 if fix_thp else max(0.0, 1.0 - pd.STLB_REACH_B / ws) * pd.TLB_PENALTY_NS
            return p_l1 * pd.LAT_L1_NS + p_l2 * pd.LAT_L2_NS + p_l3 * pd.LAT_L3_NS + p_dram * lat_dram + tlb

        l_edges = rand_ns(edge_bytes * m16)
        l_rv = rand_ns(48.0 * ns)
        t_s1 = r0 * 3.5 + m16 * 0.35 + n * 0.015
        t_s2 = (3.93 * ns) * (22.2 * max(1.0, l_rv / pd.LAT_L2_NS))
        spell_bytes = min(0.5 * n, 0.042 * n * math.sqrt(max(1.0, rho / 10.6)))
        t_s3a = (1.26 * ns) * (31.0 * max(1.0, l_rv / pd.LAT_L2_NS)) + spell_bytes * pd.seq_emit_ns_per_byte(spell_bytes, workers, 0.22)
        t_s4 = m16 * (0.18 * l_edges + 0.15)
        t_emit_chase = m16 * (0.45 * l_edges)
        t_emit_copy = n * pd.seq_emit_ns_per_byte(n, workers, 0.26)
        t_emit = max(t_emit_chase, t_emit_copy) + 0.35 * min(t_emit_chase, t_emit_copy)
        return (t_s1 + t_s2 + t_s3a + t_s4 + t_emit) / 1e6

    def time_b4_with_slp_emit(n, rho, workers, slp_emit_factor=0.0):
        r0 = n / rho
        ws_b4 = 48.0 * r0
        l_b4 = pd.random_access_ns(ws_b4, workers)
        op_ns = 55.6 * (0.55 + 0.45 * (l_b4 / pd.LAT_L3_NS))
        t_contract = (1.45 * r0) * op_ns
        # Conservative SLP traversal cost: slp_emit_factor * r0 random node derefs during DFS expansion
        t_emit = n * pd.seq_emit_ns_per_byte(n, workers, 0.26) + slp_emit_factor * r0 * l_b4
        return (t_contract + t_emit) / 1e6

    def bisect_cross(fn_open, fn_comp):
        lo, hi = 1e5, 1e10
        if fn_comp(lo) < fn_open(lo):
            return lo
        if fn_comp(hi) >= fn_open(hi):
            return None
        for _ in range(60):
            mid = math.sqrt(lo * hi)
            if fn_comp(mid) < fn_open(mid):
                hi = mid
            else:
                lo = mid
        return hi

    print("\n  2. Crossover n*(rho) sensitivity for B4-Flat16 vs ungated Open (1w / 10w):")
    for workers in (1, 10):
        print(f"     --- {workers}w ---")
        for rho in (10, 20, 50):
            orig = pd.find_crossover_n(rho, workers, pd.time_b4_grid_flat16_ms, "heavy_tail", True)
            fix_e = bisect_cross(
                lambda n: time_open_fixed(n, rho, workers, fix_edge=True, fix_thp=False),
                lambda n: pd.time_b4_grid_flat16_ms(n, rho, workers)["ms"],
            )
            fix_es = bisect_cross(
                lambda n: time_open_fixed(n, rho, workers, fix_edge=True, fix_thp=False),
                lambda n: time_b4_with_slp_emit(n, rho, workers, slp_emit_factor=0.5),
            )
            fmt = lambda v: "None" if v is None else f"{v/1e6:.2f}M"
            print(f"     rho={rho:2d}: Beta-2 orig={fmt(orig):>8s} | Fix edge m>2^20={fmt(fix_e):>8s} | +0.5*r0 SLP emit={fmt(fix_es):>8s}")


if __name__ == "__main__":
    audit_theorem_l0()
    audit_theorem_l1()
    audit_phase_diagram()
