#!/usr/bin/env python3
"""
Cycle 5: Complete Mathematical Dissection & Generalization Audit of Open's Operator O_W
on all 6 authenticated Wikidata blocks (/tmp/mibz2-blocks/):
  1. Exact Width Spectrum (w = 1..15) of Open's m_16 retained thin rows.
  2. Exact Replica of Open's Auxiliary-IET Elimination (`src/open/endpoint.rs`) across
     W in {16, 12, 8, 6, 4, 3, 2}: exact n_s, m_W, open_nodes, Narrow/Wide, steps,
     rules, bands, arena_KiB, edges_KiB, and marginal cost per saved row!
  3. Exact Thin-Structure Anatomy of m_16:
     (A) Split-free (kappa=1) thin strips (w in 2..15) — 1-step & multi-step lockstep depth,
         and exact Open auxiliary-IET elimination when adding ONLY kappa=1 thin strips!
     (B) Cross-byte / periodic L-motifs (period p=2,3,4 alternating runs in L like (a,b)^k)
         whose LF images coalesce into stride-p contiguous strips in F!
     (C) Width-1 row fate census: where do w=1 rows land under LF (into thick >=16, thin 2..15, or w=1)?
         And how many thin rows sit in `Open`'s gap nodes vs isolated between thick strips?
"""
import hashlib
import json
import os
from collections import defaultdict

BLOCKS_DIR = "/tmp/mibz2-blocks"


def load_blocks():
    manifest = json.load(open(os.path.join(BLOCKS_DIR, "manifest.json")))
    out = []
    for blk in manifest["blocks"]:
        data = open(os.path.join(BLOCKS_DIR, blk["file"]), "rb").read()
        assert hashlib.sha256(data).hexdigest() == blk["sha256"]
        assert len(data) == blk["n"]
        out.append((blk, data))
    return out


def compute_maximal_runs_and_lf(data):
    """Return maximal runs [(pos, width, byte, dst)], C[0..256], and per-row run_id."""
    n = len(data)
    runs = []
    counts = [0] * 256
    i = 0
    while i < n:
        b = data[i]
        j = i + 1
        while j < n and data[j] == b:
            j += 1
        w = j - i
        runs.append((i, w, b))
        counts[b] += w
        i = j
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    occ = C[:]
    full_runs = []
    for pos, w, b in runs:
        dst = occ[b]
        occ[b] = dst + w
        full_runs.append((pos, w, b, dst))
    return full_runs, C, counts


