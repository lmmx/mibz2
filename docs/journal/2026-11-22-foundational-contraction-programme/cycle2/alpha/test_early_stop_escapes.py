#!/usr/bin/env python3
"""
Cycle 2 — Script 2 (Escape A):
Early-Stop Compositional Redheffer BEFORE the K_16 GUID Hex Clique
+ Comparison against Gamma's K = {'"'} hub + OpenCells (m_16) + Flat-Leaf <= 16B!

Mathematical construction:
Given ANY retained hub subset K subset Sigma (and eliminated set E = Sigma \ K):
1. Run Compositional Redheffer Star Product ONLY on E to compute Phi_E: In(U_E) -> Out(U_E) subset U_K!
   (Zero merges touch K! Every piece of Phi_E has target_sym in K!)
2. Compose every piece of K whose target lies in E with Phi_E (1-shot elimination of E into K!).
   Result: The EXACT induced first-return W-IET f_K: U_K -> U_K (with SLP / Flat-Leaf words)!
3. Measure TWO exact emission backends on f_K:
   (i)  Direct dense walk on all |U_K| rows of f_K (m_raw = |U_K|), AND
   (ii) Post-contraction Thick-Strip Induction (Open-style width > 16 or width > 1 strip collapse)
        on f_K -> dense walk on ONLY the thin retained rows m_thin(K) (analogous to Gamma's Core-RV)!
   AND verify 100% byte-for-byte exact decoding on ALL 6 authenticated Wikidata blocks!
4. Also sweep early-stop cutoffs along our `width_times_rows` merge tree (keeping the last k_rem
   clusters unmerged, including k_rem = 1 (Gamma '"'), hex_lo, hex_all, etc.) and measure
   Beta's Flat-Leaf (<= 16 B inline pool) compression ratio!
"""
import hashlib
import json
import sys
import time
from bisect import bisect_right
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import measure_all_trees as mt
import test_leverage_and_lb as tl


class FlatSlpStore:
    """
    Beta-inspired Flat-Leaf (<= 16 B inline) + Memo-SLP store:
    Any word of length <= 16 bytes is stored inline (0 SLP node, 1 flat 16B copy at emit!).
    Only words of length > 16 bytes become SLP concat/power nodes!
    """
    __slots__ = ("kind", "a", "b", "length", "inline_bytes", "slp_nodes_created", "inline_merges")

    def __init__(self):
        # IDs 0..255 are 1-byte literals (inline!)
        self.kind = [0] * 256       # 0 = inline bytes (len <= 16), 1 = concat, 2 = power
        self.a = list(range(256))
        self.b = [0] * 256
        self.length = [1] * 256
        self.inline_bytes = [bytes([c]) for c in range(256)]
        self.slp_nodes_created = 0
        self.inline_merges = 0

    def concat(self, u, v):
        lu = self.length[u]
        lv = self.length[v]
        if lu + lv <= 16 and self.kind[u] == 0 and self.kind[v] == 0:
            # Inline flat merge! Zero SLP hierarchy!
            idx = len(self.kind)
            self.kind.append(0)
            self.a.append(0)
            self.b.append(0)
            self.length.append(lu + lv)
            self.inline_bytes.append(self.inline_bytes[u] + self.inline_bytes[v])
            self.inline_merges += 1
            return idx
        idx = len(self.kind)
        self.kind.append(1)
        self.a.append(u)
        self.b.append(v)
        self.length.append(lu + lv)
        self.inline_bytes.append(b"")
        self.slp_nodes_created += 1
        return idx

    def power(self, u, k):
        if k == 1:
            return u
        lu = self.length[u]
        if lu * k <= 16 and self.kind[u] == 0:
            idx = len(self.kind)
            self.kind.append(0)
            self.a.append(0)
            self.b.append(0)
            self.length.append(lu * k)
            self.inline_bytes.append(self.inline_bytes[u] * k)
            self.inline_merges += 1
            return idx
        idx = len(self.kind)
        self.kind.append(2)
        self.a.append(u)
        self.b.append(k)
        self.length.append(lu * k)
        self.inline_bytes.append(b"")
        self.slp_nodes_created += 1
        return idx

    def expand_word(self, nid, memo_out=None):
        """Expand a single word (used for residual step words)."""
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
                    st.append((self.b[u], 1))
                    st.append((self.a[u], 1))
                else:
                    sub = self.expand_word(u)
                    out.extend(sub * mult)
            else:
                st.append((self.a[u], self.b[u] * mult))
        return bytes(out)


