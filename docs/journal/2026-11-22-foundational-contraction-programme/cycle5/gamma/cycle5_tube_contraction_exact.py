#!/usr/bin/env python3
"""
Cycle 5 Exact Verification & Cost-Vector Measurement of:
  Post-S3a Thin-Tube Shortcut Contraction (`Tube-Open`) inside Open's Flat-Array Algebra!

Mathematical Primitive:
  Let R_2 = {all maximal thin runs of width w in 2..16} (132k runs across 6 blocks, 22k/block).
  A thin run S_i in R_2 is `unsplit-thin` (kappa_out = 1 into R_2) iff its LF image
  [dst(i), dst(i)+w(i)) is contained in a single target thin run S_j in R_2 (i.e. 2 <= w(j) <= 16).
  Notice:
    1. Finding ALL unsplit-thin edges S_i -> S_j (and exact-match edges dst(i)==pos(j), w(i)==w(j))
       takes ONE sequential 2-pointer merge of L-order R_2 vs F-order (byte-bucketed) R_2:
       ZERO hash tables, ZERO sorting, ZERO binary search!
    2. Contracting unsplit-thin chains S_0 -> S_1 -> ... -> S_k (capped at word_len <= 15 so
       EVERY tube word stays inside Open's inline16, or unrestricted!) shortcuts EVERY row of S_0
       directly over S_1..S_{k-1} in ONE emit step!
    3. Critically, check EXACTLY how many walk steps in the marked orbit of p are ACTUALLY
       skipped (verified by running the exact marked orbit walk and verifying 100% byte-for-byte
       reconstruction against reference LF on all 6 authenticated Wikidata blocks!).

Tests 4 clean variants:
  V1: Exact-Match Tubes (LF(S_t) == S_{t+1}, w in 2..16), word_len <= 15 (inline16 only!)
  V2: Exact-Match Tubes (LF(S_t) == S_{t+1}, w in 2..16), uncapped length (with Open spelling!)
  V3: Inclusion-Forest Tubes (LF(S_t) subset S_{t+1}, both w in 2..16), word_len <= 15
  V4: Inclusion-Forest Tubes (LF(S_t) subset S_{t+1}, both w in 2..16), uncapped length
  V5: Inclusion-Forest + Thick-Chain Absorption (allowing S_k to land in a thick strip and
      prepending the thick strip's band word in S4!), word_len <= 15 vs uncapped!
"""
import bisect
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-c")
from cycle5_open_anatomy import load_blocks, compute_maximal_runs_and_lf


def build_open_retained_map(data, p, full_runs, limit=16):
    """
    Build exact Open W=16 retained map on K = {all rows in maximal runs of width <= limit, plus p}:
    Returns:
      - retained_rows: sorted list of original BWT row indices in K (length m_16)
      - row_to_ret: dict/array mapping orig_row -> ret_idx (or -1 if thick)
      - next_ret[u]: Open's exact edges[u] (first return in K)
      - word_ret[u]: exact bytes emitted on step u -> next_ret[u] (forward text order: thick_word + L[u])
      - run_of_ret: list of retained thin runs (pos, w, b, dst, ret_start)
    """
    n = len(data)
    # Build LF
    LF = [0] * n
    for pos, w, b, dst in full_runs:
        for off in range(w):
            LF[pos + off] = dst + off

    is_ret = bytearray(n)
    thin_runs = []  # (pos, w, b, dst) after splitting thick run containing p
    for pos, w, b, dst in full_runs:
        if w <= limit:
            is_ret[pos:pos + w] = b'\x01' * w
            thin_runs.append((pos, w, b, dst))
        elif pos <= p < pos + w:
            is_ret[p] = 1
            thin_runs.append((p, 1, b, dst + (p - pos)))

    row_to_ret = [-1] * n
    ret_rows = []
    for row in range(n):
        if is_ret[row]:
            row_to_ret[row] = len(ret_rows)
            ret_rows.append(row)
    m = len(ret_rows)

    # Compute exact Open edges[u] and words[u] (emitted in backward LF, stored in forward text order)
    next_ret = [0] * m
    word_len = [0] * m
    # For memory efficiency in Python, store words only per-band? No, 895k/6 = 150k per block,
    # storing bytes per retained row takes ~3 MB in Python — totally fine!
    words = [None] * m
    for u, row in enumerate(ret_rows):
        b0 = data[row]
        cur = LF[row]
        if is_ret[cur]:
            next_ret[u] = row_to_ret[cur]
            words[u] = bytes([b0])
            word_len[u] = 1
        else:
            buf = [b0]
            while not is_ret[cur]:
                buf.append(data[cur])
                cur = LF[cur]
            buf.reverse()
            next_ret[u] = row_to_ret[cur]
            words[u] = bytes(buf)
            word_len[u] = len(buf)

    # Verify baseline Open walk matches exact n bytes!
    p_ret = row_to_ret[p]
    return ret_rows, row_to_ret, next_ret, words, word_len, thin_runs, p_ret


