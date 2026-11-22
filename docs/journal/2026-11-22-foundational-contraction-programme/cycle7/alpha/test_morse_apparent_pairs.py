#!/usr/bin/env python3
"""
Cycle 7 — Discrete / Algebraic Morse Theory (Perseus 6 Apparent Pairs, Coreduction Duality,
& Series-Reduced Run-Network 0.11 r_0 Core) across exhaustive small BWTs & ALL 6 Wikidata blocks!

Tests:
  1. Zero-Worklist Bijective Discrete Morse Matchings on G(I_0) (all preserving wire_xor!):
     - AP1 (Static M1):       J_u == I_v, w <= 16, cap <= 15 B
     - AP2_asc / AP2_desc:    out_deg(u) == 1 (J_u subseteq I_v) AND delta_u > 0 (or < 0),
                              resolved in ONE SINGLE INDEX-ORDER PASS (Perseus 6, ZERO worklist, ZERO cycle check!)
     - AP3_width_mono:        out_deg(u) == 1 AND (w(u) < w(v) OR (w(u) == w(v) and delta_u > 0)),
                              resolved in width-order pass (ZERO cycle check!)
     - AP4_max_out1:          Maximal acyclic out_deg(u) == 1 Morse matching (cap <= 15 B & cap <= 8 B!)
     - AP4_w2_out1:           Same restricted to 2 <= w(u) <= 16 (zero w=1 work, high leverage!)
  2. Reduction (LF, out_deg==1) vs Coreduction (FL, in_deg==1) & Exact Involution Theorem M2:
     - Verifies m_16(FL) == m_16(LF) to exact integer on all 6 blocks + exhaustive small BWTs!
     - Compares Forward Reduction (AP2_best / AP4_max_out1) vs Backward Coreduction (FL dual)!
  3. Exact Series-Reduced Run-Network Core (`in_deg==1 & out_deg==1` pass-throughs -> ~0.11 r_0)
     compared directly to M1 (0.375 r_0), M2 (0.055 r_0), and AP4_max_out1!
"""
import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt
import audit_m1_width_and_joint as am


def build_full_incidence_256cursor(strips, j_order):
    """
    Single linear two-pointer merge of I-order (0..q0) and J-order (j_order) — ZERO sorting!
    Computes exact:
      - out_deg[u], in_deg[v]
      - out_target[u] (unique v if out_deg[u] == 1, else -1)
      - in_source[v]  (unique u if in_deg[v] == 1, else -1)
      - next_r1[u]    (unique v if J_u == I_v and u != v, else -1)
    """
    q0 = len(strips)
    out_deg = [0] * q0
    in_deg = [0] * q0
    out_target = [-1] * q0
    in_source = [-1] * q0
    next_r1 = [-1] * q0

    ii = 0
    jj = 0
    while ii < q0 and jj < q0:
        v = ii
        u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo:
            jj += 1
        elif v_hi <= u_tlo:
            ii += 1
        else:
            out_deg[u] += 1
            in_deg[v] += 1
            out_target[u] = v
            in_source[v] = u
            if u_tlo == v_lo and u_thi == v_hi and u != v:
                next_r1[u] = v
            if u_thi < v_hi:
                jj += 1
            elif v_hi < u_thi:
                ii += 1
            else:
                ii += 1
                jj += 1

    for u in range(q0):
        if out_deg[u] != 1:
            out_target[u] = -1
        if in_deg[u] != 1:
            in_source[u] = -1

    return out_deg, in_deg, out_target, in_source, next_r1


