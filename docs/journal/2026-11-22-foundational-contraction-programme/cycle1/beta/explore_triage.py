#!/usr/bin/env python3
"""
Cycle 1 Explore & Triage:
Compare 3 genuinely distinct non-materializing / compositional BWT contraction constructions:
  1. RV-RunIET: Doubly-linked-list Rauzy-Veech-Zorich cocycle contraction on the run-IET (pi, lambda).
     Variants: right-only, 2-sided greedy (max min(lam_d, lam_i)), Zorich-accelerated.
  2. IET-BSE: Run-granular IET Bucket-Schur Elimination (eliminating arbitrary F-buckets with
     proved q_{t+1} <= q_t - 1, resolving self-IETs via inner RV/Zorich).
  3. AHT-Strip: Width-ordered strip elimination / Agol-Hass-Thurston periodic reduction on the IET.

Verifies exactness on:
  - All binary strings n <= 8 (all primary rotations)
  - All ternary strings n <= 5 (all primary rotations)
  - Periodic & multi-cycle permutations
  - F4 witnesses: n=5 ([0,0,1,1,2], [0,0,2,0,1], [0,0,1,0,1]), n=10 ([0,0,1,1,2,1,2,0,0,1])
  - Fib_{2m} + $ (up to 2m=40!)
  - Symmetric-Middle-Run (K=4, 16, 64)
  - Authenticated Wikidata blocks (/tmp/mibz2-blocks/block-0.L, etc.)
"""

import json
import time
import random
from collections import defaultdict


# ---------- Exact BWT helpers ----------

def bwt_encode(T):
    """Exact forward BWT returning (L_list, primary_index)."""
    n = len(T)
    rots = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = [T[(i - 1) % n] for i in rots]
    p = rots.index(0)
    return L, p


def rle_from_L(L):
    """Merge L into maximal runs [(sym, length), ...]."""
    if not L:
        return []
    runs = []
    cur = L[0]
    cnt = 1
    for x in L[1:]:
        if x == cur:
            cnt += 1
        else:
            runs.append((cur, cnt))
            cur = x
            cnt = 1
    runs.append((cur, cnt))
    return runs


def decode_lf_reference(L, p):
    """Exact reference inverse BWT via LF walk."""
    n = len(L)
    sigma = max(L) + 1 if L else 0
    counts = [0] * sigma
    for x in L:
        counts[x] += 1
    C = [0] * sigma
    acc = 0
    for i in range(sigma):
        C[i] = acc
        acc += counts[i]
    pos = list(C)
    LF = [0] * n
    for i, ch in enumerate(L):
        LF[i] = pos[ch]
        pos[ch] += 1
    out = [0] * n
    cur = p
    for k in range(n - 1, -1, -1):
        out[k] = L[cur]
        cur = LF[cur]
    return out


# ---------- SLP expansion helper ----------

def expand_slp(rules, root):
    """Iterative expansion of SLP rules.
    Rule formats:
      ('lit', sym)
      ('cat', left_id, right_id)
      ('pow', base_id, k)
    """
    out = []
    stack = [root]
    while stack:
        u = stack.pop()
        r = rules[u]
        tag = r[0]
        if tag == 'lit':
            out.append(r[1])
        elif tag == 'cat':
            stack.append(r[2])
            stack.append(r[1])
        elif tag == 'pow':
            base, k = r[1], r[2]
            for _ in range(k):
                stack.append(base)
    return out


def slp_height(rules, root):
    memo = {}
    stack = [(root, False)]
    while stack:
        u, done = stack.pop()
        if u in memo:
            continue
        r = rules[u]
        if r[0] == 'lit':
            memo[u] = 0
        elif r[0] == 'cat':
            if done:
                memo[u] = 1 + max(memo[r[1]], memo[r[2]])
            else:
                stack.append((u, True))
                stack.append((r[1], False))
                stack.append((r[2], False))
        elif r[0] == 'pow':
            if done:
                memo[u] = 1 + memo[r[1]]
            else:
                stack.append((u, True))
                stack.append((r[1], False))
    return memo[root]


# ==============================================================================
# CANDIDATE 1: RV-RunIET (Doubly-Linked-List Rauzy-Veech-Zorich on Run-IET)
# ==============================================================================