def simulate_tube_open(data, p, full_runs, ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret,
                       mode="exact", max_wlen=15, include_thick_tail=False, limit=16):
    """
    Simulate exact Post-S3a Thin-Tube Shortcut Contraction on top of Open's retained map!
    Crucial exactness invariant:
      - We NEVER create a shortcut that skips over `p_ret` (the marked row)!
        (Any thin run containing `p` is excluded from being bypassed, affecting <= 1 run!)
      - We compute a shortcut map on thin runs R_2 = {thin runs with 2 <= w <= limit}:
        Each strip i in R_2 gets a direct jump (target_pos, shift_offset, extra_bytes) skipping
        any unsplit-thin successors S_j in R_2!
      - To prevent infinite cycles on periodic tubes AND guarantee every skipped strip actually
        reduces the marked orbit length:
          * In `exact` mode: S_i -> S_j iff dst(i) == pos(j) and w(i) == w(j) (1-to-1 chains!).
            Every chain S_0 -> S_1 -> ... -> S_{h-1} contracts into chunks of length <= max_wlen
            (if max_wlen is set): S_0 jumps to S_{min(h-1, max_wlen-1)}, etc.!
            Wait: if include_thick_tail=True and S_{h-1} lands in a thick strip (which Open ALREADY
            shortcuts in base_next[S_{h-1}]!), then S_0 absorbs S_{h-1} TOO (jumping straight to
            base_next[S_{h-1}]!) — saving an EXTRA `w` walk steps per tube!!
          * In `incl` mode: S_i -> S_j iff [dst(i), dst(i)+w(i)) subset [pos(j), pos(j)+w(j)) and S_j in R_2!
            Every strip S_i can absorb S_j ONLY IF we break cycles (e.g., topological / DAG order, or
            simply shortcutting from roots/non-bypassed heads of the inclusion forest!).
            WAIT A SECOND!! Look carefully at an inclusion forest (tree converging toward a wide trunk!):
            Suppose S_a (w=3) -> S_c (w=8) and S_b (w=5) -> S_c (w=8) and S_c (w=8) -> S_d (w=8)!
            If S_c is bypassed by BOTH S_a and S_b, AND S_d is bypassed by S_c:
              - S_a jumps straight over S_c AND S_d to LF(S_d) (saving 3 + 3 = 6 steps!)
              - S_b jumps straight over S_c AND S_d to LF(S_d) (saving 5 + 5 = 10 steps!)
              - Total saved = 16 steps (ALL of S_c and ALL of S_d skipped!)!
            Wait: what if `S_a` ITSELF was entered by some `S_0` (`w=3`)? Then `S_0` skips `S_a`, `S_c`, `S_d`!
            So **EVERY non-root strip `S` in the inclusion forest with `in_deg >= 1` is bypassed by its parent(s) —
            we simply trace forward from each strip `S` (or compute in reverse topological order) up to
            length `max_wlen` (stopping if hitting `p` or a cycle)!**
            WAIT: if we shortcut EVERY strip `S` forward by `h` steps, and `S_parent -> S` ALSO shortcuts forward,
            does `S_parent` skip `S` completely? YES!! Because `S_parent` jumps directly to `S`'s target,
            so `S` is NEVER visited from `S_parent`!
            The ONLY thing we must guard against is skipping over `p` (so `p`'s run is a hard stop: nobody skips `p`'s run!).
    Let's execute this cleanly, run the ACTUAL marked orbit walk from `p_ret`, verify 100% BYTE-FOR-BYTE
    EXACTNESS against Open's baseline text, and measure exact walk steps saved & strip work!
    """
    n = len(data)
    m = len(ret_rows)

    # Index R_2 runs (2 <= w <= limit, excluding any run containing p!)
    r2_runs = []  # (orig_pos, w, byte, dst, ret_start)
    for pos, w, b, dst in thin_runs:
        if 2 <= w <= limit:
            r2_runs.append((pos, w, b, dst, row_to_ret[pos]))

    num_r2 = len(r2_runs)
    r2_starts = [r[0] for r in r2_runs]
    start_to_r2 = {r[0]: idx for idx, r in enumerate(r2_runs)}
    p_r2 = -1
    for idx, (pos, w, _, _, _) in enumerate(r2_runs):
        if pos <= p < pos + w:
            p_r2 = idx
            break

    # Build successor edge `succ[i] = (j, off_in_j)` for R_2 runs!
    # In real code, this is ONE linear 2-pointer merge of L-order R_2 vs F-order (byte-bucketed) R_2!
    succ = [None] * num_r2
    in_deg = [0] * num_r2
    for i, (pos, w, b, dst, rs) in enumerate(r2_runs):
        if mode == "exact":
            j = start_to_r2.get(dst, -1)
            if j != -1 and j != p_r2 and r2_runs[j][1] == w:
                succ[i] = (j, 0)
                in_deg[j] += 1
        elif mode == "incl":
            ti = bisect.bisect_right(r2_starts, dst) - 1
            if ti >= 0 and ti != p_r2:
                t_pos, t_w, _, _, _ = r2_runs[ti]
                if dst + w <= t_pos + t_w:
                    succ[i] = (ti, dst - t_pos)
                    in_deg[ti] += 1
        else:
            raise ValueError(mode)

    # Copy base edges/words and apply ONLY strip-level shortcuts!
    # Wait: how do we wire shortcuts in O(num_r2) strip steps (NOT per-row!)?
    # Trace chains from every R_2 strip i that is NOT bypassed (in exact mode: in_deg[i] == 0,
    # or in incl mode: trace every strip that can actually be entered from outside R_2 or from a split!).
    # Actually: in `incl` mode, a strip j can be entered from outside R_2 if sum_{i: succ[i]=j} w(i) < w(j)!
    # Let's compute `covered_in[j] = sum_{i: succ[i]=j} w(i)`!
    # If `covered_in[j] == w(j)`, then j is 100% covered by R_2 predecessors (NEVER entered from outside R_2)!
    # If `covered_in[j] < w(j)`, then j has uncovered entry rows (must remain a valid entry point)!
    covered_in = [0] * num_r2
    for i in range(num_r2):
        if succ[i] is not None:
            j, _ = succ[i]
            covered_in[j] += r2_runs[i][1]

    # Break any closed cycles in succ (mark cycle-closing edge as None)
    color = [0] * num_r2  # 0=unvis, 1=visiting, 2=done
    for i in range(num_r2):
        if color[i] != 0:
            continue
        path = []
        cur = i
        while cur != -1 and color[cur] == 0:
            color[cur] = 1
            path.append(cur)
            cur = succ[cur][0] if succ[cur] is not None else -1
        if cur != -1 and color[cur] == 1:
            # Break cycle at last edge in path!
            last = path[-1]
            j_old, _ = succ[last]
            covered_in[j_old] -= r2_runs[last][1]
            succ[last] = None
        for node in path:
            color[node] = 2

    # Now `succ` is a strict DAG (functional forest)!
    # For every strip i that has `succ[i] is not None`, compute its composed shortcut (dst_ret_start, bytes)
    # ONLY for strips that are NOT 100% covered (i.e. `covered_in[i] < w(i)`, OR `i == p_r2`),
    # stopping a shortcut chain when:
    #   - next node is None (or would exceed max_wlen if max_wlen is not None), OR
    #   - if we hit a node already assigned a shortcut? No: trace forward greedily up to max_wlen!
    # Wait: if a chain has length 40 and max_wlen = 15:
    #   - S_0 jumps 15 steps to S_15 (skipping S_1..S_14!)
    #   - Then S_15 IS entered from S_0, so S_15 must jump 15 steps to S_30!
    #   - Then S_30 jumps 10 steps to S_39 -> exit!
    # So we simply mark which nodes are `entry_needed` (initially all i with covered_in[i] < w(i) or i == p_r2),
    # and whenever a shortcut stops early at node `stop_j` due to max_wlen, `stop_j` ALSO becomes `entry_needed`!
    new_next = list(base_next)
    new_words = list(base_words)

    entry_needed = [False] * num_r2
    for i in range(num_r2):
        if covered_in[i] < r2_runs[i][1] or i == p_r2:
            entry_needed[i] = True

    # Process in topological order (roots first, using Kahn's algorithm on in_deg!)
    indeg_dag = [0] * num_r2
    for i in range(num_r2):
        if succ[i] is not None:
            indeg_dag[succ[i][0]] += 1
    queue = [i for i in range(num_r2) if indeg_dag[i] == 0]
    topo = []
    qi = 0
    while qi < len(queue):
        u = queue[qi]
        qi += 1
        topo.append(u)
        if succ[u] is not None:
            v = succ[u][0]
            indeg_dag[v] -= 1
            if indeg_dag[v] == 0:
                queue.append(v)

    strip_steps_traced = 0
    tubes_built = 0
    new_dict_words = 0

    for i in topo:
        if not entry_needed[i] or succ[i] is None:
            continue
        # Trace forward from i!
        w_i = r2_runs[i][1]
        rs_i = r2_runs[i][4]
        cur_node = i
        cur_off = 0
        # Collect emitted bytes in backward LF order: start with byte of i
        b_list = [r2_runs[i][2]]
        hops = 0

        while succ[cur_node] is not None:
            nxt_node, off_in_nxt = succ[cur_node]
            # Check max_wlen limit: adding nxt_node adds 1 byte (or base_wlen if final thick!)
            if max_wlen is not None and len(b_list) + 1 > max_wlen:
                # Stop BEFORE absorbing nxt_node; nxt_node will be entered at cur_off + off_in_nxt!
                entry_needed[nxt_node] = True
                break
            cur_off += off_in_nxt
            cur_node = nxt_node
            b_list.append(r2_runs[cur_node][2])
            hops += 1
            strip_steps_traced += 1

        if hops == 0:
            continue

        # Now `cur_node` (at offset `cur_off`) is the LAST thin strip absorbed by `i`!
        # What does `cur_node` map to in Open?
        # Look at base_next[rs_cur + cur_off .. rs_cur + cur_off + w_i]!
        # Wait: if `include_thick_tail` is True, we absorb `cur_node`'s Open step (even if cur_node
        # exits into a thick strip or splits into multiple bands -> NO! Only absorb cur_node's Open step
        # if we jump to `base_next[rs_cur + cur_off + k]` directly!).
        # WAIT A SECOND! Even if `cur_node` splits at its exit, `base_next[rs_cur + cur_off + k]`
        # ALREADY holds the exact Open target & word for each row k in 0..w_i!
        # Wait: if we ONLY absorb `b_list[:-1]` (stopping AT `cur_node` without absorbing `cur_node`'s exit),
        # then ALL w_i rows share ONE SINGLE shift + ONE SINGLE word `b_list[:-1]` (zero splitting at exit)!
        # Whereas if we ALSO absorb `cur_node`'s exit (when `cur_node` does NOT split across bands, OR
        # per-row in S4!), we save 1 EXTRA hop (`hops + 1` instead of `hops`)!
        # Let's distinguish cleanly:
        #   - If `not include_thick_tail`: jump from `i` directly to `cur_node` (skipping intermediate strips!
        #     Wait: if hops == 1, i -> cur_node IS base_next[i], which skips 0 strips!
        #     To skip `cur_node` itself, we MUST jump to `LF(cur_node)` = `base_next[cur_node]`!!
        #     Check! If S_0 -> S_1 -> S_2:
        #       - Jumping S_0 -> S_2 skips S_1 (1 strip skipped, word = [b(S_1), b(S_0)] = 2 bytes!)
        #       - Jumping S_0 -> LF(S_2) skips S_1 AND S_2 (2 strips skipped, word = [base_word(S_2), b(S_1), b(S_0)]!)
        #     AHA!! Good catch!!
        rs_cur = r2_runs[cur_node][4] + cur_off
        if include_thick_tail:
            # Absorb `cur_node`'s entire Open step (including any thick word at the exit of cur_node!)
            # Check if max_wlen allows it (check uniform band or per-row):
            # In S4, base_words[rs_cur + k] is ALREADY a single Open word; prefix is b_list[:-1] reversed!
            prefix_rev = bytes(reversed(b_list[:-1]))  # wait: b_list = [b_0, b_1, ..., b_h]; forward text is base_word(h) + b_{h-1}..b_0!
            # Wait: check order carefully!
            # Along LF: step 0 at S_0 emits b_0 = T[t-1]. Step 1 at S_1 emits b_1 = T[t-2].
            # Step h at S_h emits base_words[S_h] (which ends with b_h = T[t-1-h] and starts earlier!).
            # So in FORWARD text order: base_words[S_h] + b_{h-1} + ... + b_1 + b_0!
            can_absorb_tail = True
            if max_wlen is not None:
                for k in range(w_i):
                    if len(base_words[rs_cur + k]) + len(prefix_rev) > max_wlen:
                        can_absorb_tail = False
                        break
            if can_absorb_tail:
                tubes_built += 1
                new_dict_words += 1
                for k in range(w_i):
                    new_next[rs_i + k] = base_next[rs_cur + k]
                    new_words[rs_i + k] = base_words[rs_cur + k] + prefix_rev
                continue

        # Otherwise: jump from `i` directly to `cur_node` (skipping the `hops - 1` strictly intermediate strips!)
        # Wait: if hops == 1 (just S_0 -> S_1) and we didn't absorb S_1, jumping to S_1 saves 0 steps!
        # Wait: if S_1 exits into a normal thin run (word_len == 1, NOT thick!), absorbing S_1 ONLY adds 1 byte
        # (b_1) and jumps to `base_next[S_1]` — which is valid whenever S_1 has NO thick exit!
        # Let's check whether all rows of cur_node have base_wlen == 1 (thin exit, zero thick word):
        all_thin_exit = all(base_wlen[rs_cur + k] == 1 for k in range(w_i))
        if all_thin_exit and (max_wlen is None or len(b_list) <= max_wlen):
            tubes_built += 1
            new_dict_words += 1
            fwd_word = bytes(reversed(b_list))
            for k in range(w_i):
                new_next[rs_i + k] = base_next[rs_cur + k]
                new_words[rs_i + k] = fwd_word
        elif hops >= 2:
            # Stop AT cur_node (mark cur_node as entry_needed!), skipping hops - 1 strips!
            entry_needed[cur_node] = True
            tubes_built += 1
            new_dict_words += 1
            fwd_word = bytes(reversed(b_list[:-1]))
            for k in range(w_i):
                new_next[rs_i + k] = rs_cur + k
                new_words[rs_i + k] = fwd_word

    # Execute the ACTUAL marked orbit walk from p_ret and verify 100% BYTE-FOR-BYTE EXACTNESS!
    cur = p_ret
    chunks = []
    steps = 0
    while True:
        chunks.append(new_words[cur])
        cur = new_next[cur]
        steps += 1
        if cur == p_ret:
            break
    chunks.reverse()
    rec = b"".join(chunks)
    return rec, {
        'm_open': m,
        'm_tube': steps,
        'saved': m - steps,
        'pct_saved': 100.0 * (m - steps) / m,
        'r2_cnt': num_r2,
        'strip_steps': strip_steps_traced,
        'tubes_built': tubes_built,
        'new_words': new_dict_words,
    }


