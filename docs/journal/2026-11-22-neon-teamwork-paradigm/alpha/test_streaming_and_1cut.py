#!/usr/bin/env python3
"""
Round 2 Quantitative Investigation:
  Part A: Exact Q=0 Radix-Streaming / External-Memory Lazard-BWT measurements on all 6 Wikidata blocks
          (passes p, total runs & bytes streamed, working memory M, with L1D cursor cap B_L1 in {64, 256, inf}).
  Part B: Split-Free Lazard x BDMR Hybrid: 1-Cut Split-Free Recompression (1Cut-BWT, kappa=1, ZERO bucket splits!)
          - Exhaustively verified on binary n<=8 (all p), ternary n<=5 (all p), and all 6 Wikidata blocks!
          - Measures: rounds D, sum r_t, max_r/r_0, peak sigma_t, tot_syms, SLP height h,
            total streaming bytes (Q=0 streaming model), and Q_indep (table-lookup model)!
"""

import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/teamwork-a")
from verify_full_and_falsify import bwt_encode, bwt_decode_reference


# =============================================================================
# PART A: Exact Q=0 Streaming Lazard-BWT (flat sequential passes)
# =============================================================================
def measure_streaming_lazard(L: bytes, cursor_cap: int = 256):
    """
    Simulates exact Q=0 streaming Lazard-BWT where each round:
      - Takes maximal independent prefix S_L and suffix S_R with |S_L| + |S_R| <= cursor_cap
      - Pass 1: sequentially reads F_{S_L} (prefix) and F_{S_R} (suffix), resolves self-loops sequentially
      - Pass 2: sequentially reads remaining L_t, splices from <= cursor_cap L1D cursors, writes L_{t+1}
    Uses exact flat list of (sym, length) runs (no linked list!).
    Also counts exact bytes streamed assuming 8 bytes/run (sym:u32, len:u32).
    """
    n = len(L)
    # Initial maximal runs
    runs = []
    sym_freq = defaultdict(int)
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        runs.append((c, j - i))
        sym_freq[c] += (j - i)
        i = j
    r0 = len(runs)

    # Maintain active ordered symbols as a Python list of sym IDs (in sorted lex order)
    # Wait: maintaining rank map for fast lookup in simulation
    active_syms = sorted(sym_freq.keys())
    next_sym = 256

    rounds = 0
    total_runs_read = 0      # sum |runs_t|
    total_runs_written = 0   # sum |runs_{t+1}|
    total_fx_runs_streamed = 0  # sum of F_S runs read+written to exit_buf
    max_exit_buf_runs = 0
    max_active_syms = len(active_syms)
    max_live_runs = r0

    while len(active_syms) > 0 and len(runs) > 0:
        # Strip zero-freq at ends
        while active_syms and sym_freq[active_syms[0]] == 0:
            active_syms.pop(0)
        while active_syms and sym_freq[active_syms[-1]] == 0:
            active_syms.pop()
        if not active_syms:
            break
        if len(active_syms) == 1:
            # Pure single-symbol cycle! Done!
            break

        # Greedily extend S_L from front, S_R from back, subject to |S_L| + |S_R| <= cursor_cap
        S_L = []
        S_R = []
        S_set = set()
        forbidden = set()

        # Left prefix scan
        idx_r = 0
        off_r = 0
        a_idx = 0
        while a_idx < len(active_syms) and len(S_L) < cursor_cap:
            cand = active_syms[a_idx]
            fc = sym_freq[cand]
            if fc == 0:
                a_idx += 1
                continue
            if cand in forbidden or cand in S_set:
                break
            rem = fc
            ti, to = idx_r, off_r
            csyms = set()
            conflict = False
            while rem > 0:
                s, ln = runs[ti]
                avail = ln - to
                if s in S_set:
                    conflict = True
                    break
                if s != cand:
                    csyms.add(s)
                if avail <= rem:
                    rem -= avail
                    ti += 1
                    to = 0
                else:
                    to += rem
                    rem = 0
            if conflict:
                break
            S_L.append(cand)
            S_set.add(cand)
            forbidden.update(csyms)
            idx_r, off_r = ti, to
            a_idx += 1

        # Right suffix scan
        idx_b = len(runs) - 1
        off_b = 0  # from right end of runs[idx_b]
        b_idx = len(active_syms) - 1
        while b_idx >= a_idx and len(S_L) + len(S_R) < cursor_cap:
            cand = active_syms[b_idx]
            fc = sym_freq[cand]
            if fc == 0:
                b_idx -= 1
                continue
            if cand in forbidden or cand in S_set:
                break
            rem = fc
            ti, to = idx_b, off_b
            csyms = set()
            conflict = False
            while rem > 0:
                s, ln = runs[ti]
                avail = ln - to
                if s in S_set:
                    conflict = True
                    break
                if s != cand:
                    csyms.add(s)
                if avail <= rem:
                    rem -= avail
                    ti -= 1
                    to = 0
                else:
                    to += rem
                    rem = 0
            if conflict:
                break
            S_R.append(cand)
            S_set.add(cand)
            forbidden.update(csyms)
            idx_b, off_b = ti, to
            b_idx -= 1

        assert len(S_L) + len(S_R) >= 1
        rounds += 1
        total_runs_read += len(runs)
        if len(runs) > max_live_runs:
            max_live_runs = len(runs)
        if len(active_syms) > max_active_syms:
            max_active_syms = len(active_syms)

        # Pop prefix runs for S_L and build exit streams
        exit_streams = {}  # x -> list of (new_sid, length)
        insert_before = defaultdict(list)  # y -> list of new_sids (in lex order!)
        insert_after = defaultdict(list)   # y -> list of new_sids (in lex order!)

        # Consume left prefix of `runs`
        left_ptr = 0
        left_off = 0
        round_fx_runs = 0
        for x in S_L:
            fx = sym_freq[x]
            fx_runs = []
            rem = fx
            while rem > 0:
                s, ln = runs[left_ptr]
                avail = ln - left_off
                if avail <= rem:
                    fx_runs.append((s, avail))
                    rem -= avail
                    left_ptr += 1
                    left_off = 0
                else:
                    fx_runs.append((s, rem))
                    left_off += rem
                    rem = 0
            round_fx_runs += len(fx_runs)
            if fx_runs and fx_runs[0][0] == x:
                fx_runs.pop(0)
            if not fx_runs:
                exit_streams[x] = []
                continue
            # Queue scan
            ex = []
            qi = 0
            qo = 0
            for s, ln in fx_runs:
                if s != x:
                    ex.append((s, 1, ln))
                else:
                    rl = ln
                    while rl > 0:
                        ey, ek, eln = ex[qi]
                        av = eln - qo
                        tk = rl if rl < av else av
                        ex.append((ey, ek + 1, tk))
                        rl -= tk
                        qo += tk
                        if qo == eln:
                            qi += 1
                            qo = 0
            # Suffix of ex from (qi, qo)
            suf = []
            y_ks = defaultdict(set)
            for idx_e in range(qi, len(ex)):
                ey, ek, eln = ex[idx_e]
                st_o = qo if idx_e == qi else 0
                if eln > st_o:
                    suf.append((ey, ek, eln - st_o))
                    y_ks[ey].add(ek)
            mp = {}
            for ey, ks in y_ks.items():
                for ek in sorted(ks, reverse=True):
                    sid = next_sym
                    next_sym += 1
                    mp[(ey, ek)] = sid
                    insert_before[ey].append(sid)
            stream = []
            for ey, ek, ln in suf:
                sid = mp[(ey, ek)]
                stream.append((sid, ln))
                sym_freq[sid] += ln
                sym_freq[ey] -= ln
            exit_streams[x] = stream

        # Consume right suffix of `runs`
        right_ptr = len(runs) - 1
        right_off = 0
        for x in S_R:
            fx = sym_freq[x]
            fx_rev = []
            rem = fx
            while rem > 0:
                s, ln = runs[right_ptr]
                avail = ln - right_off
                if avail <= rem:
                    fx_rev.append((s, avail))
                    rem -= avail
                    right_ptr -= 1
                    right_off = 0
                else:
                    fx_rev.append((s, rem))
                    right_off += rem
                    rem = 0
            round_fx_runs += len(fx_rev)
            if fx_rev and fx_rev[0][0] == x:
                fx_rev.pop(0)
            if not fx_rev:
                exit_streams[x] = []
                continue
            ex = []
            qi = 0
            qo = 0
            for s, ln in fx_rev:
                if s != x:
                    ex.append((s, 1, ln))
                else:
                    rl = ln
                    while rl > 0:
                        ey, ek, eln = ex[qi]
                        av = eln - qo
                        tk = rl if rl < av else av
                        ex.append((ey, ek + 1, tk))
                        rl -= tk
                        qo += tk
                        if qo == eln:
                            qi += 1
                            qo = 0
            suf = []
            y_ks = defaultdict(set)
            for idx_e in range(qi, len(ex)):
                ey, ek, eln = ex[idx_e]
                st_o = qo if idx_e == qi else 0
                if eln > st_o:
                    suf.append((ey, ek, eln - st_o))
                    y_ks[ey].add(ek)
            suf.reverse()
            mp = {}
            for ey, ks in y_ks.items():
                # For max: y < y*x^1 < y*x^2 ..., and as we process S_R from max downward,
                # larger z in S_R comes AFTER smaller z!
                # Wait: since S_R is [z_1 > z_2 > ...], z_1's creations go AFTER z_2's creations!
                # So we prepend to insert_after[ey] in order [y*z_b^1..k, ..., y*z_1^1..k]!
                sids_for_z = []
                for ek in sorted(ks):
                    sid = next_sym
                    next_sym += 1
                    mp[(ey, ek)] = sid
                    sids_for_z.append(sid)
                insert_after[ey] = sids_for_z + insert_after[ey]
            stream = []
            for ey, ek, ln in suf:
                sid = mp[(ey, ek)]
                stream.append((sid, ln))
                sym_freq[sid] += ln
                sym_freq[ey] -= ln
            exit_streams[x] = stream

        exit_tot = sum(len(v) for v in exit_streams.values())
        if exit_tot > max_exit_buf_runs:
            max_exit_buf_runs = exit_tot
        total_fx_runs_streamed += round_fx_runs + exit_tot

        # Sequential pass over remaining U (from (left_ptr, left_off) to (right_ptr, right_off))
        cursors = {x: [0, 0] for x in S_set}
        new_runs = []
        # Extract middle slice of runs
        if left_ptr <= right_ptr:
            for idx in range(left_ptr, right_ptr + 1):
                s, ln = runs[idx]
                st_cut = left_off if idx == left_ptr else 0
                en_cut = right_off if idx == right_ptr else 0
                eff_len = ln - st_cut - en_cut
                if eff_len <= 0:
                    continue
                if s not in S_set:
                    if new_runs and new_runs[-1][0] == s:
                        new_runs[-1] = (s, new_runs[-1][1] + eff_len)
                    else:
                        new_runs.append((s, eff_len))
                else:
                    # Pull eff_len from exit_streams[s] sequentially!
                    st_arr = exit_streams[s]
                    cur = cursors[s]
                    rem = eff_len
                    while rem > 0:
                        ns, nln = st_arr[cur[0]]
                        av = nln - cur[1]
                        tk = rem if rem < av else av
                        if new_runs and new_runs[-1][0] == ns:
                            new_runs[-1] = (ns, new_runs[-1][1] + tk)
                        else:
                            new_runs.append((ns, tk))
                        rem -= tk
                        cur[1] += tk
                        if cur[1] == nln:
                            cur[0] += 1
                            cur[1] = 0

        # Rebuild active_syms in exact sorted order
        new_active = []
        for s in active_syms:
            if s in S_set:
                del sym_freq[s]
                continue
            if s in insert_before:
                new_active.extend(insert_before[s])
            if sym_freq[s] > 0:
                new_active.append(s)
            if s in insert_after:
                new_active.extend(insert_after[s])
        active_syms = new_active
        runs = new_runs
        total_runs_written += len(runs)

    # Each run = 8 bytes (sym:u32, len:u32). Total bytes streamed = 8 * (read + written + fx_exit)
    bytes_streamed = 8 * (total_runs_read + total_runs_written + total_fx_runs_streamed)
    return {
        "r0": r0,
        "rounds": rounds,
        "passes": 2 * rounds,  # Pass 1: F_S -> exit_buf; Pass 2: U -> L_{t+1}
        "sum_runs_read": total_runs_read,
        "sum_runs_written": total_runs_written,
        "bytes_streamed_MB": bytes_streamed / 1e6,
        "max_live_runs": max_live_runs,
        "max_exit_buf_runs": max_exit_buf_runs,
        "max_active_syms": max_active_syms,
    }