def simulate_open_endpoint(n, p, full_runs, eliminated_mask):
    """
    Exact faithful simulation of `/tmp/moonshot-scorer/src/open/endpoint.rs` (`induced_with` + `eliminate`)
    for ANY chosen subset of eliminated runs (`eliminated_mask[idx] == True`)!
    Splits any eliminated run containing primary row `p` into (before, retained singleton p, after)
    identically to `open.rs::split`.
    Returns exact Open counts:
      n_s, m_ret, open_nodes, is_wide, steps, rules, bands, arena_kib, edges_kib
    """
    strips = []  # (from, to, width, byte)
    thin_by_byte = [0] * 256
    m_ret = 0

    for idx, (pos, w, b, dst) in enumerate(full_runs):
        if not eliminated_mask[idx]:
            thin_by_byte[b] += w
            m_ret += w
        else:
            if pos <= p < pos + w:
                if pos < p:
                    strips.append((pos, dst, p - pos, b))
                thin_by_byte[b] += 1
                m_ret += 1
                if p + 1 < pos + w:
                    strips.append((p + 1, dst + (p + 1 - pos), pos + w - (p + 1), b))
            else:
                strips.append((pos, dst, w, b))

    ns = len(strips)
    # Build image order (strips sorted by `to`)
    image_order = sorted(range(ns), key=lambda i: strips[i][1])

    # Build doubly linked circular lists on side 0 and side 1 identically to endpoint.rs!
    # Node 0 = sentinel; nodes 1..=ns = strips; nodes ns+1.. = domain & image gaps
    max_nodes = 3 * ns + 3
    prev0 = [0] * max_nodes
    next0 = [0] * max_nodes
    prev1 = [0] * max_nodes
    next1 = [0] * max_nodes
    width = [0] * max_nodes
    has_word = [False] * max_nodes  # True iff word != 0 (used to count rules accurately!)
    nodes = 1

    for i in range(ns):
        width[nodes] = strips[i][2]
        has_word[nodes] = True
        nodes += 1

    # Side 0: image gaps (terminals[0]) then domain order (order[0])
    # Side 1: domain gaps (terminals[1]) then image order (order[1])
    term0, ord0 = [], []
    term1, ord1 = [], []

    x = 0
    for i in range(ns):
        s_from, s_to, s_w, _ = strips[i]
        if x < s_from:
            nid = nodes
            nodes += 1
            width[nid] = s_from - x
            ord0.append(nid)
            term1.append(nid)
        ord0.append(i + 1)
        x = s_from + s_w
    if x < n:
        nid = nodes
        nodes += 1
        width[nid] = n - x
        ord0.append(nid)
        term1.append(nid)

    x = 0
    for i in image_order:
        s_from, s_to, s_w, _ = strips[i]
        if x < s_to:
            nid = nodes
            nodes += 1
            width[nid] = s_to - x
            ord1.append(nid)
            term0.append(nid)
        ord1.append(i + 1)
        x = s_to + s_w
    if x < n:
        nid = nodes
        nodes += 1
        width[nid] = n - x
        ord1.append(nid)
        term0.append(nid)

    open_nodes = nodes - 1

    def close_side(p_arr, n_arr, seq):
        prev = 0
        for nid in seq:
            n_arr[prev] = nid
            p_arr[nid] = prev
            prev = nid
        n_arr[prev] = 0
        p_arr[0] = prev

    close_side(prev0, next0, term0 + ord0)
    close_side(prev1, next1, term1 + ord1)

    # Exact `eliminate::<true>` loop (with spirals!) from endpoint.rs lines 385-450!
    left = n
    steps = 0
    rules = 0

    while left > 0:
        x_id = prev0[0]
        y_id = prev1[0]
        aw = width[x_id]
        bw = width[y_id]
        if x_id != y_id:
            if aw > bw and prev1[y_id] == x_id and (aw - 1) // bw >= 2:
                q = (aw - 1) // bw
                # power(x, q) + join(head, y)
                if has_word[x_id]:
                    # binary squaring rules count
                    bits = q.bit_length()
                    popc = q.bit_count()
                    rules += (bits - 1) + (popc - 1) + (1 if has_word[y_id] else 0)
                    has_word[y_id] = True
                width[x_id] = aw - q * bw
                left -= q * bw
                steps += q
                continue
            if aw < bw and prev0[x_id] == y_id and (bw - 1) // aw >= 2:
                q = (bw - 1) // aw
                if has_word[y_id]:
                    bits = q.bit_length()
                    popc = q.bit_count()
                    rules += (bits - 1) + (popc - 1) + (1 if has_word[x_id] else 0)
                    has_word[x_id] = True
                width[y_id] = bw - q * aw
                left -= q * aw
                steps += q
                continue
        w = aw if aw < bw else bw
        left -= w
        steps += 1
        if x_id == y_id:
            # pop_tail both sides
            p0 = prev0[x_id]
            next0[p0] = 0
            prev0[0] = p0
            p1 = prev1[x_id]
            next1[p1] = 0
            prev1[0] = p1
            continue
        if has_word[x_id] and has_word[y_id]:
            rules += 1
        new_hw = has_word[x_id] or has_word[y_id]
        if aw <= bw:
            # pop_tail(x, 0)
            p0 = prev0[x_id]
            next0[p0] = 0
            prev0[0] = p0
            # insert(y, x, 0)
            ny = next0[y_id]
            prev0[x_id] = y_id
            next0[x_id] = ny
            next0[y_id] = x_id
            prev0[ny] = x_id
            has_word[x_id] = new_hw
            if aw == bw:
                # cut(y, 0)
                py0, ny0 = prev0[y_id], next0[y_id]
                next0[py0] = ny0
                prev0[ny0] = py0
                # pop_tail(y, 1)
                p1 = prev1[y_id]
                next1[p1] = 0
                prev1[0] = p1
            else:
                width[y_id] = bw - w
        else:
            # pop_tail(y, 1)
            p1 = prev1[y_id]
            next1[p1] = 0
            prev1[0] = p1
            # insert(x, y, 1)
            nx = next1[x_id]
            prev1[y_id] = x_id
            next1[y_id] = nx
            next1[x_id] = y_id
            prev1[nx] = y_id
            has_word[y_id] = new_hw
            width[x_id] = aw - w

    # Count resulting bands (exact endpoint.rs lines 797-835)
    bands = 0
    nid = next0[0]
    in_node = 0
    k = 0
    for a in range(256):
        end = k + thin_by_byte[a]
        while k < end:
            nw = width[nid]
            w_take = min(end - k, nw - in_node)
            bands += 1
            in_node += w_take
            k += w_take
            if in_node == nw:
                nid = next0[nid]
                in_node = 0
    assert nid == 0

    is_wide = (3 * ns + 3) > 65536
    stride = 6 if is_wide else 4
    # Exact peak `edges` arena words: max(4*ns + stride*nodes + 4*rules, m_ret)
    arena_words = max(4 * ns + stride * nodes + 4 * rules, m_ret)
    # Plus bands (8B each) + inline16/lens (~17B per band upper bound, actually deduped!)
    return {
        'ns': ns,
        'm_ret': m_ret,
        'open_nodes': open_nodes,
        'is_wide': is_wide,
        'steps': steps,
        'rules': rules,
        'bands': bands,
        'arena_kib': arena_words * 4 / 1024.0,
        'edges_kib': m_ret * 4 / 1024.0,
        'bands_kib': bands * 8 / 1024.0,
    }