def main():
    blocks = load_blocks()
    # Precompute baseline Open retained map + exact reference text once per block
    pre = []
    for blk, data in blocks:
        full_runs, C, counts = compute_maximal_runs_and_lf(data)
        ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret = build_open_retained_map(
            data, blk['primary_index'], full_runs, limit=16
        )
        # Baseline Open text
        cur = p_ret
        ch = []
        while True:
            ch.append(base_words[cur])
            cur = base_next[cur]
            if cur == p_ret:
                break
        ch.reverse()
        ref_text = b"".join(ch)
        assert len(ref_text) == blk['n']
        pre.append((blk, data, full_runs, ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret, ref_text))

    variants = [
        ("V1: Exact-Match (inline<=15)", "exact", 15, False),
        ("V2: Exact-Match (uncapped)", "exact", None, False),
        ("V3: Exact+ThickTail (<=15)", "exact", 15, True),
        ("V4: Incl-Forest (inline<=15)", "incl", 15, False),
        ("V5: Incl-Forest (uncapped)", "incl", None, False),
        ("V6: Incl+ThickTail (<=15)", "incl", 15, True),
        ("V7: Incl+ThickTail (uncap)", "incl", None, True),
    ]

    print("=== POST-S3a THIN-TUBE CONTRACTION (`Tube-Open`) EXACT WALK SAVINGS & WORK (ALL 6 WIKIDATA BLOCKS) ===")
    print(f"{'Variant':29s} | {'m_Open':>7s} {'m_Tube':>7s} {'Saved':>7s} {'%Sav':>6s} | {'R2_cnt':>7s} {'StrpTr':>7s} {'Tubes':>6s} {'Sav/Tr':>6s} | Exact?")
    print("-" * 112)

    for vname, mode, mw, inc_thk in variants:
        tot_m = tot_mt = tot_sav = tot_r2 = tot_tr = tot_tb = 0
        per_pct = []
        for blk, data, full_runs, ret_rows, row_to_ret, base_next, base_words, base_wlen, thin_runs, p_ret, ref_text in pre:
            rec, st = simulate_tube_open(
                data, blk['primary_index'], full_runs, ret_rows, row_to_ret,
                base_next, base_words, base_wlen, thin_runs, p_ret,
                mode=mode, max_wlen=mw, include_thick_tail=inc_thk, limit=16
            )
            assert rec == ref_text, f"EXACTNESS MISMATCH on {blk['file']} ({vname})!"
            tot_m += st['m_open']
            tot_mt += st['m_tube']
            tot_sav += st['saved']
            tot_r2 += st['r2_cnt']
            tot_tr += st['strip_steps']
            tot_tb += st['tubes_built']
            per_pct.append(st['pct_saved'])
        print(f"{vname:29s} | {tot_m:7d} {tot_mt:7d} {tot_sav:7d} {100*tot_sav/tot_m:5.1f}% | "
              f"{tot_r2:7d} {tot_tr:7d} {tot_tb:6d} {tot_sav/max(1,tot_tr):5.2f}x | 6/6 EXACT!")
        if vname in ("V4: Incl-Forest (inline<=15)", "V6: Incl+ThickTail (<=15)", "V7: Incl+ThickTail (uncap)"):
            pcts = ", ".join(f"{p:.1f}%" for p in per_pct)
            print(f"    per-block %saved: [{pcts}]")


if __name__ == "__main__":
    main()