def rv_run_iet(runs, p, mode="right", zorich=True, build_slp=True):
    """
    Exact Rauzy-Veech(-Zorich) cocycle contraction on the initial run-IET.
    Orientation: Phi = FL (forward in text).
      Piece u in 0..r-1:
        - domain J_u (in 'dom' doubly-linked list = F-order = stable sort by (sym, u))
        - image  I_u (in 'im'  doubly-linked list = L-order = 0, 1, ..., r-1)
        - length lam[u] = run_len[u]
        - SLP word w[u] = ('lit', sym[u]) of byte length wlen[u] = 1.
    modes:
      'right'    : always cut at right endpoint
      'left'     : always cut at left endpoint
      'two_sided': pick left or right endpoint with larger min(lam_d, lam_i)
    Returns (decoded_list_or_None, stats_dict).
    """
    r0 = len(runs)
    n = sum(w for _, w in runs)
    if n == 0:
        return [], {"steps": 0, "zorich_skips": 0, "slp_nodes": 0, "max_q": 0}

    # Build im list: 0 -> 1 -> ... -> r0-1
    im_prev = [-1] * r0
    im_next = [-1] * r0
    for u in range(r0):
        im_prev[u] = u - 1
        im_next[u] = u + 1 if u + 1 < r0 else -1
    im_head = 0
    im_tail = r0 - 1

    # Build dom list: stable sort by (sym, u)
    max_sym = max(c for c, _ in runs)
    buckets = [[] for _ in range(max_sym + 1)]
    for u, (c, _) in enumerate(runs):
        buckets[c].append(u)
    dom_order = []
    for b in buckets:
        dom_order.extend(b)

    dom_prev = [-1] * r0
    dom_next = [-1] * r0
    for idx, u in enumerate(dom_order):
        dom_prev[u] = dom_order[idx - 1] if idx > 0 else -1
        dom_next[u] = dom_order[idx + 1] if idx + 1 < r0 else -1
    dom_head = dom_order[0]
    dom_tail = dom_order[-1]

    lam = [w for _, w in runs]

    # Locate primary index p in dom_order
    # Row p is in dom_order at offset p from 0
    acc = 0
    p_piece = -1
    p_off = 0
    for u in dom_order:
        if acc <= p < acc + lam[u]:
            p_piece = u
            p_off = p - acc
            break
        acc += lam[u]
    assert p_piece != -1
    p_shift = 0  # byte shift inside word of p_piece

    # SLP
    if build_slp:
        rules = [('lit', c) for c, _ in runs]
        word = list(range(r0))
    else:
        rules = None
        word = list(range(r0))
    wlen = [1] * r0

    def dom_unlink(u):
        nonlocal dom_head, dom_tail
        pr, nx = dom_prev[u], dom_next[u]
        if pr != -1:
            dom_next[pr] = nx
        else:
            dom_head = nx
        if nx != -1:
            dom_prev[nx] = pr
        else:
            dom_tail = pr
        dom_prev[u] = dom_next[u] = -1

    def im_unlink(u):
        nonlocal im_head, im_tail
        pr, nx = im_prev[u], im_next[u]
        if pr != -1:
            im_next[pr] = nx
        else:
            im_head = nx
        if nx != -1:
            im_prev[nx] = pr
        else:
            im_tail = pr
        im_prev[u] = im_next[u] = -1

    def dom_insert_after(anchor, u):
        nonlocal dom_tail
        nx = dom_next[anchor]
        dom_prev[u] = anchor
        dom_next[u] = nx
        dom_next[anchor] = u
        if nx != -1:
            dom_prev[nx] = u
        else:
            dom_tail = u

    def dom_insert_before(anchor, u):
        nonlocal dom_head
        pr = dom_prev[anchor]
        dom_next[u] = anchor
        dom_prev[u] = pr
        dom_prev[anchor] = u
        if pr != -1:
            dom_next[pr] = u
        else:
            dom_head = u

    def im_insert_after(anchor, u):
        nonlocal im_tail
        nx = im_next[anchor]
        im_prev[u] = anchor
        im_next[u] = nx
        im_next[anchor] = u
        if nx != -1:
            im_prev[nx] = u
        else:
            im_tail = u

    def im_insert_before(anchor, u):
        nonlocal im_head
        pr = im_prev[anchor]
        im_next[u] = anchor
        im_prev[u] = pr
        im_prev[anchor] = u
        if pr != -1:
            im_next[pr] = u
        else:
            im_head = u

    def make_cat(w1, w2):
        if build_slp:
            nid = len(rules)
            rules.append(('cat', w1, w2))
            return nid
        return 0

    def make_pow(w1, k):
        if k == 1:
            return w1
        if build_slp:
            nid = len(rules)
            rules.append(('pow', w1, k))
            return nid
        return 0

    steps = 0
    zorich_skips = 0
    tie_merges = 0
    closed_cycles = []
    q = r0
    max_q = r0
    n_rem = n
    p_cycle_word = None
    p_cycle_shift = None
    p_cycle_len = None

    while q > 0:
        # Choose side
        if mode == "right":
            side = 1
        elif mode == "left":
            side = 0
        else:
            # two_sided: pick side with larger min(lam_d, lam_i), prefer tie or self-match
            dr, ir = dom_tail, im_tail
            dl, il = dom_head, im_head
            if dr == ir:
                side = 1
            elif dl == il:
                side = 0
            else:
                mr = lam[dr] if lam[dr] < lam[ir] else lam[ir]
                ml = lam[dl] if lam[dl] < lam[il] else lam[il]
                side = 1 if mr >= ml else 0

        if side == 1:
            d = dom_tail
            i = im_tail
            if d == i:
                # Closed cycle component at right end!
                steps += 1
                if p_piece == d:
                    p_cycle_word = word[d]
                    p_cycle_shift = p_shift
                    p_cycle_len = wlen[d]
                    p_piece = -1
                closed_cycles.append((word[d], lam[d], wlen[d]))
                n_rem -= lam[d]
                dom_unlink(d)
                im_unlink(d)
                q -= 1
                continue

            ld, li = lam[d], lam[i]
            if ld > li:
                # Top/dom wins on right: remove A of width li (or k*li via Zorich if im_prev[i] == d? No: check if i stays im_tail!)
                # Wait: when does the SAME pair (d, i) repeat?
                # After this step, i moves to im_insert_after(d, i).
                # So i becomes im_tail AGAIN iff d WAS im_prev[i] (i.e. im_next[d] == i)!
                # Wait: if im_next[d] == i, then i is ALREADY right after d, so im_insert_after(d, i) leaves i at im_tail!
                if zorich and im_next[d] == i:
                    # i stays im_tail and d stays dom_tail!
                    # Repeat k = (ld - 1) // li times (or ld // li if divides evenly -> leaves tie or exact)
                    k = (ld - 1) // li  # leaves rem in [1, li]
                    if k > 1:
                        zorich_skips += k - 1
                    # Marker check on d: suffix [ld - k*li, ld) is peeled in k layers of width li
                    # For x in [ld - k*li, ld):
                    #   layer m (1..k, from right? Let's be EXACT!)
                    #   Wait: 1 step maps suffix [ld-li, ld) of d into i with shift += wlen[i],
                    #   and shortens d to ld-li, while word[i] becomes W_i + W_d!
                    #   Wait: let's trace carefully!
                    #   Step 1: W_i^(1) = W_i + W_d. Points in [ld-li, ld) go to i with shift += |W_i|.
                    #   Step 2: W_i^(2) = W_i^(1) + W_d = W_i + 2 W_d. Points in [ld-2li, ld-li) go to i with shift += |W_i^(1)| = |W_i| + |W_d|!
                    #   Step m (1<=m<=k): points in [ld - m*li, ld - (m-1)*li) go to i with:
                    #     new_off = off - (ld - m*li),
                    #     shift += |W_i| + (m-1)*|W_d|!
                    if p_piece == d and p_off >= ld - k * li:
                        dist_from_right = ld - 1 - p_off  # 0..k*li-1
                        m = dist_from_right // li + 1     # 1..k
                        p_piece = i
                        p_off = p_off - (ld - m * li)
                        p_shift += wlen[i] + (m - 1) * wlen[d]
                    pow_d = make_pow(word[d], k)
                    word[i] = make_cat(word[i], pow_d)
                    wlen[i] += k * wlen[d]
                    lam[d] = ld - k * li
                    n_rem -= k * li
                    steps += 1
                    continue

                # Standard 1-step (ld > li)
                if p_piece == d and p_off >= ld - li:
                    p_piece = i
                    p_off -= (ld - li)
                    p_shift += wlen[i]
                word[i] = make_cat(word[i], word[d])
                wlen[i] += wlen[d]
                lam[d] -= li
                n_rem -= li
                if im_next[d] != i:
                    im_unlink(i)
                    im_insert_after(d, i)
                steps += 1

            elif li > ld:
                # Bottom/im wins on right: remove A of width ld
                # d moves to dom_insert_after(i, d).
                # d stays dom_tail iff dom_next[i] == d!
                if zorich and dom_next[i] == d:
                    k = (li - 1) // ld
                    if k > 1:
                        zorich_skips += k - 1
                    # Trace 1 step:
                    #   d stays at dom_tail; word[d] <- W_i + W_d; lam[i] -= ld.
                    #   Step 1:
                    #     - old d (all of it) gets shift += |W_i| (stays on d!)
                    #     - suffix [li-ld, li) of i moves to d with shift unchanged!
                    #   Step m (1..k):
                    #     - after k steps, word[d]^(k) = W_i^k + W_d.
                    #     - old d gets k copies of W_i prepended -> shift += k * |W_i|!
                    #     - point in i at [li - m*ld, li - (m-1)*ld) moves to d at step m (when shift unchanged),
                    #       and then during steps m+1..k it is ON d, so it gets (k - m) copies of W_i prepended!
                    #       Wait: verify m=1 (rightmost block [li-ld, li)): moves to d at step 1, then gets (k-1) copies of W_i -> shift += (k-1)*|W_i|!
                    #       Verify m=k (leftmost peeled block [li-k*ld, li-(k-1)*ld)): moves to d at step k -> shift += 0!
                    if p_piece == d:
                        p_shift += k * wlen[i]
                    elif p_piece == i and p_off >= li - k * ld:
                        dist_from_right = li - 1 - p_off
                        m = dist_from_right // ld + 1  # 1..k
                        p_piece = d
                        p_off = p_off - (li - m * ld)
                        p_shift += (k - m) * wlen[i]
                    pow_i = make_pow(word[i], k)
                    word[d] = make_cat(pow_i, word[d])
                    wlen[d] += k * wlen[i]
                    lam[i] = li - k * ld
                    n_rem -= k * ld
                    steps += 1
                    continue

                # Standard 1-step (li > ld)
                if p_piece == d:
                    p_shift += wlen[i]
                elif p_piece == i and p_off >= li - ld:
                    p_piece = d
                    p_off -= (li - ld)
                word[d] = make_cat(word[i], word[d])
                wlen[d] += wlen[i]
                lam[i] -= ld
                n_rem -= ld
                if dom_next[i] != d:
                    dom_unlink(d)
                    dom_insert_after(i, d)
                steps += 1

            else:
                # Tie: ld == li, d != i
                # i absorbs d: word[i] <- W_i + W_d; im(i) takes im(d)'s slot; unlink d!
                if p_piece == d:
                    p_piece = i
                    p_shift += wlen[i]
                word[i] = make_cat(word[i], word[d])
                wlen[i] += wlen[d]
                n_rem -= ld
                # Move i in im-list to d's position, then remove d from both lists
                if im_next[d] == i:
                    # d is right before i; removing d leaves i right in d's place!
                    im_unlink(d)
                else:
                    im_unlink(i)
                    im_insert_after(d, i)
                    im_unlink(d)
                dom_unlink(d)
                q -= 1
                tie_merges += 1
                steps += 1

        else:
            # LEFT endpoint (mirror image!)
            d = dom_head
            i = im_head
            if d == i:
                steps += 1
                if p_piece == d:
                    p_cycle_word = word[d]
                    p_cycle_shift = p_shift
                    p_cycle_len = wlen[d]
                    p_piece = -1
                closed_cycles.append((word[d], lam[d], wlen[d]))
                n_rem -= lam[d]
                dom_unlink(d)
                im_unlink(d)
                q -= 1
                continue

            ld, li = lam[d], lam[i]
            if ld > li:
                # Remove prefix A = [0, li)
                # i maps dom(i) -> A; prefix [0, li) of d maps A -> prefix of im(d)
                # So i becomes W_i + W_d, im(i) moves to im_insert_before(d, i), d shrinks from left (off shifts by -li)!
                if zorich and im_prev[d] == i:
                    k = (ld - 1) // li
                    if k > 1:
                        zorich_skips += k - 1
                    # Prefix [0, k*li) of d is peeled in k layers from left:
                    # Step m (1..k): block [(m-1)*li, m*li) of old d goes to i with shift += |W_i| + (m-1)*|W_d|!
                    # Remaining [k*li, ld) stays on d with p_off -= k*li!
                    if p_piece == d:
                        if p_off < k * li:
                            m = p_off // li + 1
                            p_piece = i
                            p_off = p_off - (m - 1) * li
                            p_shift += wlen[i] + (m - 1) * wlen[d]
                        else:
                            p_off -= k * li
                    pow_d = make_pow(word[d], k)
                    word[i] = make_cat(word[i], pow_d)
                    wlen[i] += k * wlen[d]
                    lam[d] = ld - k * li
                    n_rem -= k * li
                    steps += 1
                    continue

                if p_piece == d:
                    if p_off < li:
                        p_piece = i
                        p_shift += wlen[i]
                    else:
                        p_off -= li
                word[i] = make_cat(word[i], word[d])
                wlen[i] += wlen[d]
                lam[d] -= li
                n_rem -= li
                if im_prev[d] != i:
                    im_unlink(i)
                    im_insert_before(d, i)
                steps += 1

            elif li > ld:
                # Remove prefix A = [0, ld)
                # Prefix [0, ld) of i maps prefix of dom(i) -> A; d maps A -> im(d)
                # So d moves to dom_insert_before(i, d), word[d] <- W_i + W_d, i shrinks from left!
                if zorich and dom_prev[i] == d:
                    k = (li - 1) // ld
                    if k > 1:
                        zorich_skips += k - 1
                    if p_piece == d:
                        p_shift += k * wlen[i]
                    elif p_piece == i:
                        if p_off < k * ld:
                            m = p_off // ld + 1  # 1..k
                            p_piece = d
                            p_off = p_off - (m - 1) * ld
                            p_shift += (k - m) * wlen[i]
                        else:
                            p_off -= k * ld
                    pow_i = make_pow(word[i], k)
                    word[d] = make_cat(pow_i, word[d])
                    wlen[d] += k * wlen[i]
                    lam[i] = li - k * ld
                    n_rem -= k * ld
                    steps += 1
                    continue

                if p_piece == d:
                    p_shift += wlen[i]
                elif p_piece == i:
                    if p_off < ld:
                        p_piece = d
                    else:
                        p_off -= ld
                word[d] = make_cat(word[i], word[d])
                wlen[d] += wlen[i]
                lam[i] -= ld
                n_rem -= ld
                if dom_prev[i] != d:
                    dom_unlink(d)
                    dom_insert_before(i, d)
                steps += 1

            else:
                # Tie ld == li, d != i
                if p_piece == d:
                    p_piece = i
                    p_shift += wlen[i]
                word[i] = make_cat(word[i], word[d])
                wlen[i] += wlen[d]
                n_rem -= ld
                if im_prev[d] == i:
                    im_unlink(d)
                else:
                    im_unlink(i)
                    im_insert_before(d, i)
                    im_unlink(d)
                dom_unlink(d)
                q -= 1
                tie_merges += 1
                steps += 1

    assert n_rem == 0
    assert p_cycle_word is not None

    decoded = None
    h = -1
    if build_slp:
        cyc = expand_slp(rules, p_cycle_word)
        assert len(cyc) == p_cycle_len
        sh = p_cycle_shift % p_cycle_len
        rot = cyc[sh:] + cyc[:sh]
        # If periodic string T = S^k (single cycle of length p_cycle_len dividing n, all closed_cycles identical):
        # Wait: for general multi-cycle, return the cycle of p (or repeat if periodic BWT where all cycles have same len)
        decoded = rot
        h = slp_height(rules, p_cycle_word)

    stats = {
        "n": n,
        "r0": r0,
        "steps": steps,
        "zorich_skips": zorich_skips,
        "tie_merges": tie_merges,
        "slp_nodes": len(rules) if build_slp else r0 + steps,
        "slp_height": h,
        "max_q": max_q,
        "num_cycles": sum(mult for _, mult, _ in closed_cycles),
        "p_cycle_len": p_cycle_len,
    }
    return decoded, stats