# =============================================================================
# PART B: 1-Cut Split-Free Recompression (1Cut-BWT: kappa=1, ZERO bucket splits!)
# =============================================================================
# Let's prove & verify exactness first!
# Given current cyclic BWT L (as runs) over ordered alphabet Sigma = [s_0 < s_1 < ... < s_{sigma-1}]:
# Step 1 (optional self-run check):
#   Wait: if self-loops (L[j] == F[j]) exceed a threshold (or if max non-self 1-cut shrinks by < 5%),
#   we can eliminate min(Sigma) or max(Sigma) via Lazard (which contracts all self-runs of that extreme
#   symbol cleanly with ZERO bucket splits and Q=0!).
#   EVEN BETTER: if we do 1-Cut on Sigma = A U B where A = [0..cut) and B = [cut..sigma),
#   choose orientation (Sigma_L, Sigma_R) = (A, B) or (B, A) maximizing |D| = #{j : L[j] in Sigma_L, F[j] in Sigma_R}!
#   Notice: EVERY edge in D connects two DISTINCT symbols a in Sigma_L, b in Sigma_R (so NO self-loops in D!!).
#   How do we execute a 1-Cut round with ZERO bucket splits AND exact primary mark tracking?!
#
#   Trace carefully:
#   - Partition Sigma into Sigma_L and Sigma_R (either Sigma_L = prefix [0..cut) and Sigma_R = suffix [cut..sigma),
#     OR Sigma_L = suffix [cut..sigma) and Sigma_R = prefix [0..cut)).
#   - Because BOTH Sigma_L and Sigma_R are CONTIGUOUS intervals:
#       Case 1: Sigma_L = [0..cut), Sigma_R = [cut..sigma).
#         - Every row j in F_{Sigma_R} (= suffix of L) with L[j] = a in Sigma_L is in D!
#           We delete j from F_{Sigma_R}, and at i = FL(j) (which has L[i] = b = F[j] in Sigma_R),
#           we replace b by the new symbol (a, b) representing w(a) + w(b)!
#         - Where does (a, b) sit in the new sorted alphabet?
#           In F_a, rows j in F_{Sigma_L} (= [0..cut)) come BEFORE rows j in F_{Sigma_R} (= [cut..sigma)),
#           and within F_{Sigma_R}, rows are ordered by b = F[j] in INCREASING order!
#           Therefore, for each a in Sigma_L, the new symbols (a, b) sit IMMEDIATELY AFTER `a`
#           in INCREASING order of `b`!
#           ZERO bucket splits of `a` (all uncontracted `a` stay as a single symbol `a` before all `(a, b)`)!
#           ZERO bucket splits of `b` (all uncontracted `b` in F_b survive in exact order)!
#       Case 2: Sigma_L = [cut..sigma), Sigma_R = [0..cut).
#         - Every row j in F_{Sigma_R} (= prefix of L) with L[j] = a in Sigma_L is in D!
#           In F_a, rows j in F_{Sigma_R} (= [0..cut)) come BEFORE rows j in F_{Sigma_L} (= [cut..sigma)),
#           ordered by b = F[j] in INCREASING order!
#           Therefore, for each a in Sigma_L, the new symbols (a, b) sit IMMEDIATELY BEFORE `a`
#           in INCREASING order of `b`!
#           ZERO bucket splits of `a` OR `b`!
#
#   WAIT A SECOND!! CHECK PRIMARY MARK `p`!!
#   - If `p` is row `j in D` (in F_b, with L[j] = a in Sigma_L):
#     Then `j = LF(i)` where `i` is the row whose `L[i] = b` gets replaced by `(a, b)`!
#     So row `p = j` starts `|w(b)|` bytes BEFORE row `i`!!
#     Therefore we move the mark `p` to row `i` and add `p_byte_shift += sym_len[b]`!!
#   - If `p` is NOT in `D`: `p` survives in place with shift `0`!
#
#   WAIT A SECOND!! CHECK WHAT HAPPENS IF `|D| == 0` (ONLY SELF-LOOPS REMAIN, e.g. T = a^n)!
#   - If `|Sigma| == 1`: we reached the pure single-symbol cycle! Done!
#   - Otherwise, for any cyclic string with `|Sigma| >= 2`, there MUST exist at least one edge `a -> b` with `a != b`,
#     so there MUST exist a cut `c` with `|D| >= 1`! (Because if `a < b`, cut between `a` and `b` has `a -> b` crossing forward;
#     if `a > b`, cut between `b` and `a` has `a -> b` crossing backward!)
#   - **PROOF OF TERMINATION**: For ANY single-cycle string with `|Sigma| >= 2`, `max_cut |D| >= 1`, so `n` STRICTLY DECREASES every round until `|Sigma| == 1`!! `[proved]`!
#   - (And if `|D|` is small due to self-runs `c^k`, e.g. `|D| < 0.05 * n_rem`, we can ALSO fall back to 1-step Lazard on `min`/`max` or self-run contraction!)
#
#   WAIT A SECOND!! HOW DO WE EXECUTE `1Cut-BWT` IN `O(r_t)` PER ROUND?!
#   In round `t`, we have `runs` (list of `(sym, len, has_p, p_off)`).
#   1. Scan `runs` ONCE (prefix-sum over F-buckets!):
#      Since we know `sym_freq[s]` for each active symbol `s` in sorted order, we know which F-bucket `b`
#      each run belongs to!
#      During this 1 scan, for each run `(a, len)` inside `F_b` (with `a != b`):
#        - If `rank[a] < rank[b]`: this edge crosses EVERY cut `c in (rank[a], rank[b]]` in the FORWARD direction (`A -> B`)!
#        - If `rank[a] > rank[b]`: this edge crosses EVERY cut `c in (rank[b], rank[a]]` in the BACKWARD direction (`B -> A`)!
#      Using TWO difference arrays `diff_fwd[0..sigma]` and `diff_bwd[0..sigma]`:
#        - `diff_fwd[rank[a]+1] += len; diff_fwd[rank[b]+1] -= len` (for `rank[a] < rank[b]`)
#        - `diff_bwd[rank[b]+1] += len; diff_bwd[rank[a]+1] -= len` (for `rank[a] > rank[b]`)
#      Prefix-sum `diff_fwd` and `diff_bwd` -> **IN `O(r_t + sigma_t)` EXACT TIME, WE FIND THE GLOBALLY OPTIMAL 1-CUT `c*` AND ORIENTATION MAXIMIZING `|D|`!!**
#   2. Execute the cut `c*`:
#      - Scan the `F_{Sigma_R}` half of `runs` (either suffix or prefix):
#        * Runs with `a in Sigma_L` are deleted from `F_{Sigma_R}` and emitted into `mask[b]` as `(new_sid(a, b), len, p_info)`!
#        * Runs with `a in Sigma_R` SURVIVE in `F_{Sigma_R}` and emit `(b, len, no_p)` into `mask[b]` (unchanged `b`!)!
#      - Scan ALL surviving runs of `L` (both `F_{Sigma_L}` and the surviving runs of `F_{Sigma_R}`):
#        * If `sym in Sigma_L`: copy directly!
#        * If `sym = b in Sigma_R`: pull `len` rows sequentially from `mask[b]`!
#   3. Done! ZERO bucket splits (`kappa = 1`), ZERO pointer chasing on `L`!