def analyze_thin_anatomy(data, full_runs, limit=16):
    """
    Deep structural census of ALL m_16 retained thin rows (w <= limit):
      1. Width spectrum w = 1..15 (count, rows, % of m_16)
      2. Split-free (kappa = 1) thin strips (w in 2..15):
         - 1-step unsplit (LF(S) lands inside a SINGLE target run of L)
         - 1-step unsplit into a THICK strip (w_dst > limit) vs into a THIN strip
         - multi-step unsplit lockstep depth h >= 2!
      3. Width-1 row fate census: LF(x) lands in thick (>16), thin (2..15), or w=1?
      4. Periodic alternating L-motifs (period p = 2, 3, 4) among thin rows!
    """
    n = len(data)
    # Build run_starts for bisect lookup of target run
    import bisect
    run_starts = [r[0] for r in full_runs]
    run_widths = [r[1] for r in full_runs]

    width_cnt = [0] * (limit + 1)
    m_16 = 0
    # Classify every thin run (w <= limit)
    k1_thin2_15_strips = 0
    k1_thin2_15_rows = 0
    k1_to_thick_rows = 0
    k1_to_thin_rows = 0
    k1_depth2_rows = 0
    split_thin2_15_strips = 0
    split_thin2_15_rows = 0

    w1_to_thick = 0
    w1_to_thin2_15 = 0
    w1_to_w1 = 0
    w1_in_pure_w1_chain_len2 = 0  # w=1 rows whose LF AND FL are both w=1!

    # Also build exact `is_w1` / `k1_mask` for Open simulation!
    k1_2_15_mask = [False] * len(full_runs)
    k1_4_15_mask = [False] * len(full_runs)

    for idx, (pos, w, b, dst) in enumerate(full_runs):
        if w <= limit:
            width_cnt[w] += 1
            m_16 += w
            ti = bisect.bisect_right(run_starts, dst) - 1
            t_pos, t_w, _, t_dst = full_runs[ti]
            if w == 1:
                if t_w > limit:
                    w1_to_thick += 1
                elif t_w >= 2:
                    w1_to_thin2_15 += 1
                else:
                    w1_to_w1 += 1
            else:
                # w in 2..=limit: check if [dst, dst + w) fits completely inside target run ti!
                if dst + w <= t_pos + t_w:
                    k1_thin2_15_strips += 1
                    k1_thin2_15_rows += w
                    k1_2_15_mask[idx] = True
                    if w >= 4:
                        k1_4_15_mask[idx] = True
                    if t_w > limit:
                        k1_to_thick_rows += w
                    else:
                        k1_to_thin_rows += w
                    # Check 2-step unsplit!
                    dst2 = t_dst + (dst - t_pos)
                    ti2 = bisect.bisect_right(run_starts, dst2) - 1
                    if dst2 + w <= run_starts[ti2] + run_widths[ti2]:
                        k1_depth2_rows += w
                else:
                    split_thin2_15_strips += 1
                    split_thin2_15_rows += w

    # Periodic alternating L-motifs (period p = 2, 3, 4) among thin rows of length >= 16!
    # Mark which rows are thin (in runs of width <= limit)
    is_thin = bytearray(n)
    for pos, w, _, _ in full_runs:
        if w <= limit:
            is_thin[pos:pos + w] = b'\x01' * w

    # Count thin rows covered by ANY period p in {2, 3, 4} repeating motif of length >= 16 (and >= 8) in L!
    motif_cov16 = bytearray(n)
    motif_cov8 = bytearray(n)
    for p_per in (2, 3, 4):
        run_l = 0
        for i in range(p_per, n):
            if is_thin[i] and is_thin[i - p_per] and data[i] == data[i - p_per]:
                run_l += 1
            else:
                if run_l + p_per >= 8:
                    start = i - run_l - p_per
                    if run_l + p_per >= 16:
                        motif_cov16[start:i] = b'\x01' * (i - start)
                    motif_cov8[start:i] = b'\x01' * (i - start)
                run_l = 0
        if run_l + p_per >= 8:
            start = n - run_l - p_per
            if run_l + p_per >= 16:
                motif_cov16[start:n] = b'\x01' * (n - start)
            motif_cov8[start:n] = b'\x01' * (n - start)

    motif16_rows = sum(motif_cov16)
    motif8_rows = sum(motif_cov8)
    # Count how many of those motif rows are specifically w=1 rows!
    w1_in_motif16 = 0
    for pos, w, _, _ in full_runs:
        if w == 1 and motif_cov16[pos]:
            w1_in_motif16 += 1

    return {
        'm_16': m_16,
        'width_cnt': width_cnt,
        'k1_2_15_strips': k1_thin2_15_strips,
        'k1_2_15_rows': k1_thin2_15_rows,
        'k1_to_thick': k1_to_thick_rows,
        'k1_to_thin': k1_to_thin_rows,
        'k1_depth2': k1_depth2_rows,
        'split_2_15_strips': split_thin2_15_strips,
        'split_2_15_rows': split_thin2_15_rows,
        'w1_to_thick': w1_to_thick,
        'w1_to_thin2_15': w1_to_thin2_15,
        'w1_to_w1': w1_to_w1,
        'motif16_rows': motif16_rows,
        'motif8_rows': motif8_rows,
        'w1_in_motif16': w1_in_motif16,
        'k1_2_15_mask': k1_2_15_mask,
        'k1_4_15_mask': k1_4_15_mask,
    }