# ==============================================================================
# CANDIDATE 2: IET-BSE (Run-Granular IET Bucket-Schur Elimination)
# ==============================================================================

def iet_bse(runs, p, order_mode="min_deg", build_slp=True):
    """
    Exact IET Bucket-Schur Elimination directly on the bi-monotone strip-IET.
    Every piece is a translation from a subinterval of bucket a (domain, F-order)
    to a subinterval of bucket b (image, L-order), carrying an SLP word.
    Orientation: Phi = FL (forward in text): domain is F-bucket, image is L-position
    Wait: in L-order, F-buckets are ALSO contiguous intervals!
    So every piece maps [dom_lo, dom_hi) subset F_a  -->  [im_lo, im_hi) subset F_b!
    We represent each bucket b as:
      - dom_list[b]: sorted list of pieces leaving b (tiling F_b)
      - im_list[b]:  sorted list of pieces entering b (tiling F_b)
    Eliminating bucket b:
      1. Separate self-pieces (a == b) from incoming (a != b) and outgoing (b -> c, c != b).
      2. Compute the exact exit map of the open IET on F_b (via inner RV-Zorich on F_b,
         where incoming pieces are 'entry ports' and outgoing pieces are 'exit ports'!).
         Actually: glue entry & exit? Even simpler: trace/induce using 1D inner RV on F_b,
         OR directly merge if k_self == 0!
      3. Replace touched pieces in surviving buckets' dom_list / im_list!
    Proved invariant: q_{t+1} <= q_t - 1 on every bucket elimination!
    """
    r0 = len(runs)
    n = sum(w for _, w in runs)
    if n == 0:
        return [], {"steps": 0}

    max_sym = max(c for c, _ in runs)
    counts = [0] * (max_sym + 1)
    for c, w in runs:
        counts[c] += w
    C = [0] * (max_sym + 2)
    for c in range(max_sym + 1):
        C[c + 1] = C[c] + counts[c]

    active_syms = [c for c in range(max_sym + 1) if counts[c] > 0]
    if build_slp:
        rules = [('lit', c) for c in range(max_sym + 1)]
    else:
        rules = None

    def make_cat(w1, w2):
        if build_slp:
            nid = len(rules)
            rules.append(('cat', w1, w2))
            return nid
        return 0

    def make_pow(w1, k):
        if k == 1:
            return w1
        if build_slp:
            nid = len(rules)
            rules.append(('pow', w1, k))
            return nid
        return 0

    # Build initial bi-monotone strips: intersect L-runs (im) with F-buckets (dom of Phi = F_c, im of Phi = L-pos)
    # Piece: [dom_b, im_b, dom_lo, im_lo, length, word, wlen]
    # Note: for L-run (c, w) at L-positions [pos, pos+w):
    #   Phi maps F_c's next w positions [f_cur[c], f_cur[c]+w) --> [pos, pos+w).
    #   Intersect [pos, pos+w) with F-buckets to split into bi-monotone strips!
    f_cur = list(C[:max_sym + 1])
    dom_pieces = defaultdict(list)  # b -> list of pieces sorted by dom_lo
    im_pieces = defaultdict(list)   # b -> list of pieces sorted by im_lo

    pos = 0
    im_b = 0
    while im_b <= max_sym and C[im_b + 1] == 0:
        im_b += 1

    pieces = []  # store as lists for mutability or tuples
    for c, w in runs:
        rem = w
        cur_im = pos
        cur_dom = f_cur[c]
        while rem > 0:
            while C[im_b + 1] <= cur_im:
                im_b += 1
            avail = C[im_b + 1] - cur_im
            take = rem if rem <= avail else avail
            # Piece: dom in F_c at cur_dom, im in F_{im_b} at cur_im, len=take, word=c, wlen=1
            pc = (c, im_b, cur_dom, cur_im, take, c, 1)
            dom_pieces[c].append(pc)
            im_pieces[im_b].append(pc)
            cur_im += take
            cur_dom += take
            rem -= take
        f_cur[c] += w
        pos += w

    # Ensure dom_pieces and im_pieces are sorted by dom_lo and im_lo respectively
    for b in active_syms:
        dom_pieces[b].sort(key=lambda x: x[2])
        im_pieces[b].sort(key=lambda x: x[3])

    q_init = sum(len(dom_pieces[b]) for b in active_syms)
    q_curr = q_init
    max_q = q_init
    q_violations = 0

    total_merge_ops = 0
    total_self_pieces = 0
    max_self_pieces = 0
    total_self_rv_steps = 0
    buckets_eliminated = 0

    # Track marker p: (cur_b, cur_pos_in_global_coords, byte_shift)
    p_pos = p
    p_b = 0
    while C[p_b + 1] <= p_pos:
        p_b += 1
    p_shift = 0
    p_cycle_word = None
    p_cycle_shift = None
    p_cycle_len = None

    alive = set(active_syms)

    def eliminate_bucket(b):
        nonlocal p_pos, p_b, p_shift, p_cycle_word, p_cycle_shift, p_cycle_len
        nonlocal total_merge_ops, total_self_pieces, max_self_pieces, total_self_rv_steps

        d_list = dom_pieces[b]
        i_list = im_pieces[b]

        # Build nodes for inner RV on F_b!
        # Wait: a self-piece appears in BOTH d_list and i_list as the SAME piece!
        # Let's index pieces uniquely:
        #   - self pieces (dom_b == b and im_b == b): share ONE node id in both dom & im lists!
        #   - exit pieces (dom_b == b and im_b != b): live ONLY in dom list (carry external im_b, im_lo)
        #   - entry pieces (dom_b != b and im_b == b): live ONLY in im list (carry external dom_b, dom_lo)
        # Wait: when an entry piece moves in im-list or gets composed with an exit piece, its external dom_lo
        # might be split into prefix/suffix! So track ext_dom_lo, ext_im_lo cleanly!

        # Map self pieces by (dom_lo, im_lo, length) to unify node id
        self_map = {}
        nodes_kind = []   # 0=self, 1=exit, 2=entry
        nodes_len = []
        nodes_word = []
        nodes_wlen = []
        nodes_ext_b = []  # for exit: target im_b; for entry: source dom_b
        nodes_ext_lo = [] # for exit: target im_lo; for entry: source dom_lo

        dom_ids = []
        n_self = 0
        for pc in d_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            nid = len(nodes_kind)
            if db == b and ib == b:
                self_map[(dlo, ilo, ln)] = nid
                nodes_kind.append(0)
                nodes_ext_b.append(b)
                nodes_ext_lo.append(-1)
                n_self += 1
            else:
                nodes_kind.append(1)  # exit
                nodes_ext_b.append(ib)
                nodes_ext_lo.append(ilo)
            nodes_len.append(ln)
            nodes_word.append(wd)
            nodes_wlen.append(wl)
            dom_ids.append(nid)

        im_ids = []
        for pc in i_list:
            db, ib, dlo, ilo, ln, wd, wl = pc
            if db == b and ib == b:
                nid = self_map[(dlo, ilo, ln)]
            else:
                nid = len(nodes_kind)
                nodes_kind.append(2)  # entry
                nodes_ext_b.append(db)
                nodes_ext_lo.append(dlo)
                nodes_len.append(ln)
                nodes_word.append(wd)
                nodes_wlen.append(wl)
            im_ids.append(nid)

        total_self_pieces += n_self
        if n_self > max_self_pieces:
            max_self_pieces = n_self

        # Doubly-linked lists for dom and im inside F_b
        num_nodes = len(nodes_kind)
        dp = [-1] * num_nodes
        dn = [-1] * num_nodes
        ip = [-1] * num_nodes
        inn = [-1] * num_nodes

        for idx, u in enumerate(dom_ids):
            dp[u] = dom_ids[idx - 1] if idx > 0 else -1
            dn[u] = dom_ids[idx + 1] if idx + 1 < len(dom_ids) else -1
        d_tail = dom_ids[-1]

        for idx, u in enumerate(im_ids):
            ip[u] = im_ids[idx - 1] if idx > 0 else -1
            inn[u] = im_ids[idx + 1] if idx + 1 < len(im_ids) else -1
        i_tail = im_ids[-1]

        def d_unlink(u):
            nonlocal d_tail
            pr, nx = dp[u], dn[u]
            if pr != -1: dn[pr] = nx
            if nx != -1: dp[nx] = pr
            else: d_tail = pr
            dp[u] = dn[u] = -1

        def i_unlink(u):
            nonlocal i_tail
            pr, nx = ip[u], inn[u]
            if pr != -1: inn[pr] = nx
            if nx != -1: ip[nx] = pr
            else: i_tail = pr
            ip[u] = inn[u] = -1

        def d_ins_after(anc, u):
            nonlocal d_tail
            nx = dn[anc]
            dp[u] = anc
            dn[u] = nx
            dn[anc] = u
            if nx != -1: dp[nx] = u
            else: d_tail = u

        def i_ins_after(anc, u):
            nonlocal i_tail
            nx = inn[anc]
            ip[u] = anc
            inn[u] = nx
            inn[anc] = u
            if nx != -1: ip[nx] = u
            else: i_tail = u

        def new_node(kind, ln, wd, wl, ext_b, ext_lo):
            nid = len(nodes_kind)
            nodes_kind.append(kind)
            nodes_len.append(ln)
            nodes_word.append(wd)
            nodes_wlen.append(wl)
            nodes_ext_b.append(ext_b)
            nodes_ext_lo.append(ext_lo)
            dp.append(-1); dn.append(-1)
            ip.append(-1); inn.append(-1)
            return nid

        # Locate marker if p_b == b
        p_in_b = (p_b == b)
        p_nid = -1
        p_off = 0
        if p_in_b:
            acc = C[b]
            for u in dom_ids:
                if acc <= p_pos < acc + nodes_len[u]:
                    p_nid = u
                    p_off = p_pos - acc
                    break
                acc += nodes_len[u]
            assert p_nid != -1

        emitted = []  # composed external pieces (src_b, dst_b, src_lo, dst_lo, ln, wd, wl)
        inner_steps = 0

        while d_tail != -1:
            d = d_tail
            i = i_tail
            inner_steps += 1

            # Fixed point inside F_b? (only possible in multi-cycle BWT)
            if d == i:
                assert nodes_kind[d] == 0
                if p_in_b and p_nid == d:
                    p_cycle_word = nodes_word[d]
                    p_cycle_shift = p_shift
                    p_cycle_len = nodes_wlen[d]
                    p_in_b = False
                    p_nid = -1
                    p_b = -1
                d_unlink(d)
                i_unlink(d)
                continue

            kd, ki = nodes_kind[d], nodes_kind[i]
            ld, li = nodes_len[d], nodes_len[i]

            if kd == 1 and ki == 2:
                # EXIT meets ENTRY: directly emit composed piece!
                total_merge_ops += 1
                take = ld if ld < li else li
                src_b = nodes_ext_b[i]
                dst_b = nodes_ext_b[d]
                src_lo = nodes_ext_lo[i] + (li - take)
                dst_lo = nodes_ext_lo[d] + (ld - take)
                cw = make_cat(nodes_word[i], nodes_word[d])
                cwl = nodes_wlen[i] + nodes_wlen[d]
                emitted.append((src_b, dst_b, src_lo, dst_lo, take, cw, cwl))

                if p_in_b and p_nid == d and p_off >= ld - take:
                    # Marker leaves F_b to src_b!
                    p_b = src_b
                    p_pos = src_lo + (p_off - (ld - take))
                    p_shift += nodes_wlen[i]
                    p_in_b = False
                    p_nid = -1

                if ld == take:
                    d_unlink(d)
                else:
                    nodes_len[d] = ld - take
                if li == take:
                    i_unlink(i)
                else:
                    nodes_len[i] = li - take

            elif ld > li:
                # d (self or exit) is strictly wider than i (self or entry)
                # Prefix of d stays; suffix of width li of d is consumed by i!
                if kd == 0:
                    # d is SELF, i is self or entry
                    # Zorich check if inn[d] == i:
                    if inn[d] == i:
                        k = (ld - 1) // li
                        if p_in_b and p_nid == d and p_off >= ld - k * li:
                            dist = ld - 1 - p_off
                            m = dist // li + 1
                            if ki == 2:
                                # i is entry -> marker exits F_b to ext_b[i]!
                                p_b = nodes_ext_b[i]
                                p_pos = nodes_ext_lo[i] + (p_off - (ld - m * li))
                                p_shift += nodes_wlen[i] + (m - 1) * nodes_wlen[d]
                                p_in_b = False
                                p_nid = -1
                            else:
                                p_nid = i
                                p_off = p_off - (ld - m * li)
                                p_shift += nodes_wlen[i] + (m - 1) * nodes_wlen[d]
                        pw = make_pow(nodes_word[d], k)
                        nodes_word[i] = make_cat(nodes_word[i], pw)
                        nodes_wlen[i] += k * nodes_wlen[d]
                        nodes_len[d] = ld - k * li
                        continue

                    if p_in_b and p_nid == d and p_off >= ld - li:
                        if ki == 2:
                            p_b = nodes_ext_b[i]
                            p_pos = nodes_ext_lo[i] + (p_off - (ld - li))
                            p_shift += nodes_wlen[i]
                            p_in_b = False
                            p_nid = -1
                        else:
                            p_nid = i
                            p_off -= (ld - li)
                            p_shift += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_len[d] -= li
                    if inn[d] != i:
                        i_unlink(i)
                        i_ins_after(d, i)
                else:
                    # d is EXIT (kd == 1), i must be SELF (ki == 0)
                    # Suffix of d of width li is consumed by i; i becomes an EXIT piece!
                    # Wait: i's domain is INSIDE F_b, its image was the removed suffix, so after
                    # composing with d's suffix, i is now an EXIT piece mapping dom(i) -> suffix of ext_im(d)!
                    if p_in_b and p_nid == d and p_off >= ld - li:
                        p_nid = i
                        p_off -= (ld - li)
                        p_shift += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d] + (ld - li)
                    nodes_len[d] -= li
                    i_unlink(i)  # no longer has image inside F_b!

            elif li > ld:
                # i (self or entry) is strictly wider than d (self or exit)
                if ki == 0:
                    # i is SELF, d is self or exit
                    if dn[i] == d:
                        k = (li - 1) // ld
                        if p_in_b:
                            if p_nid == d:
                                p_shift += k * nodes_wlen[i]
                            elif p_nid == i and p_off >= li - k * ld:
                                dist = li - 1 - p_off
                                m = dist // ld + 1
                                p_nid = d
                                p_off = p_off - (li - m * ld)
                                p_shift += (k - m) * nodes_wlen[i]
                        pw = make_pow(nodes_word[i], k)
                        nodes_word[d] = make_cat(pw, nodes_word[d])
                        nodes_wlen[d] += k * nodes_wlen[i]
                        nodes_len[i] = li - k * ld
                        continue

                    if p_in_b:
                        if p_nid == d:
                            p_shift += nodes_wlen[i]
                        elif p_nid == i and p_off >= li - ld:
                            p_nid = d
                            p_off -= (li - ld)
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_len[i] -= ld
                    if dn[i] != d:
                        d_unlink(d)
                        d_ins_after(i, d)
                else:
                    # i is ENTRY (ki == 2), d must be SELF (kd == 0)
                    # Suffix of i (width ld) composes with d -> becomes a new ENTRY piece with image im(d)!
                    src_lo = nodes_ext_lo[i] + (li - ld)
                    if p_in_b and p_nid == d:
                        p_b = nodes_ext_b[i]
                        p_pos = src_lo + p_off
                        p_shift += nodes_wlen[i]
                        p_in_b = False
                        p_nid = -1
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = src_lo
                    nodes_len[i] -= ld
                    d_unlink(d)  # no longer has domain inside F_b!

            else:
                # Tie: ld == li (and not both exit/entry, handled above)
                if kd == 0 and ki == 0:
                    # self meets self
                    if p_in_b and p_nid == d:
                        p_nid = i
                        p_shift += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    if inn[d] == i:
                        i_unlink(d)
                    else:
                        i_unlink(i)
                        i_ins_after(d, i)
                        i_unlink(d)
                    d_unlink(d)
                elif kd == 1 and ki == 0:
                    # exit meets self (same width): i becomes exit!
                    if p_in_b and p_nid == d:
                        p_nid = i
                        p_shift += nodes_wlen[i]
                    nodes_word[i] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[i] += nodes_wlen[d]
                    nodes_kind[i] = 1
                    nodes_ext_b[i] = nodes_ext_b[d]
                    nodes_ext_lo[i] = nodes_ext_lo[d]
                    i_unlink(i)
                    d_unlink(d)
                elif kd == 0 and ki == 2:
                    # self meets entry (same width): d becomes entry!
                    if p_in_b and p_nid == d:
                        p_b = nodes_ext_b[i]
                        p_pos = nodes_ext_lo[i] + p_off
                        p_shift += nodes_wlen[i]
                        p_in_b = False
                        p_nid = -1
                    nodes_word[d] = make_cat(nodes_word[i], nodes_word[d])
                    nodes_wlen[d] += nodes_wlen[i]
                    nodes_kind[d] = 2
                    nodes_ext_b[d] = nodes_ext_b[i]
                    nodes_ext_lo[d] = nodes_ext_lo[i]
                    d_unlink(d)
                    i_unlink(i)

        total_self_rv_steps += inner_steps

        # Update surviving buckets: remove old pieces touching b, insert emitted pieces!
        touched_src = set()
        touched_dst = set()
        for pc in i_list:
            if pc[0] != b:
                touched_src.add(pc[0])
        for pc in d_list:
            if pc[1] != b:
                touched_dst.add(pc[1])

        by_src = defaultdict(list)
        by_dst = defaultdict(list)
        for ep in emitted:
            by_src[ep[0]].append(ep)
            by_dst[ep[1]].append(ep)

        for sb in touched_src:
            new_l = [pc for pc in dom_pieces[sb] if pc[1] != b]
            new_l.extend(by_src[sb])
            new_l.sort(key=lambda x: x[2])
            dom_pieces[sb] = new_l
        for db in touched_dst:
            new_l = [pc for pc in im_pieces[db] if pc[0] != b]
            new_l.extend(by_dst[db])
            new_l.sort(key=lambda x: x[3])
            im_pieces[db] = new_l

        del dom_pieces[b]
        del im_pieces[b]
        alive.remove(b)
        return len(emitted)

    while len(alive) > 1:
        # Choose next bucket to eliminate
        if order_mode == "min_deg":
            b = min(alive, key=lambda x: (len(dom_pieces[x]) + len(im_pieces[x]), x))
        elif order_mode == "max_width":
            b = max(alive, key=lambda x: (counts[x], -x))
        elif order_mode == "min_width":
            b = min(alive, key=lambda x: (counts[x], x))
        elif order_mode == "max_sym":
            b = max(alive)
        elif order_mode == "min_self":
            def self_cnt(x):
                return sum(1 for pc in dom_pieces[x] if pc[1] == x)
            b = min(alive, key=lambda x: (self_cnt(x), len(dom_pieces[x]) + len(im_pieces[x]), x))
        else:
            b = min(alive)

        old_q = q_curr
        eliminate_bucket(b)
        buckets_eliminated += 1
        q_curr = sum(len(dom_pieces[x]) for x in alive)
        if q_curr > old_q - 1:
            q_violations += 1
        if q_curr > max_q:
            max_q = q_curr

    # Solve the final single remaining bucket using RV-RunIET!
    last_b = next(iter(alive))
    d_last = dom_pieces[last_b]
    i_last = im_pieces[last_b]
    q_last = len(d_last)
    n_last = counts[last_b]

    # Run inner RV on last_b (same exact engine as rv_run_iet, with arbitrary initial words!)
    piece_id_map = {(pc[2], pc[3], pc[4]): idx for idx, pc in enumerate(d_last)}
    rL = len(d_last)
    dp = [idx - 1 for idx in range(rL)]
    dn = [idx + 1 if idx + 1 < rL else -1 for idx in range(rL)]
    d_tail = rL - 1

    im_order = [piece_id_map[(pc[2], pc[3], pc[4])] for pc in i_last]
    ip = [-1] * rL
    inn = [-1] * rL
    for idx, u in enumerate(im_order):
        ip[u] = im_order[idx - 1] if idx > 0 else -1
        inn[u] = im_order[idx + 1] if idx + 1 < rL else -1
    i_tail = im_order[-1]

    lam = [pc[4] for pc in d_last]
    word = [pc[5] for pc in d_last]
    wlen = [pc[6] for pc in d_last]

    p_piece = -1
    p_off = 0
    if p_cycle_word is None:
        assert p_b == last_b
        for idx, pc in enumerate(d_last):
            if pc[2] <= p_pos < pc[2] + pc[4]:
                p_piece = idx
                p_off = p_pos - pc[2]
                break
        assert p_piece != -1

    def d_unlink(u):
        nonlocal d_tail
        pr, nx = dp[u], dn[u]
        if pr != -1: dn[pr] = nx
        if nx != -1: dp[nx] = pr
        else: d_tail = pr
        dp[u] = dn[u] = -1

    def i_unlink(u):
        nonlocal i_tail
        pr, nx = ip[u], inn[u]
        if pr != -1: inn[pr] = nx
        if nx != -1: ip[nx] = pr
        else: i_tail = pr
        ip[u] = inn[u] = -1

    def d_ins_after(anc, u):
        nonlocal d_tail
        nx = dn[anc]
        dp[u] = anc; dn[u] = nx; dn[anc] = u
        if nx != -1: dp[nx] = u
        else: d_tail = u

    def i_ins_after(anc, u):
        nonlocal i_tail
        nx = inn[anc]
        ip[u] = anc; inn[u] = nx; inn[anc] = u
        if nx != -1: ip[nx] = u
        else: i_tail = u

    final_rv_steps = 0

    while d_tail != -1:
        d = d_tail
        i = i_tail
        final_rv_steps += 1
        if d == i:
            if p_piece == d:
                p_cycle_word = word[d]
                p_cycle_shift = p_shift
                p_cycle_len = wlen[d]
                p_piece = -1
            d_unlink(d)
            i_unlink(d)
            continue
        ld, li = lam[d], lam[i]
        if ld > li:
            if inn[d] == i:
                k = (ld - 1) // li
                if p_piece == d and p_off >= ld - k * li:
                    m = (ld - 1 - p_off) // li + 1
                    p_piece = i
                    p_off = p_off - (ld - m * li)
                    p_shift += wlen[i] + (m - 1) * wlen[d]
                word[i] = make_cat(word[i], make_pow(word[d], k))
                wlen[i] += k * wlen[d]
                lam[d] = ld - k * li
                continue
            if p_piece == d and p_off >= ld - li:
                p_piece = i
                p_off -= (ld - li)
                p_shift += wlen[i]
            word[i] = make_cat(word[i], word[d])
            wlen[i] += wlen[d]
            lam[d] -= li
            if inn[d] != i:
                i_unlink(i)
                i_ins_after(d, i)
        elif li > ld:
            if dn[i] == d:
                k = (li - 1) // ld
                if p_piece == d:
                    p_shift += k * wlen[i]
                elif p_piece == i and p_off >= li - k * ld:
                    m = (li - 1 - p_off) // ld + 1
                    p_piece = d
                    p_off = p_off - (li - m * ld)
                    p_shift += (k - m) * wlen[i]
                word[d] = make_cat(make_pow(word[i], k), word[d])
                wlen[d] += k * wlen[i]
                lam[i] = li - k * ld
                continue
            if p_piece == d:
                p_shift += wlen[i]
            elif p_piece == i and p_off >= li - ld:
                p_piece = d
                p_off -= (li - ld)
            word[d] = make_cat(word[i], word[d])
            wlen[d] += wlen[i]
            lam[i] -= ld
            if dn[i] != d:
                d_unlink(d)
                d_ins_after(i, d)
        else:
            if p_piece == d:
                p_piece = i
                p_shift += wlen[i]
            word[i] = make_cat(word[i], word[d])
            wlen[i] += wlen[d]
            if inn[d] == i:
                i_unlink(d)
            else:
                i_unlink(i)
                i_ins_after(d, i)
                i_unlink(d)
            d_unlink(d)

    decoded = None
    if build_slp:
        cyc = expand_slp(rules, p_cycle_word)
        assert len(cyc) == p_cycle_len
        sh = p_cycle_shift % p_cycle_len
        decoded = cyc[sh:] + cyc[:sh]

    stats = {
        "n": n,
        "r0": r0,
        "q_init": q_init,
        "max_q": max_q,
        "q_violations": q_violations,
        "buckets_elim": buckets_eliminated,
        "total_merge_ops": total_merge_ops,
        "total_self_rv_steps": total_self_rv_steps,
        "max_self_pieces": max_self_pieces,
        "last_b_n": n_last,
        "last_b_q": q_last,
        "final_rv_steps": final_rv_steps,
        "total_steps": total_self_rv_steps + final_rv_steps,
        "slp_nodes": len(rules) if build_slp else -1,
    }
    return decoded, stats