def onecut_bwt_decode(L: bytes, p: int, measure: bool = False):
    """
    Exact Inverse BWT via 1-Cut Split-Free Recompression (1Cut-BWT).
    Guaranteed kappa = 1 (ZERO bucket splits of any symbol in Sigma_L or Sigma_R!).
    """
    n = len(L)
    if n == 0:
        return b"", {}

    # Build initial maximal runs
    runs = []  # [sym, length, p_off_or_-1]
    sym_freq = defaultdict(int)
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        po = (p - i) if (i <= p < j) else -1
        runs.append([c, j - i, po])
        sym_freq[c] += (j - i)
        i = j

    r0 = len(runs)
    active_syms = sorted(sym_freq.keys())
    next_sym = 256
    prod = {}  # sid -> (a, b) meaning w(a) + w(b)
    sym_len = {s: 1 for s in active_syms}
    sym_height = {s: 0 for s in active_syms}
    p_byte_shift = 0

    rounds = 0
    sum_rt = 0
    max_rt = r0
    max_sigma = len(active_syms)
    total_syms_created = len(active_syms)
    total_D_rows = 0
    q_indep_mask_reads = 0  # lookups into mask[b] cursor table (table size <= |Sigma_R|)
    max_sigma_R = 0
    sum_rows_rem = 0
    rows_rem = n

    while len(active_syms) > 1:
        sigma = len(active_syms)
        if sigma > max_sigma:
            max_sigma = sigma
        if len(runs) > max_rt:
            max_rt = len(runs)
        sum_rt += len(runs)
        sum_rows_rem += rows_rem

        rank = {s: idx for idx, s in enumerate(active_syms)}

        # Pass 1: find globally optimal 1-cut (cut in 1..sigma-1, dir in {FWD, BWD}) maximizing |D|
        diff_fwd = [0] * (sigma + 1)
        diff_bwd = [0] * (sigma + 1)

        # Walk runs alongside F-buckets
        b_idx = 0
        b_rem = sym_freq[active_syms[0]]
        for a, ln, _ in runs:
            ra = rank[a]
            rem = ln
            while rem > 0:
                tk = rem if rem < b_rem else b_rem
                if ra < b_idx:
                    diff_fwd[ra + 1] += tk
                    diff_fwd[b_idx + 1] -= tk
                elif ra > b_idx:
                    diff_bwd[b_idx + 1] += tk
                    diff_bwd[ra + 1] -= tk
                rem -= tk
                b_rem -= tk
                if b_rem == 0:
                    b_idx += 1
                    if b_idx < sigma:
                        b_rem = sym_freq[active_syms[b_idx]]

        # Prefix sums to find best cut
        best_D = -1
        best_cut = 1
        best_dir = 0  # 0: Sigma_L = [0..cut), Sigma_R = [cut..sigma); 1: Sigma_L = [cut..sigma), Sigma_R = [0..cut)
        cf = 0
        cb = 0
        for c in range(1, sigma):
            cf += diff_fwd[c]
            cb += diff_bwd[c]
            if cf > best_D:
                best_D = cf
                best_cut = c
                best_dir = 0
            if cb > best_D:
                best_D = cb
                best_cut = c
                best_dir = 1

        assert best_D > 0, "No cross-symbol edges left (multi-cycle pure components?)"
        rounds += 1
        total_D_rows += best_D
        rows_rem -= best_D

        if best_dir == 0:
            # Sigma_L = [0..best_cut), Sigma_R = [best_cut..sigma)
            # F_{Sigma_R} is the SUFFIX of runs starting at row offset split_row
            split_row = sum(sym_freq[active_syms[k]] for k in range(best_cut))
            sigR_size = sigma - best_cut
            is_L = lambda r_val: r_val < best_cut
        else:
            # Sigma_L = [best_cut..sigma), Sigma_R = [0..best_cut)
            # F_{Sigma_R} is the PREFIX of runs ending at row offset split_row
            split_row = sum(sym_freq[active_syms[k]] for k in range(best_cut))
            sigR_size = best_cut
            is_L = lambda r_val: r_val >= best_cut

        if sigR_size > max_sigma_R:
            max_sigma_R = sigR_size

        # Pass 2: Split runs into F_{Sigma_L} and F_{Sigma_R}; build mask[b] and surviving runs of F_{Sigma_R}!
        # For every b in Sigma_R:
        #   mask[b] is a run-stream of length sym_freq[b]:
        #     - if L[j] = a in Sigma_L: (new_sid(a, b), len, p_off_or_-1, is_deleted_p=True)
        #     - if L[j] = x in Sigma_R: (b, len, -1, False)  [uncontracted b!]
        #   And the surviving runs of F_b are ONLY the runs with L[j] = x in Sigma_R!
        surviving_runs = []  # runs in L after deleting D (before replacing b in Sigma_R!)
        mask = defaultdict(list)  # b -> list of [new_sym, len, p_off_or_-1, b_len_for_p]
        pair_to_sid = {}
        a_new_children = defaultdict(list)  # a -> list of new_sids (in increasing order of b!)

        b_idx = 0
        b_rem = sym_freq[active_syms[0]]
        for a, ln, po in runs:
            ra = rank[a]
            rem = ln
            cur_in = 0
            while rem > 0:
                b = active_syms[b_idx]
                tk = rem if rem < b_rem else b_rem
                sub_po = (po - cur_in) if (po != -1 and cur_in <= po < cur_in + tk) else -1

                b_in_R = not is_L(b_idx)
                a_in_L = is_L(ra)

                if b_in_R:
                    if a_in_L:
                        # Row is in D! Delete from surviving_runs, emit (a, b) into mask[b]
                        key = (a, b)
                        sid = pair_to_sid.get(key)
                        if sid is None:
                            sid = next_sym
                            next_sym += 1
                            pair_to_sid[key] = sid
                            prod[sid] = (a, b)
                            sym_len[sid] = sym_len[a] + sym_len[b]
                            sym_height[sid] = max(sym_height[a], sym_height[b]) + 1
                            a_new_children[a].append((b_idx, sid))
                            total_syms_created += 1
                        mask[b].append([sid, tk, sub_po, sym_len[b]])
                        sym_freq[sid] += tk
                        sym_freq[a] -= tk
                        sym_freq[b] -= tk
                    else:
                        # b in Sigma_R, a in Sigma_R: SURVIVES! And mask[b] keeps uncontracted `b`
                        mask[b].append([b, tk, -1, 0])
                        if surviving_runs and surviving_runs[-1][0] == a and sub_po == -1 and surviving_runs[-1][2] == -1:
                            surviving_runs[-1][1] += tk
                        else:
                            surviving_runs.append([a, tk, sub_po])
                else:
                    # b in Sigma_L: SURVIVES untouched!
                    if surviving_runs and surviving_runs[-1][0] == a and sub_po == -1 and surviving_runs[-1][2] == -1:
                        surviving_runs[-1][1] += tk
                    else:
                        surviving_runs.append([a, tk, sub_po])

                cur_in += tk
                rem -= tk
                b_rem -= tk
                if b_rem == 0:
                    b_idx += 1
                    if b_idx < sigma:
                        b_rem = sym_freq[active_syms[b_idx]]

        # Pass 3: Scan surviving_runs; replace each run of b in Sigma_R using mask[b] sequentially!
        cursors = {}
        new_runs = []
        for s, ln, po in surviving_runs:
            rs = rank[s]
            if is_L(rs):
                if new_runs and new_runs[-1][0] == s and po == -1 and new_runs[-1][2] == -1:
                    new_runs[-1][1] += ln
                else:
                    new_runs.append([s, ln, po])
            else:
                # s = b in Sigma_R: pull `ln` from mask[s]
                q_indep_mask_reads += 1
                m_arr = mask[s]
                cur = cursors.get(s)
                if cur is None:
                    cur = [0, 0]
                    cursors[s] = cur
                rem = ln
                cur_in = 0
                while rem > 0:
                    ns, mln, mpo, blen = m_arr[cur[0]]
                    av = mln - cur[1]
                    tk = rem if rem < av else av
                    # Check primary mark!
                    out_po = -1
                    if po != -1 and cur_in <= po < cur_in + tk:
                        out_po = po - cur_in
                    elif mpo != -1 and cur[1] <= mpo < cur[1] + tk:
                        out_po = mpo - cur[1]
                        p_byte_shift += blen
                    if new_runs and new_runs[-1][0] == ns and out_po == -1 and new_runs[-1][2] == -1:
                        new_runs[-1][1] += tk
                    else:
                        new_runs.append([ns, tk, out_po])
                    cur_in += tk
                    rem -= tk
                    cur[1] += tk
                    if cur[1] == mln:
                        cur[0] += 1
                        cur[1] = 0

        # Rebuild active_syms in EXACT sorted order (ZERO bucket splits!)
        new_active = []
        for s in active_syms:
            children = a_new_children.get(s)
            if children is not None:
                children.sort(key=lambda x: x[0])  # increasing order of b!
                child_sids = [sid for _, sid in children]
                if best_dir == 1:
                    # Sigma_R < Sigma_L: (a, b) goes BEFORE a!
                    new_active.extend(child_sids)
                    if sym_freq[s] > 0:
                        new_active.append(s)
                else:
                    # Sigma_L < Sigma_R: (a, b) goes AFTER a!
                    if sym_freq[s] > 0:
                        new_active.append(s)
                    new_active.extend(child_sids)
            else:
                if sym_freq[s] > 0:
                    new_active.append(s)
        active_syms = new_active
        runs = new_runs

    # Single symbol remains!
    root_sym = active_syms[0]
    root_mult = sym_freq[root_sym]
    L_root = sym_len[root_sym]

    # Expand SLP via first-occurrence memcpy
    buf = bytearray(L_root)
    first_pos = {}
    slp_exp = 0
    slp_cpy = 0
    slp_cpy_bytes = 0
    pos = 0
    stack = [root_sym]
    while stack:
        s = stack.pop()
        if s < 256:
            buf[pos] = s
            pos += 1
        elif s in first_pos:
            src = first_pos[s]
            slen = sym_len[s]
            buf[pos:pos + slen] = buf[src:src + slen]
            pos += slen
            slp_cpy += 1
            slp_cpy_bytes += slen
        else:
            first_pos[s] = pos
            slp_exp += 1
            a, b = prod[s]
            stack.append(b)
            stack.append(a)
    assert pos == L_root

    shift = (-p_byte_shift) % L_root
    unit = bytes(buf[shift:]) + bytes(buf[:shift])
    T_out = unit * root_mult if root_mult > 1 else unit

    stats = {}
    if measure:
        # Streaming bytes (2 reads + 2 writes per round of r_t 8-byte runs, roughly 4 * 8 * sum_rt)
        bytes_streamed_MB = (32 * sum_rt) / 1e6
        stats = {
            "n": n, "r0": r0, "rounds": rounds, "sum_rt": sum_rt, "sum_rt_over_r0": sum_rt / r0,
            "max_rt_over_r0": max_rt / r0, "max_sigma": max_sigma, "max_sigma_R": max_sigma_R,
            "tot_syms": total_syms_created, "slp_height": sym_height[root_sym],
            "q_indep_mask": q_indep_mask_reads, "bytes_streamed_MB": bytes_streamed_MB,
            "slp_exp": slp_exp, "slp_cpy": slp_cpy, "slp_cpy_pct": 100.0 * slp_cpy_bytes / n,
        }
    return T_out, stats


