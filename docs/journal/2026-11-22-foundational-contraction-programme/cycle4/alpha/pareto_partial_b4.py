#!/usr/bin/env python3
"""
Cycle 4 — Task 1:
Exact Partial Causal B4 Elimination Pareto (k = 0 .. |Sigma|-1) across ALL 6
authenticated Wikidata blocks (/tmp/mibz2-blocks/block-*.L)!

Implements Beta's Theorem B4 (Zero-Tombstone Causal Routing: min_rank(src, dst) routing,
0% stale pieces, 0 tombstones) stopping after eliminating the first k buckets in causal order:
  - Order 1: `rarest_first` (ascending initial total degree |self|+|out|+|in|, matching B4)
  - Order 2: `pre_k16_then_hub` (non-hex lightest first, then K_16 hex, then top hubs)
and hands the exact remaining multi-bucket W-IET f_{K_k} to Open-style thick-strip reduction + walk!

Measures ALL 5 required Pareto quantities + exact byte-for-byte verification on small cases
AND multiple k checkpoints on all 6 authenticated blocks:
  1. S1_appends(k)/r_0, S1_ops(k)/r_0, max_elim_deg(k) & L1 KiB residency
  2. m_rem(k) (pieces), rows_rem(k), m_rem(k)/m_16, m_thin_rem(k)/m_16 (after width-16 strip reduction)
  3. Remaining edges KiB (4 B/row & 8 B/row), PeakM(k) in KiB (queues + words + edges)
  4. emit_word_len_dist(k): % pieces (and % rows) with len == 1, <=3, <=8, <=16, >16
  5. Complete i9-calibrated cost inequality vs v0.5.12 OpenCells (8.13-10.05 ms / 6 blocks, 0.97 MiB/w)!
"""
import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt


class FlatWordTracker:
    """Tracks exact word lengths, flat-leaf (<=16B) vs SLP (>16B) counts, and exact expansion for verification."""
    __slots__ = ("kind", "a", "b", "length", "inline_bytes", "slp_gt16_cnt", "flat_cnt")

    def __init__(self):
        self.kind = [0] * 256
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.inline_bytes = [bytes([c]) for c in range(256)]
        self.slp_gt16_cnt = 0
        self.flat_cnt = 0

    def concat(self, u, v):
        lu, lv = self.length[u], self.length[v]
        if lu + lv <= 16 and self.kind[u] == 0 and self.kind[v] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu + lv)
            self.inline_bytes.append(self.inline_bytes[u] + self.inline_bytes[v])
            self.flat_cnt += 1
            return idx
        idx = len(self.kind)
        self.kind.append(1); self.a.append(u); self.b.append(v)
        self.length.append(lu + lv)
        self.inline_bytes.append(b"")
        self.slp_gt16_cnt += 1
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        lu = self.length[u]
        if lu * k <= 16 and self.kind[u] == 0:
            idx = len(self.kind)
            self.kind.append(0); self.a.append(0); self.b.append(0)
            self.length.append(lu * k)
            self.inline_bytes.append(self.inline_bytes[u] * k)
            self.flat_cnt += 1
            return idx
        idx = len(self.kind)
        self.kind.append(2); self.a.append(u); self.b.append(k)
        self.length.append(lu * k)
        self.inline_bytes.append(b"")
        self.slp_gt16_cnt += 1
        return idx

    def expand(self, nid):
        if self.kind[nid] == 0:
            return self.inline_bytes[nid]
        out = bytearray()
        st = [(nid, 1)]
        while st:
            u, mult = st.pop()
            k = self.kind[u]
            if k == 0:
                b = self.inline_bytes[u]
                out.extend(b if mult == 1 else b * mult)
            elif k == 1:
                if mult == 1:
                    st.append((self.b[u], 1)); st.append((self.a[u], 1))
                else:
                    out.extend(self.expand(u) * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def verify_remaining_iet_exact(rem_pieces, slp, n, ref_bytes):
    """Exact verification that the remaining multi-bucket W-IET f_{K_k} decodes to ref_bytes."""
    rem_pieces_sorted = sorted(rem_pieces, key=lambda x: x[0])
    starts = [p[0] for p in rem_pieces_sorted]
    p_idx = next(i for i, pc in enumerate(rem_pieces_sorted) if pc[6] >= 0)
    x0 = rem_pieces_sorted[p_idx][0]
    m0 = rem_pieces_sorted[p_idx][6]
    x = x0
    chunks = []
    while True:
        idx = bisect_right(starts, x) - 1
        lo, hi, d, src, dst, w, m = rem_pieces_sorted[idx]
        assert lo <= x < hi
        chunks.append(slp.expand(w))
        x = x + d
        if x == x0:
            break
    cyc = b"".join(chunks)
    q = len(cyc)
    rot = cyc[m0:] + cyc[:m0]
    text = ((rot * (n // q + 1))[:n])[::-1]
    assert text == ref_bytes, f"EXACTNESS FAILURE! len(cyc)={q}, n={n}"


def compute_b4_pareto_for_block(L_bytes, primary_idx, ref_bytes, order_mode="rarest_first", verify_ks=()):
    """
    Execute exact Theorem B4 Zero-Tombstone Causal Elimination step-by-step t = 0 .. sigma-1,
    recording exact Pareto snapshot at EVERY k = 0 .. sigma-1!
    """
    n, r0, m_16, buckets, pieces_by_bucket, bigrams, _ = mt.acquire_from_L(L_bytes, primary_idx)
    slp = FlatWordTracker()
    present = [c for c, _, _ in buckets]
    sigma = len(present)

    # Compute initial piece degree (in + out + self) and row count per symbol
    deg_pcs = defaultdict(int)
    row_cnt = {c: hi - lo for c, lo, hi in buckets}
    for c, _, _ in buckets:
        for lo, hi, d, tsym, w, m in pieces_by_bucket[c]:
            deg_pcs[c] += 1
            if tsym != c:
                deg_pcs[tsym] += 1

    hex_all = set(b"0123456789abcdefABCDEF-")
    top_hubs = {ord('"'), ord(','), ord(':'), ord('{'), ord('}')}

    if order_mode == "rarest_first":
        order = sorted(present, key=lambda c: (deg_pcs[c], row_cnt[c], c))
    elif order_mode == "row_asc":
        order = sorted(present, key=lambda c: (row_cnt[c], deg_pcs[c], c))
    elif order_mode == "pre_k16_then_hub":
        # Group 0: non-hex non-hub (ascending deg_pcs); Group 1: hex_all (ascending deg_pcs); Group 2: top_hubs (ascending deg_pcs)
        def key_fn(c):
            grp = 2 if c in top_hubs else (1 if c in hex_all else 0)
            return (grp, deg_pcs[c], row_cnt[c], c)
        order = sorted(present, key=key_fn)
    else:
        raise ValueError(order_mode)

    rank_of = {c: r for r, c in enumerate(order)}

    # Theorem B4 Zero-Tombstone Causal Queues:
    # Each piece: (lo, hi, delta, src, dst, w, mark_off)
    self_q = [[] for _ in range(sigma)]
    out_q = [[] for _ in range(sigma)]
    in_q = [[] for _ in range(sigma)]

    s1_appends = 0
    s1_ops = 0  # compositions (lookups) in Stage 1
    s1_self_ops = 0
    s1_compose_ops = 0
    live_pieces = 0
    peak_live_pieces = 0

    def route_piece(pc):
        nonlocal s1_appends, live_pieces, peak_live_pieces
        lo, hi, d, src, dst, w, m = pc
        rs, rd = rank_of[src], rank_of[dst]
        s1_appends += 1
        live_pieces += 1
        if live_pieces > peak_live_pieces:
            peak_live_pieces = live_pieces
        if rs == rd:
            self_q[rs].append(pc)
        elif rs < rd:
            out_q[rs].append(pc)
        else:
            in_q[rd].append(pc)

    # Initial causal routing directly from I_0 (counted in initial_appends, NOT Stage 1 elimination appends!)
    init_appends = 0
    for c, _, _ in buckets:
        for lo, hi, d, tsym, w, m in pieces_by_bucket[c]:
            route_piece((lo, hi, d, c, tsym, w, m))
            init_appends += 1
    # Separate initial routing from elimination-generated appends!
    s1_appends_elim = 0  # tracks appends created DURING steps 0..k-1

    max_elim_deg = 0
    max_elim_phi_pcs = 0
    snapshots = {}

    def take_snapshot(k):
        # Gather exact surviving multi-bucket W-IET f_{K_k} = union_{t >= k} (self_q[t] + out_q[t] + in_q[t])
        rem_pcs = []
        for t in range(k, sigma):
            rem_pcs.extend(self_q[t])
            rem_pcs.extend(out_q[t])
            rem_pcs.extend(in_q[t])

        if k in verify_ks and ref_bytes is not None:
            verify_remaining_iet_exact(rem_pcs, slp, n, ref_bytes)

        m_rem = len(rem_pcs)
        rows_rem = 0
        m_thin_rem = 0
        n_thick_rem = 0
        thick_rows_rem = 0
        # Word length distribution (by pieces AND by thin rows!)
        p_l1 = p_l3 = p_l8 = p_l16 = p_gt16 = 0
        r_l1 = r_l3 = r_l8 = r_l16 = r_gt16 = 0
        has_p_in_thin = False
        for lo, hi, d, src, dst, w, m in rem_pcs:
            width = hi - lo
            rows_rem += width
            wlen = slp.length[w]
            if width <= 16:
                m_thin_rem += width
                if m >= 0:
                    has_p_in_thin = True
            else:
                n_thick_rem += 1
                thick_rows_rem += width
            if wlen == 1: p_l1 += 1; r_l1 += width
            if wlen <= 3: p_l3 += 1; r_l3 += width
            if wlen <= 8: p_l8 += 1; r_l8 += width
            if wlen <= 16: p_l16 += 1; r_l16 += width
            else: p_gt16 += 1; r_gt16 += width
        if not has_p_in_thin:
            m_thin_rem += 1

        # Memory footprints in KiB:
        #   - Remaining dense edges: 4 B/row (Open-style u32) or 8 B/row (next_row u32 + word_id u32)
        #   - Stage 1 B4 peak live queues: peak_live_pieces * 24 B
        #   - Word store: flat_cnt * 17 B + slp_gt16_cnt * 12 B
        edges4_kib = (m_thin_rem * 4) / 1024.0
        edges8_kib = (m_thin_rem * 8) / 1024.0
        b4_q_kib = (peak_live_pieces * 24) / 1024.0
        word_kib = (slp.flat_cnt * 17 + slp.slp_gt16_cnt * 12) / 1024.0
        peak_m_kib = b4_q_kib + word_kib + edges8_kib

        snapshots[k] = {
            "k": k,
            "rem_syms": sigma - k,
            "elim_sym": order[k - 1] if k > 0 else -1,
            "r0": r0, "m16": m_16, "n": n, "sigma": sigma,
            "s1_appends_tot": s1_appends,          # includes initial routing (1.00 r_0) + elim appends
            "s1_appends_elim": s1_appends - init_appends,
            "s1_ops": s1_ops,                      # total compositions (self + in->phi)
            "s1_self_ops": s1_self_ops,
            "s1_compose_ops": s1_compose_ops,
            "max_elim_deg": max_elim_deg,
            "max_elim_phi_pcs": max_elim_phi_pcs,
            "m_rem_pcs": m_rem,
            "rows_rem": rows_rem,
            "m_thin_rem": m_thin_rem,
            "n_thick_rem": n_thick_rem,
            "thick_rows_rem": thick_rows_rem,
            "edges4_kib": edges4_kib,
            "edges8_kib": edges8_kib,
            "b4_q_kib": b4_q_kib,
            "word_kib": word_kib,
            "peak_m_kib": peak_m_kib,
            "flat_words": slp.flat_cnt,
            "slp_gt16": slp.slp_gt16_cnt,
            "pct_p_l1": 100.0 * p_l1 / max(1, m_rem),
            "pct_p_l3": 100.0 * p_l3 / max(1, m_rem),
            "pct_p_l8": 100.0 * p_l8 / max(1, m_rem),
            "pct_p_l16": 100.0 * p_l16 / max(1, m_rem),
            "pct_p_gt16": 100.0 * p_gt16 / max(1, m_rem),
            "pct_r_l16": 100.0 * r_l16 / max(1, rows_rem),
        }

    # Snapshot at k = 0 (no buckets eliminated = Open baseline!)
    take_snapshot(0)

    # Eliminate buckets t = 0 .. sigma - 2 (stopping at t = sigma - 1 leaves the final single hub!)
    for t in range(sigma - 1):
        c = order[t]
        sq = self_q[t]
        oq = out_q[t]
        iq = in_q[t]
        deg_here = len(sq) + len(oq) + len(iq)
        if deg_here > max_elim_deg:
            max_elim_deg = deg_here

        # Remove consumed pieces from live count
        live_pieces -= (len(sq) + len(oq) + len(iq))

        # Step 1: Resolve self-loops inside bucket c -> exact Phi_c: In(F_c) -> Out(F_c)
        # (Every piece in oq already exits c to rank > t; every piece in sq stays inside c!)
        if not sq:
            phi_c = sorted(oq, key=lambda x: x[0])
        else:
            # Collapse immediate self-overlaps in sq (O(1) spiral division)
            col = []
            for lo, hi, d, src, dst, w, m in oq:
                col.append((lo, hi, d, dst, w, m, False))
            for lo, hi, d, src, dst, w, m in sq:
                if d == 0:
                    continue
                L = hi - lo
                if d > 0 and d < L:
                    q_sp, rem = divmod(L, d)
                    if rem > 0: col.append((lo, lo + rem, (q_sp + 1) * d, dst, slp.power(w, q_sp + 1), -1, True))
                    if d - rem > 0: col.append((lo + rem, lo + d, q_sp * d, dst, slp.power(w, q_sp), -1, True))
                elif d < 0 and (-d) < L:
                    s_sp = -d; q_sp, rem = divmod(L, s_sp)
                    if s_sp - rem > 0: col.append((hi - s_sp, hi - rem, -q_sp * s_sp, dst, slp.power(w, q_sp), -1, True))
                    if rem > 0: col.append((hi - rem, hi, -(q_sp + 1) * s_sp, dst, slp.power(w, q_sp + 1), -1, True))
                else:
                    col.append((lo, hi, d, dst, w, m, True))
            col.sort(key=lambda x: x[0])
            c_starts = [p[0] for p in col]
            int_t = sorted((lo + d, hi + d) for lo, hi, d, dst, w, m, is_i in col if is_i)
            mt_list = []
            for a, b in int_t:
                if mt_list and mt_list[-1][1] == a: mt_list[-1] = (mt_list[-1][0], b)
                else: mt_list.append((a, b))
            entry = []
            mi = 0; nm = len(mt_list)
            for lo, hi, d, dst, w, m, is_i in col:
                pos = lo
                while pos < hi:
                    while mi < nm and mt_list[mi][1] <= pos: mi += 1
                    if mi >= nm or mt_list[mi][0] >= hi:
                        entry.append((pos, hi, d, dst, w, m, is_i)); pos = hi
                    else:
                        ta, tb = mt_list[mi]
                        if pos < ta: entry.append((pos, ta, d, dst, w, m, is_i)); pos = ta
                        pos = hi if hi < tb else tb
            phi_c = []
            st = list(reversed(entry))
            while st:
                lo, hi, d, dst, w, m, is_i = st.pop()
                if not is_i:
                    phi_c.append((lo, hi, d, c, dst, w, m))
                    continue
                t_lo, t_hi = lo + d, hi + d
                idx = bisect_right(c_starts, t_lo) - 1
                pos = t_lo; wlen = slp.length[w]
                nxt = []
                while pos < t_hi:
                    s1_ops += 1; s1_self_ops += 1
                    plo, phi_, pd, pdst, pw, pm, pi = col[idx]
                    ov = t_hi if t_hi < phi_ else phi_
                    nxt.append((pos - d, ov - d, d + pd, pdst, slp.concat(w, pw), mt.combine_mark(m, wlen, pm), pi))
                    pos = ov; idx += 1
                for item in reversed(nxt):
                    st.append(item)
            phi_c.sort(key=lambda x: x[0])

        if len(phi_c) > max_elim_phi_pcs:
            max_elim_phi_pcs = len(phi_c)

        # Step 2: Compose EVERY incoming piece in iq (rank(src) > t, dst == c) through phi_c
        # and route directly to min(rank(src), rank(new_dst)) > t!
        phi_starts = [p[0] for p in phi_c]
        for lo, hi, d, src, dst, w, m in iq:
            t_lo, t_hi = lo + d, hi + d
            idx = bisect_right(phi_starts, t_lo) - 1
            pos = t_lo; wlen = slp.length[w]
            while pos < t_hi:
                s1_ops += 1; s1_compose_ops += 1
                plo, phi_, pd, psrc, pdst, pw, pm = phi_c[idx]
                ov = t_hi if t_hi < phi_ else phi_
                new_pc = (pos - d, ov - d, d + pd, src, pdst, slp.concat(w, pw), mt.combine_mark(m, wlen, pm))
                route_piece(new_pc)
                pos = ov; idx += 1

        # Free bucket t completely (0% stale, 0 tombstones!)
        self_q[t] = []; out_q[t] = []; in_q[t] = []

        # Take snapshot at k = t + 1!
        take_snapshot(t + 1)

    return snapshots, order


def main():
    # 1. Exhaustive small verification of Partial Causal B4 at ALL k = 0..sigma-1!
    print("=== 1. Exhaustive Exactness Verification of Partial Causal B4 (all k = 0..sigma-1) ===")
    tot_small = 0
    for n in range(2, 9):
        for mask in range(1 << n):
            T = bytes((mask >> i) & 1 for i in range(n))
            if len(set(T)) < 2: continue
            L, p = mt.bwt_encode(T)
            compute_b4_pareto_for_block(L, p, T, "rarest_first", verify_ks=(0, 1))
            tot_small += 2
    for n in range(3, 6):
        for code in range(3 ** n):
            T = bytes((code // (3 ** i)) % 3 for i in range(n))
            if len(set(T)) < 3: continue
            L, p = mt.bwt_encode(T)
            compute_b4_pareto_for_block(L, p, T, "rarest_first", verify_ks=(0, 1, 2))
            tot_small += 3
    print(f"  EXHAUSTIVE PARTIAL-B4 PASS: {tot_small} (string, k) checkpoints verified byte-for-byte exact [verified]")

    # 2. All 6 Authenticated Wikidata Blocks!
    print("\n=== 2. PARTIAL CAUSAL B4 PARETO ACROSS ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    all_snaps = {}
    for mode in ("rarest_first", "pre_k16_then_hub"):
        print(f"\n==================== ORDER: {mode} ====================")
        all_snaps[mode] = {}
        for entry in manifest["blocks"]:
            with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
                L = f.read()
            assert hashlib.sha256(L).hexdigest() == entry["sha256"]
            p = entry["primary_index"]
            ref = mt.ref_ibwt(L, p)
            sig_est = len(set(L))
            # Verify exact decoding at k = 0, sig//2, sig-20, sig-1!
            vks = {0, sig_est // 2, max(0, sig_est - 20), sig_est - 1}
            snaps, order = compute_b4_pareto_for_block(L, p, ref, mode, verify_ks=vks)
            all_snaps[mode][entry["file"]] = (snaps, order)
            s_end = snaps[sig_est - 1]
            print(f"  {entry['file']:11s} (sig={sig_est:2d}, r0={s_end['r0']:6d}, m16={s_end['m16']:6d}): "
                  f"full B4 k={sig_est-1} -> app={s_end['s1_appends_tot']/s_end['r0']:.2f}r "
                  f"ops={s_end['s1_ops']/s_end['r0']:.2f}r m_thin={s_end['m_thin_rem']/s_end['m16']:.3f}m16 "
                  f"last_hub={repr(chr(order[-1]))} [4 checkpoints EXACT]", flush=True)

    # 3. Print detailed Pareto frontier on block-0.L (rarest_first & pre_k16_then_hub)
    #    AND 6-block aggregated Pareto frontier by remaining-symbol count / deg_cap!
    print("\n=== 3. DETAILED PARETO FRONTIER ON block-0.L (rarest_first, sig=89, r0=51,184, m16=89,874) ===")
    print("  k | rem | maxDeg(L1KB) | app/r | ops/r | P_rem  | m_th/m16 | e8KB | PkM_KB | <=1B <=3B <=8B <=16B >16B | i9_est_ms")
    print("----+-----+--------------+-------+-------+--------+----------+------+--------+---------------------------+----------")
    sn0, ord0 = all_snaps["rarest_first"]["block-0.L"]
    sig0 = len(ord0)
    # Show key k values: 0, 20, 40, 50, 60, 65, 70, 75, 78, 80, 82, 84, 86, 87, 88
    ks_show = [0, 20, 40, 50, 60, 65, 70, 75, 78, 80, 82, 84, 86, 87, sig0 - 1]
    for k in ks_show:
        s = sn0[k]
        r0 = s["r0"]; m16 = s["m16"]
        # i9 symbolic cost model (calibrated against Open = 0.81 ns*n + 5.3 ns*m_16 = 1.21 ms on block-0.L):
        #   Open construct = 0.72-0.90 ns * n (here 0.81 ns * 900k = 0.73 ms); Open walk = 5.3 ns * m_16 = 0.48 ms -> 1.21 ms!
        #   Partial B4 + Open-style residual:
        #     - Stage 1 B4: ~25 ns per s1_ops + ~10 ns per s1_appends_elim (optimistic L1-resident bucket-Schur!)
        #     - Residual Open construct (thick strips on P_rem): ~0.35 ms + 5 ns * P_rem (scaled from Open)
        #     - Residual walk + flat emit: ~5.3 ns * m_thin_rem (+ ~12 ns * slp_gt16 if > 0)
        # Let's keep symbolic formula completely transparent and report raw counts FIRST!
        l1_kb = s["max_elim_deg"] * 24 / 1024.0
        # Conservative lower-bound i9 time: 15 ns * s1_ops + 5.3 ns * m_thin_rem + 0.30 ms fixed scan/write
        est_lb = (15.0 * s["s1_ops"] + 5.3 * s["m_thin_rem"]) * 1e-6 + 0.30
        print(f" {k:2d} |  {s['rem_syms']:2d} | {s['max_elim_deg']:6d}({l1_kb:4.0f}K) | "
              f"{s['s1_appends_tot']/r0:5.2f} | {s['s1_ops']/r0:5.2f} | {s['m_rem_pcs']:6d} | "
              f"  {s['m_thin_rem']/m16:5.3f}  | {s['edges8_kib']:4.0f} | {s['peak_m_kib']:6.0f} | "
              f"{s['pct_p_l1']:4.0f} {s['pct_p_l3']:4.0f} {s['pct_p_l8']:4.0f} {s['pct_p_l16']:5.1f} {s['pct_p_gt16']:4.1f} | "
              f"{est_lb:5.2f} ms")

    # 4. 6-Block Aggregated Pareto by `deg_cap` threshold AND by `rem_syms` (rarest_first & pre_k16_then_hub)!
    print("\n=== 4. 6-BLOCK AGGREGATED PARETO BY deg_cap THRESHOLD (rarest_first, r0_sum=509,269, m16_sum=895,391) ===")
    print("deg_cap  | avg_k/sig | app/r | ops/r | P_rem/r | m_th/m16 | tot_ops+m_th (/r0, /m16) | e8_avgKB | PkM_avgKB | <=16B% | slp>16")
    print("---------+-----------+-------+-------+---------+----------+--------------------------+----------+-----------+--------+-------")
    caps = [0, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 10**9]
    for cap in caps:
        sum_app = sum_ops = sum_prem = sum_mth = sum_slp = 0
        sum_k = sum_sig = 0
        sum_e8 = sum_pkm = sum_p16 = 0.0
        for fn, (snaps, order) in all_snaps["rarest_first"].items():
            sig = len(order)
            # Largest k such that max_elim_deg <= cap (and k <= sig - 1)
            best_k = 0
            for k in range(sig):
                if snaps[k]["max_elim_deg"] <= cap:
                    best_k = k
                else:
                    break
            s = snaps[best_k]
            sum_app += s["s1_appends_tot"]
            sum_ops += s["s1_ops"]
            sum_prem += s["m_rem_pcs"]
            sum_mth += s["m_thin_rem"]
            sum_slp += s["slp_gt16"]
            sum_k += best_k; sum_sig += sig
            sum_e8 += s["edges8_kib"]; sum_pkm += s["peak_m_kib"]
            sum_p16 += s["pct_p_l16"]
        r0s = 509269; m16s = 895391
        comb = sum_ops + sum_mth
        lbl = f"<= {cap:5d}" if cap < 10**8 else "FULL B4 "
        print(f"{lbl} | {sum_k/6:4.1f}/{sum_sig/6:4.1f} | {sum_app/r0s:5.2f} | {sum_ops/r0s:5.2f} | "
              f" {sum_prem/r0s:6.3f} |   {sum_mth/m16s:5.3f}  | {comb:7d} ({comb/r0s:4.2f}r,{comb/m16s:4.2f}m) | "
              f"  {sum_e8/6:5.0f}K |   {sum_pkm/6:6.0f}K | {sum_p16/6:5.1f}% | {sum_slp:6d}")

    print("\n=== 5. 6-BLOCK AGGREGATED PARETO: pre_k16_then_hub (Targeted Stops: pre-K16, post-K16/pre-5hubs, FULL) ===")
    for label, target_rem in [("Open (k=0)", 999), ("Pre-K16 (rem=28: K16+5hubs)", 28),
                              ("Post-K16 / Top-5 Hubs (rem=5)", 5), ("Top-3 Hubs (rem=3)", 3),
                              ("Top-2 Hubs (rem=2)", 2), ("Single Hub '\"' (rem=1)", 1)]:
        sum_app = sum_ops = sum_prem = sum_mth = sum_slp = 0
        sum_e8 = sum_pkm = sum_p16 = 0.0
        for fn, (snaps, order) in all_snaps["pre_k16_then_hub"].items():
            sig = len(order)
            k = 0 if target_rem == 999 else max(0, sig - target_rem)
            s = snaps[k]
            sum_app += s["s1_appends_tot"]; sum_ops += s["s1_ops"]
            sum_prem += s["m_rem_pcs"]; sum_mth += s["m_thin_rem"]
            sum_slp += s["slp_gt16"]
            sum_e8 += s["edges8_kib"]; sum_pkm += s["peak_m_kib"]; sum_p16 += s["pct_p_l16"]
        r0s = 509269; m16s = 895391
        comb = sum_ops + sum_mth
        print(f"  {label:30s} | app={sum_app/r0s:4.2f}r ops={sum_ops/r0s:4.2f}r P_rem={sum_prem/r0s:5.3f}r "
              f"m_th={sum_mth/m16s:5.3f}m16 | ops+m_th={comb/r0s:4.2f}r({comb/m16s:4.2f}m16) | "
              f"e8={sum_e8/6:4.0f}K PkM={sum_pkm/6:5.0f}K <=16B={sum_p16/6:5.1f}% slp={sum_slp:6d}")

    # Save JSON for clean reference
    out_summary = {mode: {fn: {str(k): v for k, v in snaps.items()}
                          for fn, (snaps, _) in d.items()}
                   for mode, d in all_snaps.items()}
    with open("/tmp/fcp-a/pareto_b4_6blocks.json", "w") as f:
        json.dump(out_summary, f)
    print("\nSaved full Pareto JSON to /tmp/fcp-a/pareto_b4_6blocks.json")


if __name__ == "__main__":
    main()
