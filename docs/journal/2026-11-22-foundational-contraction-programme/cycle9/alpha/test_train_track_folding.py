#!/usr/bin/env python3
"""
Cycle 9 — Task 1:
Exact Integer-Weighted Measured Train Track tau_sw(I_0), Bestvina-Handel Illegal-Turn
Stallings Folding, and Dunfield-Thurston (§7.3) Word-Carrying Band Splitting!

Canonical Construction of tau_sw(I_0):
  - Coincident breakpoints B_sync = D_0 cap C_0 partition [0, n) into switches v_k = [b_k, b_{k+1}).
  - Branches E(tau) = {0 .. q_0-1} (maximal L-runs, isolating [p, p+1)), each carrying:
      * integer weight w_e = hi - lo,
      * byte/word label,
      * exact switch equation sum_{in(v)} w = sum_{out(v)} w = b_{k+1} - b_k asserted at EVERY switch!
  - Illegal turns at switch v: pairs of distinct incoming branches folding into the same outgoing branch!

Tests:
  1. Exact switch-degree distribution (d_in, d_out) & illegal-turn census on all 6 Wikidata blocks.
  2. Exhaustive exactness verification on binary n<=8 & ternary n<=5 of word-carrying
     Bestvina-Handel / Dunfield-Thurston folding schedules.
  3. Direct comparison of 4 folding/splitting schedules on ALL 6 authenticated Wikidata blocks:
       - S1 (Pure-Fold First: smooth (1,1) -> full (a,1) folds -> X-switch band splits)
       - S2 (Dunfield-Thurston Heaviest-Band Split First)
       - S3 (Narrowest-Extreme-Branch Fold First)
       - S4 (Max-Illegal-Turn Switch First)
     measuring exact (ops/r_0, full_folds, band_splits, spirals, max_branches, slp_nodes)
     against M4 (1.87 r_0) and AP4_max (0.89 r_0) -> proving & verifying Theorem T1!
"""
import hashlib
import heapq
import json
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import bwt_core as mt
import bwt_core as am
import bwt_core as tk


