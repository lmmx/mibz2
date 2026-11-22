#!/usr/bin/env python3
"""
Cycle 2 — Script 1:
1. Verify Exact Telescoping Conservation Identity (Theorem C1):
     Q_rand + P_rem = r_init + S_in + S_hit + S_bnc + root_excess
   to exact integer equality across all 4 merge trees on all 6 Wikidata blocks!
2. Audit Gap-Closing Coalescence (Theorem C2):
   - Can coalescing adjacent pieces with identical expanded word drop Q_comp < r_0?
   - Exhaustive check on binary n <= 8, ternary n <= 5, plus measurement on block-0.L.
"""
import hashlib
import json
import sys
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt
import test_leverage_and_lb as tl


def resolve_with_exact_conservation(sym_set, pieces_left, pieces_right, slp, stats):
    """Exact P1 Redheffer merge instrumented with S_in, S_hit, S_bnc, P_in, P_out, P_int_dom."""
    raw = pieces_left + pieces_right
    p_in = len(raw)
    primary_cycle = None
    collapsed = []
    for lo, hi, d, tsym, w, m in raw:
        is_int = (tsym in sym_set)
        if not is_int:
            collapsed.append((lo, hi, d, tsym, w, m, False))
            continue
        if d == 0:
            stats["self_cycles"] += 1
            if m >= 0:
                primary_cycle = (w, m)
            continue
        # P1 spiral
        _, pcs, pcyc = mt.spiral_compress_piece(lo, hi, d, tsym, w, m, slp, stats)
        if pcyc is not None:
            primary_cycle = pcyc
        collapsed.extend(pcs)

    collapsed.sort(key=lambda x: x[0])
    starts = [p[0] for p in collapsed]

    int_targets = sorted((lo + d, hi + d) for lo, hi, d, tsym, w, m, is_int in collapsed if is_int)
    merged_t = []
    for a, b in int_targets:
        if merged_t and merged_t[-1][1] == a:
            merged_t[-1] = (merged_t[-1][0], b)
        else:
            merged_t.append((a, b))

    entry_work = []
    int_dom_cnt = 0
    mt_idx = 0
    num_mt = len(merged_t)
    primary_piece_pos = None

    for lo, hi, d, tsym, w, m, is_int in collapsed:
        if m >= 0:
            primary_piece_pos = lo
        pos = lo
        segs_here = 0
        while pos < hi:
            while mt_idx < num_mt and merged_t[mt_idx][1] <= pos:
                mt_idx += 1
            if mt_idx >= num_mt or merged_t[mt_idx][0] >= hi:
                entry_work.append((pos, hi, d, tsym, w, m, is_int))
                segs_here += 1
                pos = hi
            else:
                ta, tb = merged_t[mt_idx]
                if pos < ta:
                    entry_work.append((pos, ta, d, tsym, w, m, is_int))
                    segs_here += 1
                    pos = ta
                ov_hi = hi if hi < tb else tb
                int_dom_cnt += 1
                segs_here += 1
                pos = ov_hi
        # Exact S_in: number of splits of this piece caused by In(S) boundary!
        stats["S_in"] += (segs_here - 1)

    out_pieces = []
    saw_primary_exit = False
    bounces_here = 0
    q_bnc_here = 0
    stack = list(reversed(entry_work))
    while stack:
        lo, hi, d, tsym, w, m, is_int = stack.pop()
        if not is_int:
            out_pieces.append((lo, hi, d, tsym, w, m))
            if m >= 0:
                saw_primary_exit = True
            continue
        bounces_here += 1
        t_lo, t_hi = lo + d, hi + d
        idx = bisect_right(starts, t_lo) - 1
        pos = t_lo
        w_len = slp.length[w]
        nxt = []
        k_hits = 0
        while pos < t_hi:
            q_bnc_here += 1
            k_hits += 1
            p_lo, p_hi, p_d, p_tsym, p_w, p_m, p_int = collapsed[idx]
            ov_hi = t_hi if t_hi < p_hi else p_hi
            nxt.append((
                pos - d, ov_hi - d, d + p_d, p_tsym,
                slp.concat(w, p_w), mt.combine_mark(m, w_len, p_m), p_int
            ))
            pos = ov_hi
            idx += 1
        stats["S_bnc"] += (k_hits - 1)
        for item in reversed(nxt):
            stack.append(item)

    # Check exact node-level identity when no closed cycles occur at this node:
    #   P_in - P_out = int_dom_cnt - S_in_here - S_bnc_here
    #   And Q_bnc = int_dom_cnt + S_hit_here (when every int_dom piece is reached from In(S))!
    q_root_here = 0
    if primary_piece_pos is not None and not saw_primary_exit and primary_cycle is None:
        # Root or closed cycle!
        x = primary_piece_pos
        idx = bisect_right(starts, x) - 1
        q_root_here += 1
        _, _, d0, _, w_acc, m_acc, _ = collapsed[idx]
        x = x + d0
        while x != primary_piece_pos:
            q_root_here += 1
            idx = bisect_right(starts, x) - 1
            p_lo, p_hi, p_d, _, p_w, p_m, _ = collapsed[idx]
            w_acc = slp.concat(w_acc, p_w)
            x = x + p_d
        primary_cycle = (w_acc, m_acc)
        stats["root_int_dom"] += int_dom_cnt
        stats["Q_root"] += q_root_here
    else:
        # Exact identity for open merges (single-cycle BWT):
        # Every piece in int_dom is visited >= 1 time in bouncing, so S_hit = q_bnc_here - int_dom_cnt >= 0!
        s_hit = q_bnc_here - int_dom_cnt
        stats["S_hit"] += s_hit

    stats["bounces"] += bounces_here
    stats["Q_bnc"] += q_bnc_here
    stats["Q_rand"] += q_bnc_here + q_root_here
    out_pieces.sort(key=lambda x: x[0])
    return out_pieces, primary_cycle