def verify_1cut_exhaustive():
    print("=== EXHAUSTIVE VERIFICATION OF 1Cut-BWT (SPLIT-FREE LAZARD x BDMR) ===")
    c_bin = 0
    for n in range(1, 9):
        for mask in range(1 << n):
            T = bytes(((mask >> i) & 1) + ord('a') for i in range(n))
            L, p = bwt_encode(T)
            for pt in range(n):
                ref = bwt_decode_reference(L, pt)
                got, _ = onecut_bwt_decode(L, pt)
                assert got == ref, f"1Cut bin FAIL n={n} T={T} p={pt}: {ref} vs {got}"
                c_bin += 1
    c_ter = 0
    for n in range(1, 6):
        for code in range(3 ** n):
            tmp = code
            arr = []
            for _ in range(n):
                arr.append((tmp % 3) + ord('a'))
                tmp //= 3
            T = bytes(arr)
            L, p = bwt_encode(T)
            for pt in range(n):
                ref = bwt_decode_reference(L, pt)
                got, _ = onecut_bwt_decode(L, pt)
                assert got == ref, f"1Cut ter FAIL n={n} T={T} p={pt}: {ref} vs {got}"
                c_ter += 1
    print(f"[verified] 1Cut-BWT passed ALL binary n<=8 ({c_bin}) + ternary n<=5 ({c_ter}) = {c_bin + c_ter} cases!\n")