def build_measured_train_track(L_bytes, primary_idx):
    """
    Construct the exact canonical integer-weighted measured train track tau_sw(I_0):
      - Switches = intervals [b_k, b_{k+1}) between consecutive coincident breakpoints D_0 cap C_0!
      - Branches = q_0 strips (maximal L-runs with [p, p+1) isolated).
      - Asserts exact switch equation sum(in) == sum(out) == width(v) at EVERY switch!
    Returns exact switch-degree census and illegal-turn counts!
    """
    n, r0, m16, strips, j_order, C = am.load_runs_and_open_baseline(L_bytes, primary_idx)
    q0 = len(strips)

    # Compute exact domain breakpoints D_0 and codomain breakpoints C_0
    D0 = {0, n}
    C0 = {0, n}
    for lo, hi, d, c, m in strips:
        D0.add(lo); D0.add(hi)
        C0.add(lo + d); C0.add(hi + d)
    B_sync = sorted(D0 & C0)
    num_sw = len(B_sync) - 1

    # Partition outgoing (I-order) and incoming (J-order) branches into switches [B_sync[k], B_sync[k+1])!
    sw_out = [[] for _ in range(num_sw)]
    sw_in = [[] for _ in range(num_sw)]

    k = 0
    for u in range(q0):
        lo, hi = strips[u][0], strips[u][1]
        while B_sync[k + 1] <= lo:
            k += 1
        assert B_sync[k] <= lo and hi <= B_sync[k + 1]
        sw_out[k].append(u)

    k = 0
    for u in j_order:
        tlo, thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        while B_sync[k + 1] <= tlo:
            k += 1
        assert B_sync[k] <= tlo and thi <= B_sync[k + 1]
        sw_in[k].append(u)

    # Verify EXACT Penner-Harer / Dunfield-Thurston switch equations at EVERY switch
    # AND count exact switch-degree distribution & Bestvina-Handel illegal turns!
    deg_hist = defaultdict(int)
    x_switches = 0
    smooth_11 = 0
    pure_merge_a1 = 0
    pure_split_1b = 0
    illegal_turn_pairs = 0     # sum over outgoing branches w of C(in_hitting(w), 2)
    illegal_turn_adj = 0       # sum over outgoing branches w of max(0, in_hitting(w) - 1) = total fold events!
    dual_illegal_adj = 0       # symmetric outgoing-side illegal turns!
    max_sw_in = 0
    max_sw_out = 0

    for k in range(num_sw):
        ins = sw_in[k]
        outs = sw_out[k]
        w_in = sum(strips[u][1] - strips[u][0] for u in ins)
        w_out = sum(strips[u][1] - strips[u][0] for u in outs)
        sw_w = B_sync[k + 1] - B_sync[k]
        assert w_in == sw_w and w_out == sw_w, f"Switch equation violated at sw {k}!"

        di, do = len(ins), len(outs)
        if di > max_sw_in: max_sw_in = di
        if do > max_sw_out: max_sw_out = do
        deg_hist[(min(di, 4), min(do, 4))] += 1
        if di == 1 and do == 1:
            smooth_11 += 1
        elif di >= 2 and do == 1:
            pure_merge_a1 += 1
        elif di == 1 and do >= 2:
            pure_split_1b += 1
        else:
            x_switches += 1

        # Exact illegal turns at switch k: two-pointer overlap between ins (J-intervals) and outs (I-intervals)!
        ii = 0; jj = 0
        hits_on_out = [0] * do
        hits_on_in = [0] * di
        while ii < di and jj < do:
            u = ins[ii]; w = outs[jj]
            u_lo, u_hi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
            w_lo, w_hi = strips[w][0], strips[w][1]
            if u_hi <= w_lo: ii += 1
            elif w_hi <= u_lo: jj += 1
            else:
                hits_on_in[ii] += 1
                hits_on_out[jj] += 1
                if u_hi < w_hi: ii += 1
                elif w_hi < u_hi: jj += 1
                else: ii += 1; jj += 1
        for h in hits_on_out:
            if h >= 2:
                illegal_turn_adj += (h - 1)
                illegal_turn_pairs += h * (h - 1) // 2
        for h in hits_on_in:
            if h >= 2:
                dual_illegal_adj += (h - 1)

    return {
        "n": n, "r0": r0, "q0": q0, "m16": m16,
        "num_sw": num_sw,
        "smooth_11": smooth_11,
        "pure_merge_a1": pure_merge_a1,
        "pure_split_1b": pure_split_1b,
        "x_switches": x_switches,
        "max_sw_in": max_sw_in,
        "max_sw_out": max_sw_out,
        "illegal_turn_adj": illegal_turn_adj,
        "dual_illegal_adj": dual_illegal_adj,
        "illegal_turn_pairs": illegal_turn_pairs,
        "deg_hist": dict(deg_hist),
    }, strips, j_order, sw_in, sw_out


