"""Run-network census: Q1 separators of the series-reduced core, Q2 first-return pieces, the 41-chain.

Usage: BLOCKS=<dir with manifest.json and block-*.L>/ python census.py
Needs numpy, scipy, pymetis. Measurement only.
"""
import json, os, sys
import numpy as np
import pymetis
from scipy.sparse import csr_matrix
from scipy.sparse.csgraph import maximum_bipartite_matching, connected_components

D = os.environ.get('BLOCKS', 'admitted-bwt/')
M = json.load(open(D + 'manifest.json'))
CHAIN_ONLY = '--chain' in sys.argv


def build(L):
    n = len(L)
    a = np.frombuffer(L, dtype=np.uint8)
    starts = np.flatnonzero(np.r_[True, a[1:] != a[:-1]])
    widths = np.diff(np.r_[starts, n])
    sym = a[starts]
    r = len(starts)
    runid = np.repeat(np.arange(r), widths)
    # LF, dense: C[c] + rank of row among rows with symbol c
    cnt = np.bincount(a, minlength=256)
    C = np.r_[0, np.cumsum(cnt)[:-1]]
    order = np.argsort(a, kind='stable')          # F position -> row
    LF = np.empty(n, dtype=np.int64); LF[order] = np.arange(n)
    # arcs k -> v (distinct (run, run-of-image) pairs, in row order), with widths
    tgt = runid[LF]
    brk = np.r_[True, (runid[1:] != runid[:-1]) | (tgt[1:] != tgt[:-1])]
    ai = np.flatnonzero(brk)
    arc_src = runid[ai]; arc_dst = tgt[ai]; arc_w = np.diff(np.r_[ai, n])
    outd = np.bincount(arc_src, minlength=r); ind = np.bincount(arc_dst, minlength=r)
    return dict(n=n, a=a, r=r, starts=starts, widths=widths, sym=sym, runid=runid, LF=LF,
                arc_src=arc_src, arc_dst=arc_dst, arc_w=arc_w, outd=outd, ind=ind)


def reduce_series(G):
    r = G['r']; series = (G['ind'] == 1) & (G['outd'] == 1)
    succ = np.full(r, -1); s = series[G['arc_src']]
    succ[G['arc_src'][s]] = G['arc_dst'][s]
    core_arcs = []   # (src core, dst core, width, chain length, chain runs)
    for k, v, w in zip(G['arc_src'], G['arc_dst'], G['arc_w']):
        if series[k]:
            continue
        chain = []; x = v
        while series[x]:
            chain.append(x); x = succ[x]
        core_arcs.append((k, x, w, chain))
    return series, core_arcs