def main():
    blocks = load_blocks()

    print("=== 1. EXACT WIDTH SPECTRUM (w = 1..15) OF OPEN's m_16 RETAINED THIN ROWS (6-BLK SUM & PER BLOCK) ===")
    agg_cnt = [0] * 17
    agg_m16 = 0
    per_blk_runs = []

    for blk, data in blocks:
        full_runs, C, counts = compute_maximal_runs_and_lf(data)
        an = analyze_thin_anatomy(data, full_runs, limit=16)
        per_blk_runs.append((blk, data, full_runs, an))
        agg_m16 += an['m_16']
        for w in range(1, 17):
            agg_cnt[w] += an['width_cnt'][w]

    print(f"{'w':>3s} | {'Strips(6b)':>10s} {'Rows(6b)':>10s} {'%m_16':>6s} | {'CumRows<=w':>10s} {'m_W(=<w)/m16':>12s}")
    print("-" * 64)
    cum = 0
    for w in range(1, 17):
        rows_w = w * agg_cnt[w]
        cum += rows_w
        print(f"{w:3d} | {agg_cnt[w]:10d} {rows_w:10d} {100.0*rows_w/agg_m16:5.1f}% | {cum:10d} {100.0*cum/agg_m16:11.1f}%")

    print("\n=== 2. THIN-STRUCTURE ANATOMY OF m_16 ACROSS ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    print(f"{'Block':11s} {'m_16':>7s} | {'w=1':>7s} {'%m16':>5s} | {'w1->Thk':>7s} {'w1->2..':>7s} {'w1->w1':>7s} | "
          f"{'k1(2..15)':>9s} {'%m16':>5s} {'(strp)':>6s} {'k1->Thk':>7s} {'k1_d2':>6s} | {'Mtf16':>6s} {'w1Mtf':>5s}")
    print("-" * 124)
    tot = defaultdict(int)
    for blk, data, full_runs, an in per_blk_runs:
        m = an['m_16']
        w1 = an['width_cnt'][1]
        print(f"{blk['file']:11s} {m:7d} | {w1:7d} {100*w1/m:4.1f}% | {an['w1_to_thick']:7d} {an['w1_to_thin2_15']:7d} {an['w1_to_w1']:7d} | "
              f"{an['k1_2_15_rows']:9d} {100*an['k1_2_15_rows']/m:4.1f}% {an['k1_2_15_strips']:6d} {an['k1_to_thick']:7d} {an['k1_depth2']:6d} | "
              f"{an['motif16_rows']:6d} {an['w1_in_motif16']:5d}")
        for k in ('m_16', 'w1_to_thick', 'w1_to_thin2_15', 'w1_to_w1', 'k1_2_15_rows', 'k1_2_15_strips',
                  'k1_to_thick', 'k1_depth2', 'motif16_rows', 'w1_in_motif16', 'split_2_15_rows', 'split_2_15_strips'):
            tot[k] += an[k]
        tot['w1'] += w1
    print("-" * 124)
    tm = tot['m_16']
    print(f"{'6-BLK SUM':11s} {tm:7d} | {tot['w1']:7d} {100*tot['w1']/tm:4.1f}% | {tot['w1_to_thick']:7d} {tot['w1_to_thin2_15']:7d} {tot['w1_to_w1']:7d} | "
          f"{tot['k1_2_15_rows']:9d} {100*tot['k1_2_15_rows']/tm:4.1f}% {tot['k1_2_15_strips']:6d} {tot['k1_to_thick']:7d} {tot['k1_depth2']:6d} | "
          f"{tot['motif16_rows']:6d} {tot['w1_in_motif16']:5d}")
    print(f"  Note: Splitting thin strips (w in 2..15, kappa>=2): {tot['split_2_15_strips']} strips, {tot['split_2_15_rows']} rows ({100*tot['split_2_15_rows']/tm:.1f}% of m_16)!")

    print("\n=== 3. EXACT OPEN AUXILIARY-IET SIMULATION: THRESHOLD SWEEP W & SPLIT-FREE (kappa=1) THIN ELIMINATION (6-BLK SUM) ===")
    configs = [
        ("Open W=16 (baseline)", lambda runs, an: [r[1] > 16 for r in runs]),
        ("Open W=12", lambda runs, an: [r[1] > 12 for r in runs]),
        ("Open W=8", lambda runs, an: [r[1] > 8 for r in runs]),
        ("Open W=6", lambda runs, an: [r[1] > 6 for r in runs]),
        ("Open W=4", lambda runs, an: [r[1] > 4 for r in runs]),
        ("Open W=3", lambda runs, an: [r[1] > 3 for r in runs]),
        ("Open W=2", lambda runs, an: [r[1] > 2 for r in runs]),
        ("Open W=1 (all w>=2!)", lambda runs, an: [r[1] > 1 for r in runs]),
        ("Open W=16 + k1(4..15)", lambda runs, an: [(r[1] > 16) or an['k1_4_15_mask'][i] for i, r in enumerate(runs)]),
        ("Open W=16 + k1(2..15)", lambda runs, an: [(r[1] > 16) or an['k1_2_15_mask'][i] for i, r in enumerate(runs)]),
    ]
    print(f"{'Config':22s} | {'n_s':>7s} {'m_ret':>7s} {'/m16':>6s} | {'Nodes':>7s} {'Wide':>4s} {'Steps':>8s} {'Rules':>8s} {'Bands':>8s} | "
          f"{'MaxArK':>6s} {'MaxEdK':>6s} {'MaxBnK':>6s} | {'dStp/dm':>7s} {'dRul/dm':>7s}")
    print("-" * 130)

    base_res = None
    for cname, mask_fn in configs:
        s_ns = s_m = s_nod = s_stp = s_rul = s_bnd = wide_cnt = 0
        max_ar = max_ed = max_bn = 0.0
        for blk, data, full_runs, an in per_blk_runs:
            mask = mask_fn(full_runs, an)
            res = simulate_open_endpoint(blk['n'], blk['primary_index'], full_runs, mask)
            s_ns += res['ns']
            s_m += res['m_ret']
            s_nod += res['open_nodes']
            s_stp += res['steps']
            s_rul += res['rules']
            s_bnd += res['bands']
            if res['is_wide']:
                wide_cnt += 1
            max_ar = max(max_ar, res['arena_kib'])
            max_ed = max(max_ed, res['edges_kib'])
            max_bn = max(max_bn, res['bands_kib'])
        if base_res is None:
            base_res = (s_m, s_stp, s_rul)
            ds_dm = 0.0
            dr_dm = 0.0
        else:
            dm = max(1, base_res[0] - s_m)
            ds_dm = (s_stp - base_res[1]) / dm
            dr_dm = (s_rul - base_res[2]) / dm
        print(f"{cname:22s} | {s_ns:7d} {s_m:7d} {s_m/base_res[0]:5.3f}x | {s_nod:7d} {wide_cnt:2d}/6 {s_stp:8d} {s_rul:8d} {s_bnd:8d} | "
              f"{max_ar:5.0f}K {max_ed:5.0f}K {max_bn:5.0f}K | {ds_dm:7.2f} {dr_dm:7.2f}")


if __name__ == "__main__":
    main()