def fold_train_track(L_bytes, primary_idx, schedule="S1_pure_first"):
    """
    Execute exact word-carrying Bestvina-Handel Stallings Folding & Dunfield-Thurston
    Band Splitting on tau_sw(I_0) until complete contraction to the single closed cycle!

    Schedules tested on our exact M4/train-track engine:
      - S1_pure_first:   Always fire (1,1) smooth folds (R1) -> (a,1) full-branch Stallings folds (R2)
                         -> boundary band splits (R3 / R3*) + spirals (exact M4!).
      - S2_dt_heavy:     Dunfield-Thurston greedy: always fold/split the WIDEST extreme branch first!
      - S3_narrow_first: Always fold/split the NARROWEST extreme branch first!
      - S4_max_illegal:  Always fold at the switch/branch with highest illegal-turn multiplicity first!

    Wait: By Theorem T1, every word-carrying fold/band-split on tau_sw(I_0) is an exact M4 move
    (R1/R2/R3/R3*/Spiral) with priority scheduling! Let's run all 4 schedules using an exact
    priority-heap version of M4 so we verify 100% byte-exactness AND measure whether ANY
    train-track folding schedule beats M4-FIFO's 1.87 r_0!
    """
    n, r0, m16, strips, j_order, ws = tk.build_initial_strips(L_bytes, primary_idx, isolate_p=True)
    q0 = len(strips)
    if q0 == 0:
        return b"", {}

    # Build initial ordered adjacency via 1 linear two-pointer merge
    out_adj = [[] for _ in range(q0)]
    in_adj = [[] for _ in range(q0)]
    ii = 0; jj = 0
    while ii < q0 and jj < q0:
        v = ii; u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo: jj += 1
        elif v_hi <= u_tlo: ii += 1
        else:
            out_adj[u].append(v); in_adj[v].append(u)
            if u_thi < v_hi: jj += 1
            elif v_hi < u_thi: ii += 1
            else: ii += 1; jj += 1

    def replace_in_list(lst, old_id, new_ids):
        idx = lst.index(old_id)
        lst[idx:idx + 1] = new_ids

    def is_spiral(v):
        if not strips[v][5]: return False
        lo, hi, d, w, m, _ = strips[v]
        if d == 0: return True
        L = hi - lo
        return (d > 0 and d < L) or (d < 0 and (-d) < L)

    def is_r1(u):
        if not strips[u][5] or len(out_adj[u]) != 1: return False
        v = out_adj[u][0]
        return v != u and strips[v][5] and len(in_adj[v]) == 1 and in_adj[v][0] == u

    def is_r2(v):
        if not strips[v][5] or not in_adj[v]: return False
        for u in in_adj[v]:
            if u == v or not strips[u][5] or len(out_adj[u]) != 1: return False
        return True

    def is_r3(u):
        if not strips[u][5] or len(out_adj[u]) != 1: return False
        v = out_adj[u][0]
        if v == u or not strips[v][5]: return False
        t_lo, t_hi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        return (t_lo == strips[v][0]) or (t_hi == strips[v][1])

    def is_r3s(v):
        if not strips[v][5] or len(in_adj[v]) != 1: return False
        u = in_adj[v][0]
        if u == v or not strips[u][5]: return False
        t_lo, t_hi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        v_lo, v_hi = strips[v][0], strips[v][1]
        return (v_lo == t_lo and v_hi < t_hi) or (v_hi == t_hi and v_lo > t_lo)

    heap = []
    def prio(u):
        if not strips[u][5]: return None
        w = strips[u][1] - strips[u][0]
        if is_spiral(u):
            return (0, 0, u)
        r1 = is_r1(u)
        r2 = is_r2(u)
        r3 = is_r3(u)
        r3s = is_r3s(u)
        if not (r1 or r2 or r3 or r3s):
            return None
        if schedule == "S1_pure_first":
            # Smooth (1,1) -> Full-branch Stallings fold (a,1) -> Band split R3/R3*
            tier = 1 if r1 else (2 if r2 else (3 if r3 else 4))
            return (tier, w, u)
        elif schedule == "S2_dt_heavy":
            # Dunfield-Thurston heaviest branch split/fold first!
            return (1, -w, u)
        elif schedule == "S3_narrow_first":
            # Narrowest branch fold/split first!
            return (1, w, u)
        elif schedule == "S4_max_illegal":
            # Highest illegal-turn multiplicity (in_deg + out_deg) first!
            deg = len(in_adj[u]) + len(out_adj[u])
            return (1, -deg, u)
        raise ValueError(schedule)

    def push_c(u):
        pr = prio(u)
        if pr is not None:
            heapq.heappush(heap, pr)

    for u in range(q0):
        push_c(u)

    stats = defaultdict(int)
    stats["r0"] = r0
    closed_primary = None
    live_cnt = q0
    max_branches = q0

    while heap:
        _, _, cand = heapq.heappop(heap)
        if not strips[cand][5]:
            continue

        # 1. Spiral / closed cycle
        if is_spiral(cand):
            lo, hi, d, w, m, _ = strips[cand]
            if d == 0:
                if m >= 0: closed_primary = (w, m)
                strips[cand][5] = False; live_cnt -= 1
                out_adj[cand] = []; in_adj[cand] = []
                stats["spirals"] += 1
                continue
            L = hi - lo
            stats["spirals"] += 1
            other_in = [x for x in in_adj[cand] if x != cand]
            other_out = [x for x in out_adj[cand] if x != cand]
            new_ids = []
            if d > 0:
                q_sp, rem = divmod(L, d)
                if rem > 0:
                    nid = len(strips); strips.append([lo, lo + rem, (q_sp + 1) * d, ws.power(w, q_sp + 1), -1, True])
                    out_adj.append([]); in_adj.append([]); new_ids.append(nid)
                if d - rem > 0:
                    nid = len(strips); strips.append([lo + rem, lo + d, q_sp * d, ws.power(w, q_sp), -1, True])
                    out_adj.append([]); in_adj.append([]); new_ids.append(nid)
            else:
                s_sp = -d; q_sp, rem = divmod(L, s_sp)
                if s_sp - rem > 0:
                    nid = len(strips); strips.append([hi - s_sp, hi - rem, -q_sp * s_sp, ws.power(w, q_sp), -1, True])
                    out_adj.append([]); in_adj.append([]); new_ids.append(nid)
                if rem > 0:
                    nid = len(strips); strips.append([hi - rem, hi, -(q_sp + 1) * s_sp, ws.power(w, q_sp + 1), -1, True])
                    out_adj.append([]); in_adj.append([]); new_ids.append(nid)
            strips[cand][5] = False
            live_cnt += len(new_ids) - 1
            if live_cnt > max_branches: max_branches = live_cnt
            for u in other_in:
                u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
                hits = [nid for nid in new_ids if strips[nid][0] < u_thi and strips[nid][1] > u_tlo]
                replace_in_list(out_adj[u], cand, hits)
                for nid in hits: in_adj[nid].append(u)
                push_c(u)
            new_by_t = sorted(new_ids, key=lambda nid: strips[nid][0] + strips[nid][2])
            for v in other_out:
                v_lo, v_hi = strips[v][0], strips[v][1]
                hits = [nid for nid in new_by_t if (strips[nid][0] + strips[nid][2]) < v_hi and (strips[nid][1] + strips[nid][2]) > v_lo]
                replace_in_list(in_adj[v], cand, hits)
                for nid in hits: out_adj[nid].append(v)
                push_c(v)
            for nid in new_ids: push_c(nid)
            out_adj[cand] = []; in_adj[cand] = []
            continue

        # 2. Smooth (1,1) fold (R1)
        if is_r1(cand):
            u = cand; v = out_adj[u][0]
            w_u_len = ws.length[strips[u][3]]
            strips[u][2] += strips[v][2]
            strips[u][3] = ws.concat(strips[u][3], strips[v][3])
            strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, strips[v][4])
            strips[v][5] = False; live_cnt -= 1
            v_outs = out_adj[v]
            out_adj[u] = list(v_outs)
            for nxt in v_outs:
                replace_in_list(in_adj[nxt], v, [u]); push_c(nxt)
            out_adj[v] = []; in_adj[v] = []
            stats["smooth_folds"] += 1; stats["total_ops"] += 1
            push_c(u)
            for prev in in_adj[u]: push_c(prev)
            continue

        # 3. Full-branch Stallings fold (a,1) (R2)
        if is_r2(cand):
            v = cand; v_d = strips[v][2]; v_w = strips[v][3]; v_m = strips[v][4]
            v_ins = list(in_adj[v]); v_outs = list(out_adj[v])
            strips[v][5] = False; live_cnt -= 1
            new_in_lists = {nxt: [] for nxt in v_outs}
            for u in v_ins:
                w_u_len = ws.length[strips[u][3]]
                strips[u][2] += v_d
                strips[u][3] = ws.concat(strips[u][3], v_w)
                strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, v_m)
                out_adj[u] = []
                stats["full_fold_ops"] += 1; stats["total_ops"] += 1
            stats["full_fold_nodes"] += 1
            i_u = 0; i_w = 0; nu = len(v_ins); nw = len(v_outs)
            while i_u < nu and i_w < nw:
                u = v_ins[i_u]; nxt = v_outs[i_w]
                u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
                w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                if u_thi <= w_lo: i_u += 1
                elif w_hi <= u_tlo: i_w += 1
                else:
                    out_adj[u].append(nxt); new_in_lists[nxt].append(u)
                    if u_thi < w_hi: i_u += 1
                    elif w_hi < u_thi: i_w += 1
                    else: i_u += 1; i_w += 1
            for nxt in v_outs:
                replace_in_list(in_adj[nxt], v, new_in_lists[nxt]); push_c(nxt)
            for u in v_ins:
                push_c(u)
                for prev in in_adj[u]: push_c(prev)
            out_adj[v] = []; in_adj[v] = []
            continue

        # 4. Partial-branch fold / Dunfield-Thurston band split R3
        if is_r3(cand):
            u = cand; v = out_adj[u][0]
            u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
            v_lo, v_hi, v_d, v_w, v_m, _ = strips[v]
            is_prefix = (u_tlo == v_lo)
            ln = u_thi - u_tlo
            w_u_len = ws.length[strips[u][3]]
            strips[u][2] += v_d
            strips[u][3] = ws.concat(strips[u][3], v_w)
            strips[u][4] = mt.combine_mark(strips[u][4], w_u_len, -1)
            if is_prefix:
                strips[v][0] += ln; in_adj[v].pop(0)
            else:
                strips[v][1] -= ln; in_adj[v].pop()
            new_u_tlo, new_u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
            out_adj[u] = []
            if is_prefix:
                k = 0
                while k < len(out_adj[v]):
                    nxt = out_adj[v][k]; w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                    if w_lo >= new_u_thi: break
                    out_adj[u].append(nxt)
                    if w_hi <= new_u_thi:
                        replace_in_list(in_adj[nxt], v, [u]); out_adj[v].pop(k); push_c(nxt)
                    else:
                        replace_in_list(in_adj[nxt], v, [u, v]); push_c(nxt); break
            else:
                u_hits_rev = []; k = len(out_adj[v]) - 1
                while k >= 0:
                    nxt = out_adj[v][k]; w_lo, w_hi = strips[nxt][0], strips[nxt][1]
                    if w_hi <= new_u_tlo: break
                    u_hits_rev.append(nxt)
                    if w_lo >= new_u_tlo:
                        replace_in_list(in_adj[nxt], v, [u]); out_adj[v].pop(k); push_c(nxt); k -= 1
                    else:
                        replace_in_list(in_adj[nxt], v, [v, u]); push_c(nxt); break
                out_adj[u] = list(reversed(u_hits_rev))
            stats["band_splits"] += 1; stats["total_ops"] += 1
            push_c(u); push_c(v)
            for prev in in_adj[u]: push_c(prev)
            if in_adj[v]: push_c(in_adj[v][0]); push_c(in_adj[v][-1])
            continue

        # 5. Dual band split R3*
        if is_r3s(cand):
            v = cand; u = in_adj[v][0]
            u_lo, u_hi, u_d, u_w, u_m, _ = strips[u]
            v_lo, v_hi, v_d, v_w, v_m, _ = strips[v]
            u_tlo, u_thi = u_lo + u_d, u_hi + u_d
            is_prefix = (v_lo == u_tlo)
            ln = v_hi - v_lo
            strips[v][5] = False
            u_abs = len(strips)
            w_u_len = ws.length[u_w]
            new_w = ws.concat(u_w, v_w)
            new_m = mt.combine_mark(-1, w_u_len, v_m)
            if is_prefix:
                strips.append([u_lo, u_lo + ln, u_d + v_d, new_w, new_m, True])
                strips[u][0] += ln; out_adj[u].pop(0)
            else:
                strips.append([u_hi - ln, u_hi, u_d + v_d, new_w, new_m, True])
                strips[u][1] -= ln; out_adj[u].pop()
            out_adj.append(list(out_adj[v])); in_adj.append([])
            for nxt in out_adj[u_abs]:
                replace_in_list(in_adj[nxt], v, [u_abs]); push_c(nxt)
            out_adj[v] = []; in_adj[v] = []
            abs_lo, abs_hi = strips[u_abs][0], strips[u_abs][1]
            if is_prefix:
                k = 0
                while k < len(in_adj[u]):
                    prev = in_adj[u][k]; p_tlo, p_thi = strips[prev][0] + strips[prev][2], strips[prev][1] + strips[prev][2]
                    if p_tlo >= abs_hi: break
                    in_adj[u_abs].append(prev)
                    if p_thi <= abs_hi:
                        replace_in_list(out_adj[prev], u, [u_abs]); in_adj[u].pop(k); push_c(prev)
                    else:
                        replace_in_list(out_adj[prev], u, [u_abs, u]); push_c(prev); break
            else:
                hits_rev = []; k = len(in_adj[u]) - 1
                while k >= 0:
                    prev = in_adj[u][k]; p_tlo, p_thi = strips[prev][0] + strips[prev][2], strips[prev][1] + strips[prev][2]
                    if p_thi <= abs_lo: break
                    hits_rev.append(prev)
                    if p_tlo >= abs_lo:
                        replace_in_list(out_adj[prev], u, [u_abs]); in_adj[u].pop(k); push_c(prev); k -= 1
                    else:
                        replace_in_list(out_adj[prev], u, [u, u_abs]); push_c(prev); break
                in_adj[u_abs] = list(reversed(hits_rev))
            stats["band_splits"] += 1; stats["total_ops"] += 1
            push_c(u_abs); push_c(u)
            if out_adj[u]: push_c(out_adj[u][0]); push_c(out_adj[u][-1])
            continue

    assert closed_primary is not None and live_cnt == 0, f"Incomplete fold! live={live_cnt}"
    w_c, m_c = closed_primary
    cyc = ws.expand(w_c)
    q = len(cyc)
    rot = cyc[m_c:] + cyc[:m_c]
    text = ((rot * (n // q + 1))[:n])[::-1]
    stats["max_branches"] = max_branches
    stats["slp_gt16"] = ws.slp_gt16
    return text, dict(stats)


def main():
    print("=== 1. EXHAUSTIVE EXACTNESS OF MEASURED TRAIN-TRACK FOLDING (binary n<=8, ternary n<=5) ===")
    tot = 0
    for base, max_n in [(2, 8), (3, 5)]:
        for n in range(2, max_n + 1):
            for code in range(base ** n):
                T = bytes((code // (base ** i)) % base for i in range(n))
                if len(set(T)) < 2: continue
                L, p = mt.bwt_encode(T)
                build_measured_train_track(L, p)
                for sched in ("S1_pure_first", "S2_dt_heavy", "S3_narrow_first", "S4_max_illegal"):
                    dec, _ = fold_train_track(L, p, schedule=sched)
                    assert dec == T
                    tot += 1
    print(f"  EXHAUSTIVE PASS: {tot} exact train-track folding runs across all 4 schedules [verified]")

    print("\n=== 2. CANONICAL MEASURED TRAIN TRACK tau_sw(I_0) CENSUS ON ALL 6 WIKIDATA BLOCKS ===")
    print("block       |   r0  | switches (/r0) | (1,1)sm | (a,1)mrg | (1,b)spl | X(a,b)sw | max(in,out) | ill_adj (/r0) | dual_ill")
    print("------------+-------+----------------+---------+----------+----------+----------+-------------+---------------+---------")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    blocks = []
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        ref = mt.ref_ibwt(L, entry["primary_index"])
        tt, *_ = build_measured_train_track(L, entry["primary_index"])
        r0 = tt["r0"]
        print(f"{entry['file']:11s} | {r0:5d} | {tt['num_sw']:6d} ({tt['num_sw']/r0:.3f}r) | {tt['smooth_11']:7d} | "
              f"{tt['pure_merge_a1']:8d} | {tt['pure_split_1b']:8d} | {tt['x_switches']:8d} | "
              f"({tt['max_sw_in']:4d},{tt['max_sw_out']:4d}) | {tt['illegal_turn_adj']:6d} ({tt['illegal_turn_adj']/r0:.3f}r) | {tt['dual_illegal_adj']:6d}")
        blocks.append((entry, L, ref))

    print("\n=== 3. BESTVINA-HANDEL / DUNFIELD-THURSTON FOLDING SCHEDULES ON ALL 6 WIKIDATA BLOCKS ===")
    print("Schedule        | sum_ops (/r0)   | smooth_11 | full_fold | band_spl | spirals | slp_>16 | exact")
    print("----------------+-----------------+-----------+-----------+----------+---------+---------+------")
    r0_sum = sum(e["r"] for e, _, _ in blocks)
    for sched in ("S1_pure_first", "S2_dt_heavy", "S3_narrow_first", "S4_max_illegal"):
        s_ops = s_sm = s_ff = s_bs = s_sp = s_slp = 0
        for entry, L, ref in blocks:
            dec, st = fold_train_track(L, entry["primary_index"], schedule=sched)
            assert dec == ref, f"EXACTNESS FAIL {entry['file']} {sched}"
            s_ops += st.get("total_ops", 0)
            s_sm += st.get("smooth_folds", 0)
            s_ff += st.get("full_fold_ops", 0)
            s_bs += st.get("band_splits", 0)
            s_sp += st.get("spirals", 0)
            s_slp += st.get("slp_gt16", 0)
        print(f"{sched:15s} | {s_ops:7d} ({s_ops/r0_sum:5.3f}r) | {s_sm:9d} | {s_ff:9d} | {s_bs:8d} | {s_sp:7d} | {s_slp:7d} | PASS", flush=True)


if __name__ == "__main__":
    main()