def bisect_hierarchy(nv, edges_u, edges_v, edges_w, cap=64):
    """Recursive METIS bisection. Returns per-level lists of (edge cut count, cut weight, vertex sep)."""
    levels = {}
    ports = 0; port_rows = 0
    cut_mask = np.zeros(len(edges_u), dtype=bool)
    stack = [(np.arange(nv), 0)]
    while stack:
        verts, lev = stack.pop()
        if len(verts) <= cap:
            continue
        loc = np.full(nv, -1); loc[verts] = np.arange(len(verts))
        inside = (loc[edges_u] >= 0) & (loc[edges_v] >= 0) & ~cut_mask
        eu = loc[edges_u[inside]]; ev = loc[edges_v[inside]]; ew = edges_w[inside]
        m = len(verts)
        A = csr_matrix((np.r_[ew, ew].astype(np.int64), (np.r_[eu, ev], np.r_[ev, eu])), shape=(m, m))
        A.sum_duplicates()
        if A.nnz == 0:
            part = np.r_[np.zeros(m // 2, int), np.ones(m - m // 2, int)]
        else:
            # multiplicity as edge weight; pieces are kept balanced by METIS
            _, part = pymetis.part_graph(2, xadj=A.indptr, adjncy=A.indices, eweights=A.data)
            part = np.asarray(part)
        idx = np.flatnonzero(inside)
        crossing = part[eu] != part[ev]
        cut_idx = idx[crossing]
        cut_mask[cut_idx] = True
        # vertex separator of this cut: minimum vertex cover of the bipartite cut graph (Konig)
        cu = eu[crossing]; cv = ev[crossing]
        su = np.where(part[cu] == 0, cu, cv); sv = np.where(part[cu] == 0, cv, cu)
        if len(su):
            ul, ui = np.unique(su, return_inverse=True); vl, vi = np.unique(sv, return_inverse=True)
            B = csr_matrix((np.ones(len(ui)), (ui, vi)), shape=(len(ul), len(vl)))
            vsep = int((maximum_bipartite_matching(B, perm_type='column') >= 0).sum())
        else:
            vsep = 0
        L = levels.setdefault(lev, [0, 0, 0, 0, 0])
        L[0] += 1; L[1] += len(cut_idx); L[2] += int(edges_w_rows[cut_idx].sum()); L[3] += vsep
        L[4] = max(L[4], m)
        for p in (0, 1):
            stack.append((verts[part == p], lev + 1))
    return levels, cut_mask


for b in M['blocks']:
    L = open(D + b['file'], 'rb').read()
    G = build(L); n, r = G['n'], G['r']
    series, core_arcs = reduce_series(G)
    core = np.flatnonzero(~series); nc = len(core)
    cid = np.full(r, -1); cid[core] = np.arange(nc)
    m_open = int(G['widths'][G['widths'] <= 16].sum())
    tag = b['file'][:-2]

    if CHAIN_ONLY or b['file'] == 'block-0.L':
        lens = [len(c[3]) for c in core_arcs]; best = int(np.argmax(lens))
        k, x, w, chain = core_arcs[best]
        path = [k] + chain + [x]
        print(f"== {tag}: longest chain {len(chain)} series runs, width {w}; "
              f"chains of length 41: {sum(1 for l in lens if l == 41)}, 40: {sum(1 for l in lens if l == 40)}")
        # LF moves backwards in the text, so the symbols along the path read the text right to left
        s = bytes(int(G['sym'][p]) for p in path)
        print('   symbols along path (LF order):', s)
        print('   reversed (text order):        ', s[::-1])
        print('   widths:', [int(G['widths'][p]) for p in path])
        # the text context: walk LF from the first row of run k
        row = int(G['starts'][k]); ctx = []
        for _ in range(len(path) + 40):
            ctx.append(int(G['a'][row])); row = int(G['LF'][row])
        print('   text before (and incl.) the chain, text order:', bytes(ctx[::-1]))
        if CHAIN_ONLY:
            continue

    # ---- Q1: separators of the core (undirected, self-loops dropped, multi-arcs weighted)
    cu = np.array([cid[c[0]] for c in core_arcs]); cv = np.array([cid[c[1]] for c in core_arcs])
    cw = np.array([c[2] for c in core_arcs])
    A_core = len(core_arcs)
    nl = cu != cv
    eu, ev, ew_rows = cu[nl], cv[nl], cw[nl]
    edges_w_rows = ew_rows
    ncomp = connected_components(csr_matrix((np.ones(len(eu)), (eu, ev)), shape=(nc, nc)), directed=False)[0]
    levels, cut = bisect_hierarchy(nc, eu, ev, np.ones(len(eu), dtype=np.int64))
    ports = int(cut.sum()); port_rows = int(ew_rows[cut].sum())
    vsep_total = sum(v[3] for v in levels.values())
    print(f"## {tag} n={n} r={r} m={m_open} ({m_open/n:.3f}n, {m_open/r:.2f}r) core={nc} ({nc/r:.3f}r) "
          f"core arcs={A_core} ({A_core/r:.3f}r, {A_core/n:.4f}n) self-loops={int((~nl).sum())} components={ncomp}")
    for lev in sorted(levels):
        cnt, ecut, erows, vsep, biggest = levels[lev]
        print(f"   Q1 level {lev}: pieces split {cnt:4d} (largest {biggest:5d} v) edge cut {ecut:5d} "
              f"(rows {erows:7d}) vertex sep {vsep:5d}")
    print(f"   Q1 total cut arcs (ports) {ports} = {ports/A_core:.3f} of core arcs = {ports/n:.4f}n = "
          f"{ports/r:.3f}r = {ports/m_open:.3f}m | 2x ports (both ends) {2*ports/n:.4f}n | "
          f"rows on cut arcs {port_rows/n:.3f}n | vertex-separator total {vsep_total} ({vsep_total/n:.4f}n, {vsep_total/nc:.3f} of core)")

    # ---- Q2: first-return map to rows of branching runs, by a dense LF walk
    LF = G['LF']; runid = G['runid']
    inB = ~series[runid]
    # walk the LF cycle(s) once
    seen = np.zeros(n, dtype=bool); T = np.full(n, -1); H = np.zeros(n, dtype=np.uint64)
    ncyc = 0
    LFl = LF.tolist(); inBl = inB.tolist(); rid = runid.tolist()
    MOD = (1 << 61) - 1
    for start in range(n):
        if seen[start]:
            continue
        ncyc += 1
        # cycle from start
        cyc = [start]; x = LFl[start]
        while x != start:
            cyc.append(x); x = LFl[x]
        seen[cyc] = True
        bpos = [i for i, y in enumerate(cyc) if inBl[y]]
        if not bpos:
            continue
        cl = len(cyc)
        for j, i in enumerate(bpos):
            i2 = bpos[(j + 1) % len(bpos)]
            h = 0; t = i + 1
            while True:
                y = cyc[t % cl]
                h = (h * 1000003 + rid[y] + 1) % MOD
                if inBl[y]:
                    break
                t += 1
            T[cyc[i]] = cyc[i2 % cl]; H[cyc[i]] = h
    Brows = np.flatnonzero(inB)
    nb = len(Brows)
    adj = Brows[1:] == Brows[:-1] + 1
    same_run = adj & (runid[Brows[1:]] == runid[Brows[:-1]])
    same_path = same_run & (H[Brows[1:]] == H[Brows[:-1]])
    transl = adj & (T[Brows[1:]] == T[Brows[:-1]] + 1)
    pieces_path = nb - int(same_path.sum())
    pieces_transl = nb - int(transl.sum())
    print(f"   Q2 LF cycles {ncyc}; rows in branching runs {nb} ({nb/n:.3f}n)")
    print(f"   Q2 pieces (same arc sequence) {pieces_path} = {pieces_path/n:.4f}n = {pieces_path/r:.3f}r = "
          f"{pieces_path/m_open:.3f}m  [series-reduced core arcs {A_core}]")
    print(f"   Q2 pieces (maximal translation intervals of the first-return map) {pieces_transl} = "
          f"{pieces_transl/n:.4f}n = {pieces_transl/r:.3f}r = {pieces_transl/m_open:.3f}m")
    sys.stdout.flush()