def run_early_stop_K(L_bytes, primary_idx, K_set, verify_exact=True):
    """
    Exact Early-Stop Redheffer stopping at retained alphabet K_set subset Sigma:
    1. Build leverage tree over E = Sigma \ K_set (if E non-empty), compute Phi_E.
    2. Compose K_set pieces targeting E through Phi_E to produce exact first-return W-IET f_K on U_K.
    3. Measure:
       - Q_tree: lookups inside E merges
       - Q_compose_K: lookups composing K -> E through Phi_E
       - Q_total_con = Q_tree + Q_compose_K
       - P_K: number of pieces of f_K on U_K
       - m_raw = |U_K| (total rows in U_K)
       - m_thin16 = rows of f_K in pieces of width <= 16 (plus primary row!)
       - m_thin1  = rows of f_K in pieces of width == 1
       - Flat-Leaf stats: fraction of f_K steps / pieces that are inline <= 16 B!
    4. Verify 100% exact decoding via dense walk on f_K (if verify_exact)!
    """
    n, r0, m_16, buckets, pieces_by_bucket, bigrams, _ = mt.acquire_from_L(L_bytes, primary_idx)
    # Use FlatSlpStore!
    slp = FlatSlpStore()
    stats = defaultdict(int)
    present_syms = {c for c, _, _ in buckets}
    K = K_set & present_syms
    E = present_syms - K

    # Leaf resolution for E symbols ONLY (K symbols keep their raw pieces, or self-run resolved?
    # Keep K raw so we don't even touch K until 1-shot composition with Phi_E!)
    e_buckets = [(c, lo, hi) for c, lo, hi in buckets if c in E]
    k_buckets = [(c, lo, hi) for c, lo, hi in buckets if c in K]

    primary_cycle = None
    if len(E) > 0:
        leaves_E = {}
        for c, b_lo, b_hi in e_buckets:
            s = frozenset([c])
            out_p, pcyc = mt.resolve_subset_ablation(s, pieces_by_bucket[c], [], slp, stats, ping_mode="P1")
            if pcyc is not None:
                primary_cycle = pcyc
            leaves_E[c] = (s, out_p)

        # Build leverage tree over E ONLY!
        if len(e_buckets) == 1:
            phi_E = leaves_E[e_buckets[0][0]][1]
        else:
            # Build width_times_rows tree over e_buckets (without isolating rarest at root since E is open!)
            tree_E = build_open_leverage_tree(e_buckets, pieces_by_bucket)
            def ev(node):
                nonlocal primary_cycle
                if isinstance(node, int):
                    return leaves_E[node]
                lr = ev(node[0]); rr = ev(node[1])
                s = lr[0] | rr[0]
                op, pc = mt.resolve_subset_ablation(s, lr[1], rr[1], slp, stats, ping_mode="P1")
                if pc is not None:
                    primary_cycle = pc
                return (s, op)
            _, phi_E = ev(tree_E)
    else:
        phi_E = []

    q_tree = stats["Q_rand"]
    w_seq_tree = stats["W_seq"]

    # Step 2: Compose K's pieces targeting E through phi_E -> exact first-return f_K on U_K!
    phi_E.sort(key=lambda x: x[0])
    e_starts = [p[0] for p in phi_E]

    f_K_pieces = []
    q_comp_K = 0
    for c, b_lo, b_hi in k_buckets:
        for lo, hi, d, tsym, w, m in pieces_by_bucket[c]:
            if tsym in K:
                f_K_pieces.append((lo, hi, d, tsym, w, m))
            else:
                # Target [lo+d, hi+d) lies in In(U_E)! Look up in phi_E!
                t_lo, t_hi = lo + d, hi + d
                idx = bisect_right(e_starts, t_lo) - 1
                pos = t_lo
                w_len = slp.length[w]
                while pos < t_hi:
                    q_comp_K += 1
                    p_lo, p_hi, p_d, p_tsym, p_w, p_m = phi_E[idx]
                    assert p_lo <= pos < p_hi and p_tsym in K
                    ov_hi = t_hi if t_hi < p_hi else p_hi
                    f_K_pieces.append((
                        pos - d, ov_hi - d, d + p_d, p_tsym,
                        slp.concat(w, p_w), mt.combine_mark(m, w_len, p_m)
                    ))
                    pos = ov_hi
                    idx += 1

    f_K_pieces.sort(key=lambda x: x[0])

    # Measure exact statistics of f_K!
    m_raw = sum(hi - lo for lo, hi, *_ in f_K_pieces)
    p_K = len(f_K_pieces)
    m_thin16 = 0
    m_thin1 = 0
    inline_pieces = 0
    inline_rows = 0
    non_inline_rows = 0
    p_piece_found = False

    for lo, hi, d, tsym, w, m in f_K_pieces:
        ln = hi - lo
        if ln <= 16:
            m_thin16 += ln
        if ln == 1:
            m_thin1 += 1
        if slp.kind[w] == 0:
            inline_pieces += 1
            inline_rows += ln
        else:
            non_inline_rows += ln
        if m >= 0:
            p_piece_found = True

    # Verify exact decoding by walking f_K from the marked piece (if verify_exact)!
    if verify_exact:
        assert p_piece_found, "Primary mark lost!"
        # Map U_K coordinates densely 0..m_raw-1 (just like a dense reduced array!)
        # Actually verify directly on f_K_pieces via bisect on f_K_starts (exact first-return map)
        fk_starts = [p[0] for p in f_K_pieces]
        p_idx_piece = next(i for i, pc in enumerate(f_K_pieces) if pc[5] >= 0)
        x0 = f_K_pieces[p_idx_piece][0]
        m0 = f_K_pieces[p_idx_piece][5]
        x = x0
        chunks = []
        steps = 0
        while True:
            idx = bisect_right(fk_starts, x) - 1
            lo, hi, d, tsym, w, m = f_K_pieces[idx]
            assert lo <= x < hi
            chunks.append(slp.expand_word(w))
            x = x + d
            steps += 1
            if x == x0:
                break
        cyc = b"".join(chunks)
        assert len(cyc) == n, f"Cycle len {len(cyc)} != {n}"
        rot = cyc[m0:] + cyc[:m0]
        text = rot[::-1]
        assert steps == m_raw or n % len(cyc) == 0
    else:
        text = None

    return text, {
        "r0": r0, "m16": m_16, "K_size": len(K), "E_size": len(E),
        "Q_tree": q_tree, "Q_comp_K": q_comp_K, "Q_con": q_tree + q_comp_K,
        "W_seq_tree": w_seq_tree,
        "P_E": len(phi_E), "P_K": p_K,
        "m_raw": m_raw, "m_thin16": m_thin16, "m_thin1": m_thin1,
        "inline_pct_pcs": 100.0 * inline_pieces / max(1, p_K),
        "inline_pct_rows": 100.0 * inline_rows / max(1, m_raw),
        "slp_noninline_nodes": slp.slp_nodes_created,
        "inline_merges": slp.inline_merges,
    }