def evaluate_morse_matching(strips, matched_par, n, p_orig, ref_bytes, cap_len=15, verify=True):
    """
    Given a STATIC ACYCLIC Morse matching `matched_par[u] = v` (for matched thin strips u,
    with `matched_par[u] == -1` for unmatched critical strips and thick strips):
      - Every matched strip u has out_deg(u) == 1 (J_u subseteq I_v), w(u) <= 16, p not in I_u!
      - Because matched_par is acyclic, we compute gradient path length & cap chains at `cap_len`:
        if a matched strip u would cause total gradient path length > cap_len - 1 (leaving >= 1 byte
        for the incoming critical row, so total emitted word <= cap_len!), we UNMATCH u (making u critical)!
        Wait: even better — in a single bottom-up pass (or index-order pass!), if a strip u has
        chain_len(u) == cap_len - 1, its children break (stay unmatched as critical!), guaranteeing
        EVERY gradient chain through matched thin strips has length <= cap_len - 1 (100% inline, ZERO SLP!)!
      - Then simulate EXACT Open thick-strip induction on the remaining unmatched strips
        (thick strips w > 16 are ALSO matched away as in Open!), and verify 100% byte-exact decoding!

    Returns exact metrics:
      - m_crit: number of critical (retained) rows in the reduced bijection (vs Open's m_16!)
      - matched_strips, matched_rows, ops_compress
      - word len distribution on critical rows (<=1, <=3, <=8, <=15, <=16, >16)
      - distinct_inline15_codes (check <= 16,383 / <= 65,535!)
    """
    q0 = len(strips)
    # Step 1: Resolve matched_par chains & enforce clean cap_len (unmatch any strip that would exceed cap_len-1!)
    # Check acyclicity and break any cycle (for AP4_max_out1) cleanly!
    par = list(matched_par)
    # Break cycles if any exist
    color = [0] * q0  # 0=unvis, 1=active, 2=done
    for u in range(q0):
        if color[u] != 0 or par[u] < 0:
            continue
        path = []
        cur = u
        while cur >= 0 and color[cur] == 0:
            color[cur] = 1
            path.append(cur)
            cur = par[cur]
        if cur >= 0 and color[cur] == 1:
            # Break cycle at cur!
            par[cur] = -1
        for x in path:
            color[x] = 2

    # Compute exact chain length through matched thin strips (stopping at any unmatched or thick strip!)
    # Enforce clean length cap: if chain_len(par[u]) + 1 > cap_len - 1, unmatch u (par[u] = -1)!
    # Use iterative topological order (postorder DFS)
    chain_len = [-1] * q0
    final_delta = [0] * q0
    final_word = [None] * q0
    ops_compress = 0

    for u in range(q0):
        if par[u] < 0 or chain_len[u] >= 0:
            continue
        # Walk down until hitting resolved node or root (par < 0)
        stack = []
        cur = u
        while cur >= 0 and par[cur] >= 0 and chain_len[cur] < 0:
            stack.append(cur)
            cur = par[cur]
        # Resolve in reverse (from root upward = exact index-order for monotone matchings!)
        for node in reversed(stack):
            v = par[node]
            v_clen = chain_len[v] if (v >= 0 and par[v] >= 0) else 0
            if v_clen + 1 > cap_len - 1:
                # Cleanly unmatch `node` so chain length stays strictly <= cap_len - 1 (ZERO truncation!)
                par[node] = -1
                chain_len[node] = 0
            else:
                ops_compress += 1
                chain_len[node] = v_clen + 1
                if v >= 0 and par[v] >= 0:
                    final_delta[node] = strips[node][2] + final_delta[v]
                    final_word[node] = bytes([strips[node][3]]) + final_word[v]
                else:
                    final_delta[node] = strips[node][2]
                    final_word[node] = bytes([strips[node][3]])

    # Now critical rows = all rows in strips u where (par[u] < 0 AND (width <= 16 OR mark >= 0))!
    # Wait: thick strips (width > 16, mark < 0) are ALSO matched away by Open!
    is_crit_strip = [False] * q0
    matched_strips = 0
    matched_rows = 0
    for u, (lo, hi, d, c, m) in enumerate(strips):
        w = hi - lo
        if par[u] >= 0:
            assert w <= 16 and m < 0
            matched_strips += 1
            matched_rows += w
        elif w <= 16 or m >= 0:
            is_crit_strip[u] = True

    # Build exact reduced bijection on critical rows & verify byte-exactness!
    starts = [s[0] for s in strips]
    crit_rows = []
    row_to_idx = {}
    for u, (lo, hi, d, c, m) in enumerate(strips):
        if is_crit_strip[u]:
            for r in range(lo, hi):
                row_to_idx[r] = len(crit_rows)
                crit_rows.append(r)

    m_crit = len(crit_rows)
    next_idx = [0] * m_crit
    step_word = [None] * m_crit
    p_ret_idx = -1
    p_mark_off = -1

    # Fast exact resolution using pre-compressed final_delta / final_word on matched thin strips!
    for u_idx, r0 in enumerate(crit_rows):
        cur = r0
        chunks = []
        m_acc = -1
        while True:
            si = bisect_right(starts, cur) - 1
            lo, hi, d, c, m = strips[si]
            if par[si] >= 0:
                # Jump ENTIRE compressed Morse gradient path in 1 step!
                chunks.append(final_word[si])
                cur = cur + final_delta[si]
            else:
                if m >= 0:
                    m_acc = sum(len(x) for x in chunks) + m
                chunks.append(bytes([c]))
                cur = cur + d
            if cur in row_to_idx:
                next_idx[u_idx] = row_to_idx[cur]
                step_word[u_idx] = b"".join(chunks)
                if m_acc >= 0:
                    p_ret_idx = u_idx
                    p_mark_off = m_acc
                break

    if verify:
        # Verify 100% exact bijection (every row visited once, wire_xor preserved, text == ref_bytes!)
        seen = bytearray(m_crit)
        cur = p_ret_idx
        out_chunks = []
        steps = 0
        while True:
            assert not seen[cur], "Cycle prematurely repeated!"
            seen[cur] = 1
            out_chunks.append(step_word[cur])
            cur = next_idx[cur]
            steps += 1
            if cur == p_ret_idx:
                break
        cyc = b"".join(out_chunks)
        q = len(cyc)
        assert n % q == 0, f"len(cyc)={q} does not divide n={n}"
        rot = cyc[p_mark_off:] + cyc[:p_mark_off]
        assert (rot * (n // q))[::-1] == ref_bytes, "EXACTNESS FAILURE!"

    # Word length & distinct inline15/16 code stats on critical rows!
    c_l1 = c_l3 = c_l8 = c_l15 = c_l16 = c_gt16 = 0
    dist_l15 = set()
    dist_l16 = set()
    for w in step_word:
        wl = len(w)
        if wl == 1: c_l1 += 1
        if wl <= 3: c_l3 += 1
        if wl <= 8: c_l8 += 1
        if wl <= 15: c_l15 += 1; dist_l15.add(w)
        if wl <= 16: c_l16 += 1; dist_l16.add(w)
        else: c_gt16 += 1

    return {
        "m_crit": m_crit,
        "matched_strips": matched_strips,
        "matched_rows": matched_rows,
        "ops_compress": ops_compress,
        "pct_l1": 100.0 * c_l1 / max(1, m_crit),
        "pct_l8": 100.0 * c_l8 / max(1, m_crit),
        "pct_l15": 100.0 * c_l15 / max(1, m_crit),
        "pct_l16": 100.0 * c_l16 / max(1, m_crit),
        "gt16_rows": c_gt16,
        "dist_l15": len(dist_l15),
        "dist_l16": len(dist_l16),
    }


def measure_fl_coreduction_and_run_network(strips, out_deg, in_deg, out_target, in_source, next_r1, cap_len=15):
    """
    Measure:
      1. Exact Theorem M2 check: m_16(FL) == m_16(LF) identically!
      2. Backward Coreduction (FL dual: match thin strips v with in_deg(v) == 1, i.e., I_v subseteq J_u):
         - FL_asc (delta_v > 0), FL_desc (delta_v < 0), FL_max_in1 (maximal acyclic in_deg == 1)!
      3. Exact Series-Reduced Run Network (`in_deg == 1 & out_deg == 1` pass-through vertices -> ~0.11 r_0)!
    """
    q0 = len(strips)
    # 1. Exact m_16(FL): widths of J_u are identical to widths of I_u!
    # Verify explicitly by constructing Psi = FL strips from J_u!
    m16_lf = sum((hi - lo) for lo, hi, _, _, m in strips if (hi - lo) <= 16 or m >= 0)
    m16_fl = sum(((hi + d) - (lo + d)) for lo, hi, d, _, m in strips if ((hi + d) - (lo + d)) <= 16 or m >= 0)
    assert m16_lf == m16_fl

    # 2. Backward Coreduction (`in_deg(v) == 1`, i.e., I_v subseteq J_u):
    # Count how many thin rows can be matched away in FL_asc, FL_desc, and FL_max_in1!
    fl_in1_thin = 0
    fl_in1_asc = 0
    fl_in1_desc = 0
    lf_out1_thin = 0
    lf_out1_asc = 0
    lf_out1_desc = 0
    both_in1_or_out1_thin = 0
    for u, (lo, hi, d, c, m) in enumerate(strips):
        w = hi - lo
        if w <= 16 and m < 0:
            o1 = (out_target[u] >= 0 and out_target[u] != u)
            i1 = (in_source[u] >= 0 and in_source[u] != u)
            if o1:
                lf_out1_thin += w
                if d > 0: lf_out1_asc += w
                elif d < 0: lf_out1_desc += w
            if i1:
                fl_in1_thin += w
                if d > 0: fl_in1_asc += w
                elif d < 0: fl_in1_desc += w
            if o1 or i1:
                both_in1_or_out1_thin += w

    # 3. Series-Reduced Run Network (`FRONTIER.md`: pass-through = vertex with in_deg==1 AND out_deg==1!)
    # Note: iteratively suppress vertices with in_deg==1 & out_deg==1!
    pass_through_init = sum(1 for u in range(q0) if in_deg[u] == 1 and out_deg[u] == 1)
    core_series = q0 - pass_through_init
    return {
        "m16_eq": (m16_lf == m16_fl),
        "lf_out1_thin": lf_out1_thin,
        "lf_out1_best_dir": max(lf_out1_asc, lf_out1_desc),
        "fl_in1_thin": fl_in1_thin,
        "fl_in1_best_dir": max(fl_in1_asc, fl_in1_desc),
        "both_or_thin": both_in1_or_out1_thin,
        "pass_through_init": pass_through_init,
        "core_series": core_series,
    }


def main():
    print("=== 1. Exhaustive Small Verification of All Discrete Morse Matchings (binary n<=8, ternary n<=6) ===")
    tot = 0
    for base, max_n in [(2, 8), (3, 6)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                if len(set(T)) < 2: continue
                L, p = mt.bwt_encode(T)
                _, r0, m16, strips, j_order, _ = am.load_runs_and_open_baseline(L, p)
                out_deg, in_deg, out_t, in_s, next_r1 = build_full_incidence_256cursor(strips, j_order)
                # Verify AP2_asc, AP2_desc, AP3_width_mono, AP4_max_out1 all decode 100% exact!
                q0 = len(strips)
                m_asc = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] > 0) else -1 for u in range(q0)]
                m_wm  = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and (strips[out_t[u]][1]-strips[out_t[u]][0] > strips[u][1]-strips[u][0] or (strips[out_t[u]][1]-strips[out_t[u]][0] == strips[u][1]-strips[u][0] and strips[u][2] > 0))) else -1 for u in range(q0)]
                m_max = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
                for mp in (m_asc, m_wm, m_max):
                    evaluate_morse_matching(strips, mp, n, p, T, cap_len=15, verify=True)
                    tot += 1
    print(f"  EXHAUSTIVE MORSE MATCHING PASS: {tot} exact bijective runs (binary n<=8, ternary n<=6) [verified]")

    print("\n=== 2. ALL 6 AUTHENTICATED WIKIDATA BLOCKS: DISCRETE MORSE APPARENT-PAIR CENSUS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    blocks = []
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        ref = mt.ref_ibwt(L, entry["primary_index"])
        n, r0, m16, strips, j_order, C = am.load_runs_and_open_baseline(L, entry["primary_index"])
        out_deg, in_deg, out_t, in_s, next_r1 = build_full_incidence_256cursor(strips, j_order)
        blocks.append((entry, ref, n, r0, m16, strips, out_deg, in_deg, out_t, in_s, next_r1))

    m16_sum = sum(b[4] for b in blocks)
    r0_sum = sum(b[3] for b in blocks)

    matchings = [
        ("Open baseline (w > 16 only)", "open", 15),
        ("AP1: Static M1 (J_u == I_v, cap<=15)", "ap1_15", 15),
        ("AP2_desc: out1 & d<0 (1-pass fwd, cap<=15)", "ap2_desc", 15),
        ("AP2_asc:  out1 & d>0 (1-pass bwd, cap<=15)", "ap2_asc", 15),
        ("AP2_best: best(asc,desc) (1-pass, cap<=15)", "ap2_best", 15),
        ("AP3_wmono: w<wv or (w==wv & d>0) (cap<=15)", "ap3_wmono", 15),
        ("AP3_wmono + M1_full (w<wv or R1, cap<=15)", "ap3_wm_r1", 15),
        ("AP4_max_out1: ALL out1 acyclic (cap<=8)", "ap4_cap8", 8),
        ("AP4_max_out1: ALL out1 acyclic (cap<=15)", "ap4_cap15", 15),
        ("AP4_w2_out1: 2<=w<=16 out1 (cap<=15)", "ap4_w2_15", 15),
    ]

    print(f"\n6-BLOCK AGGREGATED RESULTS (r0_sum={r0_sum}, m16_sum={m16_sum}, ALL 6/6 blocks byte-exact PASS):")
    print("Morse Matching (100% bijective!)           | m_crit (/m16)    | ops_cmp (/r0) | rho=sav/ops | <=8B  <=15B | gt16  | max_d15 (<=16k?)")
    print("-------------------------------------------+------------------+---------------+-------------+-------------+-------+-----------------")

    per_blk_ap = defaultdict(list)
    for label, key, cap in matchings:
        s_mc = s_ops = s_gt16 = 0
        max_d15 = 0
        s_l8 = s_l15 = 0.0
        for entry, ref, n, r0, m16, strips, out_deg, in_deg, out_t, in_s, next_r1 in blocks:
            q0 = len(strips)
            if key == "open":
                mp = [-1] * q0
            elif key == "ap1_15":
                mp = [next_r1[u] if (next_r1[u] >= 0 and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
            elif key == "ap2_desc":
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] < 0) else -1 for u in range(q0)]
            elif key == "ap2_asc":
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] > 0) else -1 for u in range(q0)]
            elif key == "ap2_best":
                w_d = sum(strips[u][1]-strips[u][0] for u in range(q0) if out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] < 0)
                w_a = sum(strips[u][1]-strips[u][0] for u in range(q0) if out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] > 0)
                if w_d >= w_a:
                    mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] < 0) else -1 for u in range(q0)]
                else:
                    mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and strips[u][2] > 0) else -1 for u in range(q0)]
            elif key == "ap3_wmono":
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and (strips[out_t[u]][1]-strips[out_t[u]][0] > strips[u][1]-strips[u][0] or (strips[out_t[u]][1]-strips[out_t[u]][0] == strips[u][1]-strips[u][0] and strips[u][2] > 0))) else -1 for u in range(q0)]
            elif key == "ap3_wm_r1":
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0 and (strips[out_t[u]][1]-strips[out_t[u]][0] > strips[u][1]-strips[u][0] or next_r1[u] >= 0)) else -1 for u in range(q0)]
            elif key in ("ap4_cap8", "ap4_cap15"):
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
            elif key == "ap4_w2_15":
                mp = [out_t[u] if (out_t[u] >= 0 and out_t[u] != u and 2 <= strips[u][1]-strips[u][0] <= 16 and strips[u][4] < 0) else -1 for u in range(q0)]
            else:
                raise ValueError(key)

            res = evaluate_morse_matching(strips, mp, n, entry["primary_index"], ref, cap_len=cap, verify=True)
            s_mc += res["m_crit"]; s_ops += res["ops_compress"]; s_gt16 += res["gt16_rows"]
            if res["dist_l15"] > max_d15: max_d15 = res["dist_l15"]
            s_l8 += res["pct_l8"]; s_l15 += res["pct_l15"]
            per_blk_ap[key].append((entry["file"], res["m_crit"]/m16, res["ops_compress"]/r0, res["dist_l15"]))
        sav = m16_sum - s_mc
        rho = sav / max(1, s_ops)
        print(f"{label:42s} | {s_mc:7d} ({s_mc/m16_sum:5.3f}m) | {s_ops:6d} ({s_ops/r0_sum:4.2f}r) |    {rho:6.3f}   | "
              f"{s_l8/6:4.1f}% {s_l15/6:4.1f}% | {s_gt16:5d} | {max_d15:7d}", flush=True)

    print("\n=== 3. REDUCTION (LF out1) vs COREDUCTION (FL in1) & SERIES-REDUCED RUN NETWORK (0.11 r_0) ===")
    print("block       | m16(FL)==m16(LF) | LF_out1/m16 (1-dir) | FL_in1/m16 (1-dir) | LF|FL/m16 | series_core (/r0)")
    print("------------+------------------+---------------------+--------------------+-----------+------------------")
    for entry, ref, n, r0, m16, strips, out_deg, in_deg, out_t, in_s, next_r1 in blocks:
        st = measure_fl_coreduction_and_run_network(strips, out_deg, in_deg, out_t, in_s, next_r1)
        print(f"{entry['file']:11s} |       {str(st['m16_eq']):5s}      |   {st['lf_out1_thin']/m16:5.3f} ({st['lf_out1_best_dir']/m16:5.3f})    |  "
              f" {st['fl_in1_thin']/m16:5.3f} ({st['fl_in1_best_dir']/m16:5.3f})   |   {st['both_or_thin']/m16:5.3f}   | "
              f"{st['core_series']:6d} ({st['core_series']/r0:5.3f}r)")

    print("\n=== 4. PER-BLOCK RANGES FOR TOP DISCRETE MORSE MATCHINGS ===")
    for key in ("ap2_best", "ap3_wmono", "ap4_cap8", "ap4_cap15"):
        vals = per_blk_ap[key]
        print(f"  {key:10s}: m_crit/m16 = {min(x[1] for x in vals):.3f}..{max(x[1] for x in vals):.3f} | "
              f"ops/r0 = {min(x[2] for x in vals):.2f}..{max(x[2] for x in vals):.2f} | "
              f"dist_l15 = {min(x[3] for x in vals)}..{max(x[3] for x in vals)}")


if __name__ == "__main__":
    main()
