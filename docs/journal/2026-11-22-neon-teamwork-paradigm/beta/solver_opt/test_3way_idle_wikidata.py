#!/tmp/solver-venv/bin/python
"""
Test the 3-Way Partial Coloring / 'Idle-Thin' Efficiency-Gated BDMR law
(discovered from CP-SAT's Q_con/|D| objective: keep low-frequency / thin symbols IDLE
so sigma_t never explodes and Q_con/|D| stays << 1, then resolve the residual permutation!)
on ALL 6 authenticated Wikidata blocks in /tmp/mibz2-blocks/!
"""
import hashlib
import json
import os
import sys
import time

sys.path.insert(0, "/tmp/teamwork-b")
from deep_analysis import BLOCKS_DIR, runs_from_seq, ref_inverse_bwt


def count_m16(L, p):
    m = 0
    i = 0
    n = len(L)
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        if w <= 16:
            m += w
        elif i <= p < j:
            m += 1
        i = j
    return m


def run_idle_thin_bdmr(L_bytes, p_init, min_count_active=16, max_rounds=12):
    """
    3-Way Partial Coloring BDMR (Sigma_L U Sigma_R U Sigma_Idle):
      - Any symbol s with total frequency count[s] <= min_count_active (or avg run width <= 1.2)
        is placed in Sigma_Idle (NEVER contracted, NEVER split, Q = 0)!
      - Among active symbols (count[s] > min_count_active), pick best zero-split threshold OR
        contiguity-preserving 2-coloring maximizing |D| (deleting ONLY pairs (a, b) with both active)!
      - Stop when active deletion drops below 2% of n_t (or max_rounds), and measure:
        n_res (residual rows to walk!), Q_con (irregular contraction accesses),
        Q_total = Q_con + n_res vs Open's m_16!
    """
    n_orig = len(L_bytes)
    uniq = sorted(set(L_bytes))
    remap = {c: i for i, c in enumerate(uniq)}
    runs = runs_from_seq([remap[b] for b in L_bytes])
    sigma = len(uniq)
    rules = [('base', c) for c in uniq]
    sym_len = [1] * sigma
    sym_to_rule = list(range(sigma))

    n = n_orig
    p = p_init
    p_offset = 0
    q_scatter = 0
    q_stream = 0
    sum_r = 0
    max_sigma = sigma
    rounds = 0

    for rd in range(max_rounds):
        r_cur = len(runs)
        sum_r += r_cur
        if sigma > max_sigma:
            max_sigma = sigma

        counts = [0] * sigma
        run_cnt = [0] * sigma
        for c, w in runs:
            counts[c] += w
            run_cnt[c] += 1
        C = [0] * (sigma + 1)
        tot = 0
        for c in range(sigma):
            C[c] = tot
            tot += counts[c]
        C[sigma] = tot

        # Active mask: symbol s is active iff count[s] > min_count_active
        active = [counts[s] > min_count_active for s in range(sigma)]

        # Build bucket_runs & difference arrays ONLY on active-active non-self edges!
        bucket_runs = [[] for _ in range(sigma)]
        diff_asc = [0] * (sigma + 1)
        diff_desc = [0] * (sigma + 1)
        pos, f_sym = 0, 0
        while f_sym < sigma and C[f_sym + 1] == 0:
            f_sym += 1
        p_L = p_F = p_rank = -1
        seen = [0] * sigma
        for c, w in runs:
            if pos <= p < pos + w:
                p_L = c
                p_rank = seen[c] + (p - pos)
            seen[c] += w
            rem, cur = w, pos
            while rem > 0:
                while C[f_sym + 1] <= cur:
                    f_sym += 1
                take = min(rem, C[f_sym + 1] - cur)
                if cur <= p < cur + take:
                    p_F = f_sym
                bucket_runs[f_sym].append((c, take))
                if active[c] and active[f_sym] and c != f_sym:
                    if c < f_sym:
                        diff_asc[c + 1] += take
                        diff_asc[f_sym + 1] -= take
                    else:
                        diff_desc[f_sym + 1] += take
                        diff_desc[c + 1] -= take
                cur += take
                rem -= take
            pos += w

        # Best threshold on active symbols (zero bucket splits on ALL symbols!)
        # Wait: if idle symbols sit on BOTH sides of theta, could an active a in L have an idle follower on the R side?
        # YES! If we only contract (a, b) with a in active_L, b in active_R, and b not straddled by uncontracted followers of a
        # (using exact contiguity bounds [u_a, v_a]!), it is STILL 100% zero-split AND 100% exact!
        best_th_cnt = 0
        best_dir = 'asc'
        best_th = -1
        acc = 0
        for th in range(1, sigma):
            acc += diff_asc[th]
            if acc > best_th_cnt:
                best_th_cnt = acc; best_dir = 'asc'; best_th = th
        acc = 0
        for th in range(1, sigma):
            acc += diff_desc[th]
            if acc > best_th_cnt:
                best_th_cnt = acc; best_dir = 'desc'; best_th = th

        if best_th_cnt == 0:
            break

        # Define 3-way roles: in_L (active on L side), in_R (active on R side), Idle (not active)
        in_L = [active[s] and (s < best_th if best_dir == 'asc' else s >= best_th) for s in range(sigma)]
        in_R = [active[s] and (not in_L[s]) for s in range(sigma)]

        # Compute exact contiguity bounds [u[a], v[a]] for uncontracted followers (not in_R[b]) of each a in L!
        u = [sigma] * sigma
        v = [-1] * sigma
        for b in range(sigma):
            if not in_R[b]:
                for a, _ in bucket_runs[b]:
                    if in_L[a]:
                        if b < u[a]: u[a] = b
                        if b > v[a]: v[a] = b

        # Count exact deletions AND exact Q_con!
        del_cnt = 0
        round_q_scat = 0
        active_R_touched = [False] * sigma
        for b in range(sigma):
            if in_R[b]:
                for a, w in bucket_runs[b]:
                    if in_L[a] and (b < u[a] or b > v[a]):
                        del_cnt += w
                        round_q_scat += 1
                        active_R_touched[b] = True

        round_q_strm = sum(1 for c, _ in runs if active_R_touched[c])
        round_q = round_q_scat + round_q_strm

        # Stop if this round costs MORE Q than rows deleted (Q_round >= del_cnt)!!
        # Because walking those `del_cnt` rows later costs only 1 load/row!
        if del_cnt == 0 or round_q >= del_cnt:
            break

        q_scatter += round_q_scat
        q_stream += round_q_strm

        # Execute exact zero-split 3-way contraction!
        if in_L[p_L] and in_R[p_F] and (p_F < u[p_L] or p_F > v[p_L]):
            new_p_old = C[p_L] + p_rank
            p_offset += sym_len[sym_to_rule[p_L]]
        else:
            new_p_old = p

        b_idx = [0] * sigma
        b_off = [0] * sigma
        out_runs = []
        out_pos = 0
        new_p = -1
        pos = 0
        f_sym = 0
        while f_sym < sigma and C[f_sym + 1] == 0:
            f_sym += 1

        for c, w in runs:
            end = pos + w
            if in_L[c]:
                uc, vc = u[c], v[c]
                key = (c, uc)
                rem, cur = w, pos
                while rem > 0:
                    while C[f_sym + 1] <= cur:
                        f_sym += 1
                    take = min(rem, C[f_sym + 1] - cur)
                    deleted = in_R[f_sym] and (f_sym < uc or f_sym > vc)
                    if not deleted:
                        if cur <= new_p_old < cur + take:
                            new_p = out_pos + (new_p_old - cur)
                        if out_runs and out_runs[-1][0] == key:
                            out_runs[-1] = (key, out_runs[-1][1] + take)
                        else:
                            out_runs.append((key, take))
                        out_pos += take
                    cur += take
                    rem -= take
            elif in_R[c]:
                if pos <= new_p_old < end:
                    new_p = out_pos + (new_p_old - pos)
                br = bucket_runs[c]
                bi, bo = b_idx[c], b_off[c]
                rem = w
                while rem > 0:
                    a, blen = br[bi]
                    take = min(rem, blen - bo)
                    if in_L[a] and (c < u[a] or c > v[a]):
                        key = (a, c)
                    else:
                        key = (c, 0)
                    if out_runs and out_runs[-1][0] == key:
                        out_runs[-1] = (key, out_runs[-1][1] + take)
                    else:
                        out_runs.append((key, take))
                    out_pos += take
                    rem -= take
                    bo += take
                    if bo == blen:
                        bi += 1; bo = 0
                b_idx[c], b_off[c] = bi, bo
            else:
                # IDLE symbol: NEVER deleted, NEVER split, ZERO bucket reads!!
                if pos <= new_p_old < end:
                    new_p = out_pos + (new_p_old - pos)
                key = (c, 0)
                if out_runs and out_runs[-1][0] == key:
                    out_runs[-1] = (key, out_runs[-1][1] + w)
                else:
                    out_runs.append((key, w))
                out_pos += w
            pos = end

        assert new_p != -1 and out_pos == n - del_cnt
        uniq_keys = sorted(set(k for k, _ in out_runs))
        key_to_id = {k: i for i, k in enumerate(uniq_keys)}
        new_sym_to_rule = []
        for k1, k2 in uniq_keys:
            if (not in_L[k1]) or (k2 == u[k1]):
                new_sym_to_rule.append(sym_to_rule[k1])
            else:
                ra, rb = sym_to_rule[k1], sym_to_rule[k2]
                rid = len(rules)
                rules.append(('pair', ra, rb))
                sym_len.append(sym_len[ra] + sym_len[rb])
                new_sym_to_rule.append(rid)
        runs = [(key_to_id[k], w) for k, w in out_runs]
        sigma = len(uniq_keys)
        sym_to_rule = new_sym_to_rule
        p = new_p
        n -= del_cnt
        rounds += 1

    return {
        "rounds": rounds,
        "n_res": n,
        "r_res": len(runs),
        "max_sigma": max_sigma,
        "rules": len(rules),
        "q_con": q_scatter + q_stream,
        "q_total": q_scatter + q_stream + n,
        "sum_r": sum_r,
    }