def build_open_leverage_tree(e_buckets, pieces_by_bucket):
    """Build width_times_rows tree over E (no rarest-root isolation since E is open!)."""
    syms = [c for c, _, _ in e_buckets]
    sym_to_id = {c: i for i, c in enumerate(syms)}
    row_adj = defaultdict(lambda: defaultdict(int))
    pcs_adj = defaultdict(lambda: defaultdict(int))
    sz = {}
    for c, _, _ in e_buckets:
        u = sym_to_id[c]
        sz[u] = len(pieces_by_bucket[c])
        for lo, hi, d, tsym, w, m in pieces_by_bucket[c]:
            if tsym != c and tsym in sym_to_id:
                v = sym_to_id[tsym]
                row_adj[u][v] += (hi - lo); row_adj[v][u] += (hi - lo)
                pcs_adj[u][v] += 1; pcs_adj[v][u] += 1
    active = {i: syms[i] for i in range(len(syms))}
    nxt_id = len(syms)
    while len(active) > 1:
        best_pair = None
        best_sc = -1.0
        cids = list(active.keys())
        for u in cids:
            for v, rwt in row_adj[u].items():
                if v > u and v in active:
                    pwt = pcs_adj[u][v]
                    sc = (rwt * rwt) / ((pwt + 1.0) * (sz[u] + sz[v] + 1.0))
                    if sc > best_sc:
                        best_sc = sc; best_pair = (u, v)
        if best_pair is None:
            best_pair = (cids[0], cids[1])
        u, v = best_pair
        tu, tv = active.pop(u), active.pop(v)
        su, sv = sz.pop(u), sz.pop(v)
        cid = nxt_id; nxt_id += 1
        active[cid] = (tu, tv); sz[cid] = su + sv
        nr = defaultdict(int); np = defaultdict(int)
        for nb, wt in row_adj[u].items():
            if nb != v and nb in active:
                nr[nb] += wt; np[nb] += pcs_adj[u][nb]
        for nb, wt in row_adj[v].items():
            if nb != u and nb in active:
                nr[nb] += wt; np[nb] += pcs_adj[v][nb]
        row_adj.pop(u, None); row_adj.pop(v, None)
        pcs_adj.pop(u, None); pcs_adj.pop(v, None)
        for nb in nr:
            row_adj[cid][nb] = row_adj[nb][cid] = nr[nb]
            pcs_adj[cid][nb] = pcs_adj[nb][cid] = np[nb]
            row_adj[nb].pop(u, None); row_adj[nb].pop(v, None)
            pcs_adj[nb].pop(u, None); pcs_adj[nb].pop(v, None)
    return next(iter(active.values()))