def main():
    verify_1cut_exhaustive()

    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    print("=== PART A: Q=0 STREAMING LAZARD-BWT (cursor_cap=256, L1D-resident cursors) ===")
    for entry in manifest["blocks"]:
        fpath = os.path.join("/tmp/mibz2-blocks", entry["file"])
        with open(fpath, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        st = measure_streaming_lazard(L, cursor_cap=256)
        r0 = st["r0"]
        print(
            f"  {entry['file']:12s} | r={r0:6d} | rounds={st['rounds']:4d} (passes={st['passes']:4d}) | "
            f"sum_r={st['sum_runs_read']/1e6:5.2f}M ({st['sum_runs_read']/r0:5.1f}r) | "
            f"streamed={st['bytes_streamed_MB']:6.1f} MB | max_exit={st['max_exit_buf_runs']:5d} | max_sig={st['max_active_syms']:5d}"
        )

    print("\n=== PART B: 1Cut-BWT (SPLIT-FREE LAZARD x BDMR HYBRID, kappa=1) ON ALL 6 BLOCKS ===")
    for entry in manifest["blocks"]:
        fpath = os.path.join("/tmp/mibz2-blocks", entry["file"])
        with open(fpath, "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        T_ref = bwt_decode_reference(L, p)
        T_got, st = onecut_bwt_decode(L, p, measure=True)
        assert T_got == T_ref, f"1Cut MISMATCH on {entry['file']}!"
        r0 = st["r0"]
        print(
            f"  {entry['file']:12s} | EXACT! rounds={st['rounds']:3d} | sum_rt={st['sum_rt_over_r0']:4.2f}r | "
            f"max_r={st['max_rt_over_r0']:4.2f}r | peak_sig={st['max_sigma']:5d} (sigR={st['max_sigma_R']:5d}) | "
            f"tot_syms={st['tot_syms']/r0:4.2f}r h={st['slp_height']:2d} | "
            f"Q_mask={st['q_indep_mask']:6d} ({st['q_indep_mask']/r0:4.2f}r) | streamed={st['bytes_streamed_MB']:5.1f} MB | cpy={st['slp_cpy_pct']:.1f}%"
        )


if __name__ == "__main__":
    main()