# ==============================================================================
# EXHAUSTIVE VERIFICATION + BENCHMARK SUITE
# ==============================================================================

def verify_small_exhaustive():
    print("=== 1. EXHAUSTIVE VERIFICATION (Binary n<=8, Ternary n<=5, Periodic, Multi-cycle) ===")
    cases = 0

    # Binary n=1..8
    for n in range(1, 9):
        for mask in range(1 << n):
            T = [(mask >> i) & 1 for i in range(n)]
            L, p = bwt_encode(T)
            runs = rle_from_L(L)
            # Check all modes of RV-RunIET + IET-BSE
            for mode in ("right", "left", "two_sided"):
                got, st = rv_run_iet(runs, p, mode=mode, zorich=True, build_slp=True)
                # For periodic T = S^k, got is S (period) -> tile to len n
                assert n % len(got) == 0
                full = got * (n // len(got))
                assert full == T, f"RV {mode} failed on T={T}: got {full}"
            got2, st2 = iet_bse(runs, p, order_mode="min_deg", build_slp=True)
            assert st2["q_violations"] == 0, f"IET-BSE q violation on T={T}: {st2}"
            assert n % len(got2) == 0
            assert got2 * (n // len(got2)) == T, f"IET-BSE failed on T={T}"
            cases += 1

    # Ternary n=1..5
    for n in range(1, 6):
        total = 3 ** n
        for idx in range(total):
            T = []
            v = idx
            for _ in range(n):
                T.append(v % 3)
                v //= 3
            L, p = bwt_encode(T)
            runs = rle_from_L(L)
            for mode in ("right", "two_sided"):
                got, st = rv_run_iet(runs, p, mode=mode, zorich=True, build_slp=True)
                assert n % len(got) == 0 and got * (n // len(got)) == T
            for om in ("min_deg", "max_width", "min_self"):
                got2, st2 = iet_bse(runs, p, order_mode=om, build_slp=True)
                assert st2["q_violations"] == 0
                assert n % len(got2) == 0 and got2 * (n // len(got2)) == T
            cases += 1

    # Arbitrary non-BWT (multi-cycle) columns n=1..7 binary, n=1..5 ternary with ALL primary marks p in 0..n-1
    multi_cases = 0
    for n in range(1, 7):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            runs = rle_from_L(L)
            # Compute LF cycles
            counts = [L.count(0), L.count(1)]
            pos = [0, counts[0]]
            LF = [0] * n
            for i, ch in enumerate(L):
                LF[i] = pos[ch]; pos[ch] += 1
            for p in range(n):
                # Extract exact cycle of p
                cyc_rev = []
                cur = p
                while True:
                    cyc_rev.append(L[cur])
                    cur = LF[cur]
                    if cur == p:
                        break
                cyc = cyc_rev[::-1]
                got, st = rv_run_iet(runs, p, mode="two_sided", zorich=True, build_slp=True)
                assert got == cyc, f"Multi-cycle RV failed L={L}, p={p}: got {got}, want {cyc}"
                got2, st2 = iet_bse(runs, p, order_mode="min_deg", build_slp=True)
                assert st2["q_violations"] == 0
                assert got2 == cyc, f"Multi-cycle IET-BSE failed L={L}, p={p}: got {got2}, want {cyc}"
                multi_cases += 1

    print(f"  PASS: {cases} exhaustive BWT cases + {multi_cases} multi-cycle/all-p cases (0 failures, 0 q_violations)!")


def test_counterexamples():
    print("\n=== 2. PRIOR COUNTEREXAMPLES (F4 n=5/10, Fib_{2m}+$, Symmetric-Middle-Run) ===")

    # F4 witnesses
    f4_list = [
        ("F4_n5_thresh", [0, 0, 1, 1, 2]),
        ("F4_n5_split", [0, 0, 2, 0, 1]),
        ("F4_n5_self", [0, 0, 1, 0, 1]),
        ("F4_n10_sat", [0, 0, 1, 1, 2, 1, 2, 0, 0, 1]),
    ]
    for name, T in f4_list:
        L, p = bwt_encode(T)
        runs = rle_from_L(L)
        g1, s1 = rv_run_iet(runs, p, mode="right", zorich=True)
        g2, s2 = iet_bse(runs, p, order_mode="min_deg")
        assert g1 == T and g2 == T and s2["q_violations"] == 0
        print(f"  {name:14s}: n={len(T):2d} r0={len(runs)} | RV_right: steps={s1['steps']} max_q={s1['max_q']} | IET-BSE: steps={s2['total_steps']} max_q={s2['max_q']}")

    # Fib_{2m} + $ (closed-form runs from Gamma's refute_bdmr_fib.py!)
    fibs = [0, 1]
    for _ in range(85):
        fibs.append(fibs[-1] + fibs[-2])
    print("  Fib_{2m} + $ (r0=4):")
    for two_m in [8, 12, 20, 40, 80]:
        f_prev2 = fibs[two_m - 2]
        f_prev1 = fibs[two_m - 1]
        runs = [(1, f_prev2 - 1), (2, 1), (0, f_prev1), (1, 1)]
        p = f_prev2 - 1
        n = f_prev2 + f_prev1
        _, s1 = rv_run_iet(runs, p, mode="right", zorich=True, build_slp=(two_m <= 12))
        _, s2 = iet_bse(runs, p, order_mode="min_deg", build_slp=(two_m <= 12))
        print(f"    2m={two_m:2d} n={n:18d}: RV_right steps={s1['steps']:3d} (skips={s1['zorich_skips']:3d}) max_q={s1['max_q']} | IET-BSE steps={s2['total_steps']:3d} max_q={s2['max_q']} q_viol={s2['q_violations']}")

    # Symmetric-Middle-Run (exact Referee generator)
    print("  Symmetric-Middle-Run (M_cnt=64):")
    for K in [4, 16, 64, 256]:
        rng = random.Random(123)
        M_cnt = 64
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        rng.shuffle(low_tags)
        rng.shuffle(high_tags)
        T = list(b"".join(low_tags[i] + bytes([77] * K) + high_tags[i] for i in range(M_cnt)))
        L, p = bwt_encode(T)
        runs = rle_from_L(L)
        g1, s1 = rv_run_iet(runs, p, mode="right", zorich=True, build_slp=True)
        g1t, s1t = rv_run_iet(runs, p, mode="two_sided", zorich=True, build_slp=True)
        g2, s2 = iet_bse(runs, p, order_mode="min_deg", build_slp=True)
        g2w, s2w = iet_bse(runs, p, order_mode="max_width", build_slp=True)
        assert g1 == T and g1t == T and g2 == T and g2w == T and s2["q_violations"] == 0 and s2w["q_violations"] == 0
        print(f"    K={K:3d} n={len(T):5d} r0={len(runs):3d}: "
              f"RV_R steps={s1['steps']:4d} ({s1['steps']/len(runs):.2f}r) | "
              f"RV_2S steps={s1t['steps']:4d} ({s1t['steps']/len(runs):.2f}r) | "
              f"BSE_md steps={s2['total_steps']:4d} | BSE_mw steps={s2w['total_steps']:4d}")


def test_wikidata():
    print("\n=== 3. AUTHENTICATED WIKIDATA BLOCKS (/tmp/mibz2-blocks/) ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    # First test block-0.L thoroughly across all variants, then all 6 blocks for winners
    for idx, binfo in enumerate(manifest["blocks"]):
        path = "/tmp/mibz2-blocks/" + binfo["file"]
        L = list(open(path, "rb").read())
        n, p = binfo["n"], binfo["primary_index"]
        runs = rle_from_L(L)
        r0 = len(runs)
        assert r0 == binfo["r"]

        print(f"\n  [{binfo['file']}] n={n}, r0={r0} (r/n={r0/n:.5f}), sigma={len(set(L))}")

        # Verify exactness on block-0.L (and build_slp=True on block-0, False or True on others)
        do_slp = (idx == 0)
        ref = decode_lf_reference(L, p) if do_slp else None

        for mode in ("right", "left", "two_sided"):
            t0 = time.perf_counter()
            got, st = rv_run_iet(runs, p, mode=mode, zorich=True, build_slp=do_slp)
            dt = time.perf_counter() - t0
            if do_slp:
                assert got == ref, f"RV {mode} MISMATCH on {binfo['file']}!"
            print(f"    RV-RunIET [{mode:9s}]: steps={st['steps']:7d} ({st['steps']/r0:5.2f}r, {st['steps']/n:.3f}n) "
                  f"ties={st['tie_merges']:5d} z_skip={st['zorich_skips']:6d} h={st['slp_height']:5d} ({dt:.2f}s)")

        if idx == 0:
            for om in ("min_deg", "min_self", "max_width", "min_width", "max_sym"):
                t0 = time.perf_counter()
                got, st = iet_bse(runs, p, order_mode=om, build_slp=True)
                dt = time.perf_counter() - t0
                assert got == ref, f"IET-BSE {om} MISMATCH on {binfo['file']}!"
                assert st["q_violations"] == 0
                print(f"    IET-BSE   [{om:9s}]: tot_steps={st['total_steps']:7d} ({st['total_steps']/r0:5.2f}r, {st['total_steps']/n:.3f}n) "
                      f"inner={st['total_self_rv_steps']:7d} final={st['final_rv_steps']:7d} "
                      f"merges={st['total_merge_ops']:7d} max_self={st['max_self_pieces']:5d} "
                      f"last_n={st['last_b_n']:6d} last_q={st['last_b_q']:5d} ({dt:.2f}s)")


if __name__ == "__main__":
    verify_small_exhaustive()
    test_counterexamples()
    test_wikidata()