def main():
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    hex_lo = set(b"0123456789abcdef-")
    hex_all = set(b"0123456789abcdefABCDEF-")
    quote_hub = {ord('"')}
    # Also test minimal GUID-separator hubs: {'"'} vs {'"', '-'} vs hex_lo vs hex_all vs {top-4 hubs}
    hub_configs = [
        ("K_quote_Gamma", quote_hub),
        ("K_quote_dash", {ord('"'), ord('-')}),
        ("K_quote_colon_comma", {ord('"'), ord(':'), ord(',')}),
        ("K_hex_lo", hex_lo),
        ("K_hex_all", hex_all),
        ("K_digits_only", set(b"0123456789")),
    ]

    print("=== ESCAPE (A): EARLY-STOP REDHEFFER ACROSS HUB SETS K* ON ALL 6 WIKIDATA BLOCKS ===")
    for entry in manifest["blocks"]:
        with open(f"/tmp/mibz2-blocks/{entry['file']}", "rb") as f:
            L = f.read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        ref = mt.ref_ibwt(L, p)
        print(f"\n--- {entry['file']} (n={entry['n']}, r0={entry['r']}) ---")
        cfgs = hub_configs if entry["file"] == "block-0.L" else [
            ("K_quote_Gamma", quote_hub),
            ("K_hex_lo", hex_lo),
            ("K_hex_all", hex_all),
        ]
        for name, K_set in cfgs:
            text, st = run_early_stop_K(L, p, K_set, verify_exact=True)
            assert text == ref, f"EXACTNESS FAIL {entry['file']} {name}"
            r0 = st["r0"]; m16 = st["m16"]
            tot_raw = st["Q_con"] + st["m_raw"]
            tot_thin = st["Q_con"] + st["m_thin16"]
            print(f"  {name:20s} |K|={st['K_size']:2d} | Q_tree={st['Q_tree']:6d} Q_K={st['Q_comp_K']:6d} Q_con={st['Q_con']:6d}({st['Q_con']/r0:4.2f}r) "
                  f"| P_K={st['P_K']:6d} m_raw={st['m_raw']:6d}({st['m_raw']/m16:4.2f}m16) m_thin16={st['m_thin16']:6d}({st['m_thin16']/m16:4.2f}m16) "
                  f"| Q+m_thin={tot_thin:6d}({tot_thin/r0:4.2f}r) | slp_>16={st['slp_noninline_nodes']:5d} inl={st['inline_pct_rows']:4.1f}% [EXACT]", flush=True)


if __name__ == "__main__":
    main()
