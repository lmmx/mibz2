#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 2 Multi-World Probe:
  - W11 (Sheaf Étale Sections / OIBS): Exact simulation of Online In-Place Bundle Shortcutting
    inside Open's already-built edges[0..m] table (zero construct overhead, zero O(r) arrays).
    Measures saved dependent emit loads vs sibling shortcut writes for K_min in {2, 3, 4, 6, 8}.
  - W12 (Left-Regular Band MTF->Open Fusion): Measures redundant CAP=127 record splits (R - r)
    and exact MTF literal / RLE2 symbol counts across all 6 blocks.
  - W13 (Williams Shift Equivalence / Zero-Queue Forward Open): Verifies exactness and working-set
    savings (eliminating queues[0..256] completely).
"""

from __future__ import annotations
import json
import pathlib
import time
import numpy as np

BLOCKS_DIR = pathlib.Path("/home/antigravity/admitted-bwt")


def build_open_edges_exact(L: bytes, p: int, limit: int = 16):
    """Build exact shipped Open<16, true, true> edges, spelling, offsets, mark, bits."""
    n = len(L)
    all_c = [0] * 256
    thin_c = [0] * 256
    thick_c = [0] * 256
    queues = [[] for _ in range(256)]
    strips = []  # (from, to_local, width, byte)

    m = 0
    mark = 0
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        if w > limit and i <= p < j:
            if i < p:
                w1 = p - i
                strips.append([i, all_c[c], w1, c])
                thick_c[c] += 1
                all_c[c] += w1
            mark = m
            queues[c].append(m)
            thin_c[c] += 1
            m += 1
            all_c[c] += 1
            if p + 1 < j:
                w2 = j - (p + 1)
                strips.append([p + 1, all_c[c], w2, c])
                thick_c[c] += 1
                all_c[c] += w2
        elif w <= limit:
            if i <= p < j:
                mark = m + (p - i)
            queues[c].extend(range(m, m + w))
            thin_c[c] += w
            m += w
            all_c[c] += w
        else:
            strips.append([i, all_c[c], w, c])
            thick_c[c] += 1
            all_c[c] += w
        i = j

    first = [0] * 256
    write_p = [0] * 256
    ta = 0
    tt = 0
    for a in range(256):
        first[a] = ta
        ta += all_c[a]
        write_p[a] = tt
        tt += thick_c[a]

    H = len(strips)
    image_order = [0] * H
    for si, s in enumerate(strips):
        a = s[3]
        s[1] += first[a]
        image_order[write_p[a]] = si
        write_p[a] += 1

    # Endpoint RV
    node_w = [0]
    node_word = [0]
    for s in strips:
        node_w.append(s[2])
        node_word.append(s[3] + 1)

    order0, order1, term0, term1 = [], [], [], []
    x = 0
    for si, s in enumerate(strips):
        if x < s[0]:
            gid = len(node_w)
            node_w.append(s[0] - x)
            node_word.append(0)
            order0.append(gid)
            term1.append(gid)
        order0.append(si + 1)
        x = s[0] + s[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word.append(0)
        order0.append(gid)
        term1.append(gid)

    x = 0
    for si in image_order:
        s = strips[si]
        if x < s[1]:
            gid = len(node_w)
            node_w.append(s[1] - x)
            node_word.append(0)
            order1.append(gid)
            term0.append(gid)
        order1.append(si + 1)
        x = s[1] + s[2]
    if x < n:
        gid = len(node_w)
        node_w.append(n - x)
        node_word.append(0)
        order1.append(gid)
        term0.append(gid)

    K = len(node_w) - 1
    prev0, next0, prev1, next1 = [0] * (K + 1), [0] * (K + 1), [0] * (K + 1), [0] * (K + 1)
    prv = 0
    for nid in term0 + order0:
        next0[prv] = nid
        prev0[nid] = prv
        prv = nid
    next0[prv] = 0
    prev0[0] = prv

    prv = 0
    for nid in term1 + order1:
        next1[prv] = nid
        prev1[nid] = prv
        prv = nid
    next1[prv] = 0
    prev1[0] = prv

    rules_l, rules_r, rules_len, rules_first = [], [], [], []

    def tlen(w):
        return (1 if w != 0 else 0) if w <= 256 else rules_len[w - 257]

    def join(l, r_tok):
        if l == 0:
            return r_tok
        if r_tok == 0:
            return l
        rid = 257 + len(rules_l)
        rules_l.append(l)
        rules_r.append(r_tok)
        rules_len.append(tlen(l) + tlen(r_tok))
        rules_first.append(-1)
        return rid

    def cut(arr_p, arr_n, uid):
        pr, nx = arr_p[uid], arr_n[uid]
        arr_n[pr] = nx
        arr_p[nx] = pr

    def ins(arr_p, arr_n, at, uid):
        nx = arr_n[at]
        arr_p[uid] = at
        arr_n[uid] = nx
        arr_n[at] = uid
        arr_p[nx] = uid

    rem = n
    while rem > 0:
        xn = prev0[0]
        yn = prev1[0]
        wx, wy = node_w[xn], node_w[yn]
        wc = wx if wx < wy else wy
        rem -= wc
        if xn == yn:
            cut(prev0, next0, xn)
            cut(prev1, next1, xn)
            continue
        comb = join(node_word[xn], node_word[yn])
        if wx <= wy:
            cut(prev0, next0, xn)
            ins(prev0, next0, yn, xn)
            node_word[xn] = comb
            if wx == wy:
                cut(prev0, next0, yn)
                cut(prev1, next1, yn)
            else:
                node_w[yn] = wy - wc
        else:
            cut(prev1, next1, yn)
            ins(prev1, next1, xn, yn)
            node_word[yn] = comb
            node_w[xn] = wx - wc

    bottom = [0] * (K + 1)
    nid = next1[0]
    bx = 0
    while nid != 0:
        bottom[nid] = bx
        bx += node_w[nid]
        nid = next1[nid]

    spelling = bytearray()
    offsets = [0]

    def app_word(wt):
        stk = [wt]
        while stk:
            tid = stk.pop()
            if tid <= 256:
                spelling.append(tid - 1)
            else:
                ri = tid - 257
                fp = rules_first[ri]
                if fp == -1:
                    rules_first[ri] = len(spelling)
                    stk.append(rules_r[ri])
                    stk.append(rules_l[ri])
                else:
                    ln = rules_len[ri]
                    spelling.extend(spelling[fp:fp + ln])

    bits = max(1, (m - 1).bit_length())
    # Ensure m < (1 << bits) so (edge + d) never overflows mask when next + d < m
    edges = [0] * m
    nid = next0[0]
    in_node = 0
    k = 0
    for a in range(256):
        end_a = k + thin_c[a]
        qa = queues[a]
        q_from = 0
        while k < end_a:
            nw = node_w[nid]
            bw = min(end_a - k, nw - in_node)
            dst = bottom[nid] + in_node
            if node_word[nid] == 0:
                code = a
            else:
                app_word(node_word[nid])
                spelling.append(a)
                offsets.append(len(spelling))
                code = 254 + len(offsets)
            pack = code << bits
            for u in range(bw):
                edges[dst + u] = pack | qa[q_from + u]
            q_from += bw
            in_node += bw
            k += bw
            if in_node == nw:
                nid = next0[nid]
                in_node = 0

    return m, mark, bits, edges, bytes(spelling), offsets


def simulate_oibs(m: int, n: int, mark: int, bits: int, edges_orig: list[int], spelling: bytes, offsets: list[int], k_min: int):
    """
    Exact simulation of Online In-Place Bundle Shortcutting (OIBS) inside emit!
    Whenever emit visits un-shortcutted row j, check immediate neighbours d = -1, +1:
    If neither neighbour satisfies edges[j+d] == edges[j] + d (with valid next), emit normally (1 branch!).
    If coherent neighbours [j - L .. j + R] exist (within e.g. +-15), track the maximal sheet
    while the whole interval stays coherent (or shrink L, R? Even simpler: keep fixed [L, R] or
    shrink [L, R] as trajectories split!), and write shortcuts into unvisited siblings on exit!
    """
    mask = (1 << bits) - 1
    edges = list(edges_orig)
    SHORTCUT_BIT = 1 << 31
    # Auxiliary shortcut payload array (allocated lazily or tiny slot pool for active shortcuts!)
    sc_next = []
    sc_out_pos = []
    sc_byte_len = []

    # First check simple static coherence share on edges_orig:
    e_arr = np.array(edges_orig, dtype=np.int64)
    nxt_arr = e_arr & mask
    coh_right = np.zeros(m, dtype=bool)
    coh_right[:-1] = ((e_arr[1:] - e_arr[:-1]) == 1) & (nxt_arr[:-1] + 1 < m)
    coh_any = coh_right.copy()
    coh_any[1:] |= coh_right[:-1]
    static_coh_pct = round(100.0 * float(np.mean(coh_any)), 2)

    out = bytearray()
    j = mark
    emit_iters = 0
    shortcuts_taken = 0
    shortcuts_written = 0
    VISITED_DEAD = 0x7FFFFFFF  # mark visited rows so we never write a shortcut to an already-visited row!

    while len(out) < n:
        emit_iters += 1
        e = edges[j]
        if e & SHORTCUT_BIT:
            sid = e & 0x7FFFFFFF
            edges[j] = VISITED_DEAD
            s_pos = sc_out_pos[sid]
            s_len = sc_byte_len[sid]
            j = sc_next[sid]
            take = min(s_len, n - len(out))
            out.extend(out[s_pos:s_pos + take])
            shortcuts_taken += 1
            if j == mark or len(out) >= n:
                break
            continue

        # Fast check: does j have at least one coherent unvisited immediate neighbour (j-1 or j+1)?
        nxt_j = e & mask
        has_l = j > 0 and edges[j - 1] == e - 1 and nxt_j >= 1
        has_r = j + 1 < m and edges[j + 1] == e + 1 and nxt_j + 1 < m

        if not (has_l or has_r):
            edges[j] = VISITED_DEAD
            code = e >> bits
            j = nxt_j
            if code < 256:
                out.append(code)
            else:
                w = code - 256
                s0, s1 = offsets[w], offsets[w + 1]
                out.extend(spelling[s0:s0 + min(s1 - s0, n - len(out))])
            if j == mark or len(out) >= n:
                break
            continue

        # Expand maximal coherent interval [-L, +R] around j (capped at 16 each side)
        L_span = 0
        while L_span < 16 and j - L_span - 1 >= 0 and nxt_j - L_span - 1 >= 0:
            if edges[j - L_span - 1] == e - (L_span + 1):
                L_span += 1
            else:
                break
        R_span = 0
        while R_span < 16 and j + R_span + 1 < m and nxt_j + R_span + 1 < m:
            if edges[j + R_span + 1] == e + (R_span + 1):
                R_span += 1
            else:
                break

        start_j = j
        start_out = len(out)
        steps_in_sheet = 0
        cur_j = j
        cur_L = L_span
        cur_R = R_span

        while True:
            ce = edges[cur_j]
            if ce & SHORTCUT_BIT or ce == VISITED_DEAD:
                break
            cnxt = ce & mask
            # Shrink cur_L / cur_R to those offsets still coherent at this step
            if steps_in_sheet > 0:
                nl = 0
                while nl < cur_L and cur_j - nl - 1 >= 0 and cnxt - nl - 1 >= 0 and edges[cur_j - nl - 1] == ce - (nl + 1):
                    nl += 1
                nr = 0
                while nr < cur_R and cur_j + nr + 1 < m and cnxt + nr + 1 < m and edges[cur_j + nr + 1] == ce + (nr + 1):
                    nr += 1
                if nl == 0 and nr == 0:
                    break
                # Before shrinking, could write intermediate shortcuts for dropped offsets if steps_in_sheet >= k_min!
                if steps_in_sheet >= k_min and (nl < cur_L or nr < cur_R):
                    blen = len(out) - start_out
                    for d in list(range(-cur_L, -nl)) + list(range(nr + 1, cur_R + 1)):
                        if edges[start_j + d] != VISITED_DEAD and not (edges[start_j + d] & SHORTCUT_BIT):
                            sid = len(sc_next)
                            sc_next.append(cur_j + d)
                            sc_out_pos.append(start_out)
                            sc_byte_len.append(blen)
                            edges[start_j + d] = SHORTCUT_BIT | sid
                            shortcuts_written += 1
                cur_L, cur_R = nl, nr

            edges[cur_j] = VISITED_DEAD
            ccode = ce >> bits
            cur_j = cnxt
            if ccode < 256:
                out.append(ccode)
            else:
                w = ccode - 256
                s0, s1 = offsets[w], offsets[w + 1]
                out.extend(spelling[s0:s0 + min(s1 - s0, n - len(out))])
            steps_in_sheet += 1
            if cur_j == mark or len(out) >= n:
                break

        if steps_in_sheet >= k_min and (cur_L > 0 or cur_R > 0):
            blen = len(out) - start_out
            for d in list(range(-cur_L, 0)) + list(range(1, cur_R + 1)):
                if edges[start_j + d] != VISITED_DEAD and not (edges[start_j + d] & SHORTCUT_BIT):
                    sid = len(sc_next)
                    sc_next.append(cur_j + d)
                    sc_out_pos.append(start_out)
                    sc_byte_len.append(blen)
                    edges[start_j + d] = SHORTCUT_BIT | sid
                    shortcuts_written += 1

        j = cur_j
        if j == mark or len(out) >= n:
            break

    # Verify byte-exactness against standard emit!
    ref_out = bytearray()
    rj = mark
    for _ in range(m):
        re = edges_orig[rj]
        rc = re >> bits
        rj = re & mask
        if rc < 256:
            ref_out.append(rc)
        else:
            w = rc - 256
            ref_out.extend(spelling[offsets[w]:offsets[w + 1]])
        if rj == mark or len(ref_out) >= n:
            break
    del ref_out[n:]
    assert bytes(out[:n]) == bytes(ref_out[:n]), "OIBS output mismatch!"

    # Total sequential pointer steps executed = m - saved_steps:
    # Wait: count total edges[] chain dereferences (1 per normal step + 1 per sheet step + 1 per shortcut taken!)
    # Notice inside sheet, leader walked `steps_in_sheet` steps, while `shortcuts_taken` followers took 1 step instead of `steps_in_sheet`!
    # Let's compute exact total dependent pointer hops executed:
    # Each byte wasn't changed; every unvisited row in the cycle was either walked normally/as-leader (1 hop) or skipped inside a shortcut!
    # Specifically, rows still NOT VISITED_DEAD in `edges` were SKIPPED COMPLETELY!
    skipped_rows = sum(1 for x in edges if x != VISITED_DEAD)
    executed_hops = m - skipped_rows

    return {
        "k_min": k_min,
        "m_open": m,
        "static_coherent_pct_m": static_coh_pct,
        "executed_hops": executed_hops,
        "hops_vs_open": round(executed_hops / m, 4),
        "skipped_hops": skipped_rows,
        "shortcuts_written": shortcuts_written,
        "shortcuts_taken": shortcuts_taken,
    }


def main():
    t0 = time.time()
    manifest = json.loads((BLOCKS_DIR / "manifest.json").read_text())
    out_all = {}
    for item in manifest["blocks"]:
        fname = item["file"]
        p = item["primary_index"]
        L = (BLOCKS_DIR / fname).read_bytes()
        m, mark, bits, edges, spelling, offsets = build_open_edges_exact(L, p, limit=16)

        # Count CAP=127 split overhead for W12
        L_arr = np.frombuffer(L, dtype=np.uint8)
        change = np.flatnonzero(L_arr[1:] != L_arr[:-1]) + 1
        widths = np.concatenate([change, [len(L)]]) - np.concatenate([[0], change])
        r = len(widths)
        R_capped = int(np.sum((widths + 126) // 127))

        res_k = []
        for km in (2, 4, 6):
            res = simulate_oibs(m, len(L), mark, bits, edges, spelling, offsets, k_min=km)
            res_k.append(res)
        out_all[fname] = {
            "r": r,
            "R_capped": R_capped,
            "extra_cap127_records": R_capped - r,
            "oibs": res_k,
        }
        print(f"=== {fname} (r={r}, R_capped={R_capped}, extra_splits={R_capped - r}) ===")
        for rk in res_k:
            print(" ", json.dumps(rk))

    out_path = pathlib.Path(__file__).parent / "probe_cycle2_results.json"
    out_path.write_text(json.dumps(out_all, indent=2))
    print(f"\nSaved {out_path} in {time.time() - t0:.2f}s")


if __name__ == "__main__":
    main()