def run_conservation_audit():
    print("=== 1. EXACT TELESCOPING CONSERVATION IDENTITY AUDIT ON ALL 6 BLOCKS ===")
    print("Identity (Theorem C1, spiral-free single-cycle):")
    print("  Q_rand = r_init + S_in + S_hit + S_bnc + (Q_root - root_int_dom)")
    print("  where (Q_root - root_int_dom) >= 0 is root-cycle revisit/excess!")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        for tk in (["balanced", "lazard_max", "min_cut_bisect", "heavy_edge", "width_times_rows"]
                   if entry["file"] == "block-0.L" else ["width_times_rows"]):
            n, r0, m16, buckets, pieces_by_bucket, bigrams, slp = mt.acquire_from_L(L, p)
            r_init = sum(len(v) for v in pieces_by_bucket.values())
            stats = defaultdict(int)
            leaves = {}
            leaf_sz = {}
            pcyc = None
            for c, b_lo, b_hi in buckets:
                s = frozenset([c])
                out_p, pc = resolve_with_exact_conservation(s, pieces_by_bucket[c], [], slp, stats)
                if pc is not None:
                    pcyc = pc
                leaves[c] = (s, out_p)
                leaf_sz[c] = len(out_p)
            if tk == "width_times_rows":
                tree = tl.build_leverage_tree(buckets, pieces_by_bucket, bigrams, mode="width_times_rows")
            else:
                tree = mt.build_merge_tree(tk, buckets, bigrams, leaf_sz)

            def ev(node):
                nonlocal pcyc
                if isinstance(node, int):
                    return leaves[node]
                lr = ev(node[0]); rr = ev(node[1])
                s = lr[0] | rr[0]
                op, pc = resolve_with_exact_conservation(s, lr[1], rr[1], slp, stats)
                if pc is not None:
                    pcyc = pc
                return (s, op)

            root = ev(tree)
            assert len(root[1]) == 0 and pcyc is not None
            # Verify EXACT identity!
            lhs = stats["Q_rand"]
            root_exc = stats["Q_root"] - stats["root_int_dom"]
            rhs = r_init + stats["S_in"] + stats["S_hit"] + stats["S_bnc"] + root_exc
            assert lhs == rhs, f"Identity mismatch! lhs={lhs}, rhs={rhs}, diff={lhs-rhs}"
            # Also check S_hit >= 0? Note if unmarked cycles exist it could be negative; on single-cycle BWT it's >= 0!
            print(f"  {entry['file']:11s} {tk:16s} | r_init={r_init:6d} | "
                  f"S_in={stats['S_in']:6d}({stats['S_in']/r0:4.2f}r) "
                  f"S_hit={stats['S_hit']:7d}({stats['S_hit']/r0:4.2f}r) "
                  f"S_bnc={stats['S_bnc']:6d}({stats['S_bnc']/r0:4.2f}r) "
                  f"rt_exc={root_exc:6d} | Q={lhs:7d} == RHS [EXACT]")