def main():
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        man = json.load(f)

    print("=== 3-WAY PARTIAL COLORING ('IDLE-THIN' EFFICIENCY-GATED BDMR) ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    print("Rule: symbols with count <= tau stay IDLE (Q=0); stop when round Q_con >= del_cnt.\n")

    for tau in (16, 64, 256):
        print(f"--- Threshold tau = {tau} (count <= {tau} stays IDLE) ---")
        print(f"  {'Block':12s} | {'m_16':>7s} | {'rnd':>3s} | {'n_res':>7s} ({'vs m16':>6s}) | {'Q_con':>7s} | {'Q_con+n_res':>11s} ({'vs m16':>6s}) | {'max_sig':>7s} | {'rules':>6s}")
        for entry in man["blocks"]:
            path = os.path.join(BLOCKS_DIR, entry["file"])
            L = open(path, "rb").read()
            assert hashlib.sha256(L).hexdigest() == entry["sha256"]
            m16 = count_m16(L, entry["primary_index"])
            st = run_idle_thin_bdmr(L, entry["primary_index"], min_count_active=tau, max_rounds=15)
            print(
                f"  {entry['file']:12s} | {m16:7d} | {st['rounds']:3d} | "
                f"{st['n_res']:7d} ({st['n_res']/m16:5.2f}x) | {st['q_con']:7d} | "
                f"{st['q_total']:11d} ({st['q_total']/m16:5.2f}x) | {st['max_sigma']:7d} | {st['rules']:6d}"
            )
        print()


if __name__ == "__main__":
    main()