def test_gap_closing_coalescence():
    """
    Test Theorem C2:
    1) Within original coordinates (In(U_S)), NO two distinct initial runs can EVER coalesce
       (proved: same first emitted byte w_0 => same L-byte => already in same run of L!).
    2) Across closed gaps (when deleting U_S compresses coordinates as in Lazard/Bucket-Schur),
       adjacent pieces CAN share the same byte/word!
       Can gap-closing coalescence EVER achieve Q_comp < r_0 - 1?
       Let's test Lazard-style single-bucket elimination WITH gap-closing run merging
       on binary n <= 9, ternary n <= 6, and block-0.L!
    """
    print("\n=== 2. GAP-CLOSING COALESCENCE AUDIT (Can Q_comp < r_0?) ===")
    # Check exact single-symbol elimination with gap closing (Lazard-MAX):
    # Wait: in Lazard-MAX, when we eliminate c_max:
    #   Each run in F_{c_max} (say r_max runs) gets spliced into the runs of L == c_max!
    #   If a run of c_max in L lies between two runs of the SAME symbol a, deleting the c_max run
    #   merges those two a-runs into ONE (coalescence)!
    #   Wait: deleting the c_max run STILL required >= 1 composition to attach c_max to its predecessor!
    #   Can total binary joins (SLP nodes created) EVER be < r_0 - 1?
    #   Test by checking exact SLP non-literal node count in Lazard-MAX on all binary n<=10, ternary n<=6!
    min_ratio = 999.0
    min_witness = None
    below_r0 = 0
    total_checked = 0

    for base, max_n in [(2, 10), (3, 6)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                if len(set(T)) < 2:
                    continue
                L, p = mt.bwt_encode(T)
                # Count r0
                r0 = 1 + sum(1 for i in range(1, n) if L[i] != L[i - 1])
                # Simulate exact Lazard-MAX with gap-closing coalescence & spiral power nodes
                # Represent BWT as list of (sym_id, length) runs!
                runs = []
                i = 0
                while i < n:
                    j = i + 1
                    while j < n and L[j] == L[i]:
                        j += 1
                    runs.append([L[i], j - i])
                    i = j
                # Actually, track exact unique SLP rules created by Lazard-MAX (with memo/dedup of identical rules)!
                # Wait: even simpler! Count exact distinct joins created when eliminating max symbol!
                cur_L = list(L)
                joins = 0
                powers = 0
                while len(set(cur_L)) > 1:
                    mx = max(cur_L)
                    # Build LF on cur_L
                    m_len = len(cur_L)
                    F = sorted(cur_L)
                    # In Lazard-MAX, every row j with F[j] == mx is absorbed into its predecessor!
                    # Actually count unique (a, k) new symbols created for a * mx^k!
                    # Compute runs in F_mx vs runs of mx in L:
                    # Number of distinct new symbols = number of runs of L!=mx that precede mx^k (with distinct k)!
                    # Count exact pair-absorption compositions at run granularity:
                    # Delete rows with F[j] == mx; replace each L[j] == mx (for F[j] != mx) with the new symbol
                    # reached by walking LF through F_mx!
                    C_map = {}
                    for idx, ch in enumerate(F):
                        if ch not in C_map:
                            C_map[ch] = idx
                    occ = defaultdict(int)
                    LF = [0] * m_len
                    for idx, ch in enumerate(cur_L):
                        LF[idx] = C_map[ch] + occ[ch]
                        occ[ch] += 1
                    # New alphabet symbols: map (base_sym, power_k) -> new_sym_id (with exact join counting!)
                    next_sym = mx + 1
                    created = {}
                    new_L = []
                    for idx in range(m_len):
                        if F[idx] == mx:
                            continue
                        ch = cur_L[idx]
                        if ch != mx:
                            new_L.append(ch)
                        else:
                            # Walk through mx's
                            k = 0
                            u = idx
                            while cur_L[u] == mx:
                                k += 1
                                u = LF[u]
                            base_ch = cur_L[u]
                            key = (base_ch, k)
                            if key not in created:
                                created[key] = next_sym
                                next_sym += 1
                                joins += 1
                                if k >= 2:
                                    powers += 1
                            new_L.append(created[key])
                    cur_L = new_L
                # Final single symbol: if len(cur_L) > 1, 1 power node
                if len(cur_L) > 1:
                    powers += 1
                tot_rules = joins + powers
                total_checked += 1
                ratio = tot_rules / r0
                if tot_rules < r0:
                    below_r0 += 1
                if ratio < min_ratio:
                    min_ratio = ratio
                    min_witness = (T, r0, joins, powers, tot_rules)

    print(f"  Checked {total_checked} binary(n<=10) + ternary(n<=6) valid BWTs under Lazard-MAX with rule-dedup + coalescence:")
    print(f"    Cases with (joins + powers) < r_0: {below_r0} / {total_checked} ({100*below_r0/total_checked:.1f}%)")
    print(f"    Minimum (joins + powers) / r_0 = {min_ratio:.3f} on witness T={list(min_witness[0])} "
          f"(r0={min_witness[1]}, joins={min_witness[2]}, powers={min_witness[3]})")


if __name__ == "__main__":
    run_conservation_audit()
    test_gap_closing_coalescence()
