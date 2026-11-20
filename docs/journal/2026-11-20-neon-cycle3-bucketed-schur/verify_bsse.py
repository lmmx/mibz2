#!/usr/bin/env python3
"""Gate G1 Exhaustive & 6-Block Verifier for Bucketed Sequential Schur Elimination (BSSE).

Verifies:
1. Exhaustive small cases: all binary columns n <= 8 (510 columns) and all ternary
   columns n <= 5 (363 columns), at every primary mark p in 0..n-1, across multiple
   LIMIT in {1, 2, 4, 16} and bucket sizes bsize in {1, 2, 3, 4, 8} — including
   multi-cycle, periodic, single-retained-node, and early-closure-in-B_k cases.
2. Six authenticated admitted Wikidata BWT blocks (5,399,692 bytes total) byte-exact
   against a dense LF reference walk, reporting exact bucket/routing statistics.
"""
import json, os, sys, time
from itertools import product

DATA_DIR = "/home/antigravity/.gemini/jetski/brain/v1/default/jetski_remote_agents_3p/7e9ef9b7-4d8c-42f5-9d61-045e6541ce3d/scratch/admitted-bwt"


def dense_inverse_bwt(L: bytes, p: int) -> bytes:
    """Reference exact inverse BWT via dense LF walk (matches Open's periodic repeat semantics)."""
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    t = 0
    for c in range(256):
        C[c] = t
        t += counts[c]
    cur = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = cur[b]
        cur[b] += 1
    # Walk marked cycle backward from p: L[p] is last byte of marked cycle, etc.
    cyc_rev = bytearray()
    x = p
    while True:
        cyc_rev.append(L[x])
        x = LF[x]
        if x == p or len(cyc_rev) == n:
            break
    cyc = bytes(reversed(cyc_rev))
    if len(cyc) == n:
        return cyc
    # Repeat marked cycle to length n (exact Open semantics for non-primitive/multi-cycle columns)
    reps = (n + len(cyc) - 1) // len(cyc)
    return (cyc * reps)[:n]


def each_run_from_L(L: bytes, p: int, limit: int):
    """Exact Open maximal-run classification with primary split inside thick runs."""
    n = len(L)
    runs = []
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        if w > limit and i <= p < j:
            if i < p:
                runs.append((c, i, p - i, False))
            runs.append((c, p, 1, True))
            if p + 1 < j:
                runs.append((c, p + 1, j - p - 1, False))
        else:
            runs.append((c, i, w, w <= limit))
        i = j
    return runs


def build_bands_exact(L: bytes, p: int, limit: int):
    """Build exact Open retained set S, thin runs, P bands, and band spelling dictionary."""
    n = len(L)
    runs = each_run_from_L(L, p, limit)
    counts = [0] * 256
    for c, pos, w, keep in runs:
        counts[c] += w
    C = [0] * 256
    tot = 0
    for c in range(256):
        C[c] = tot
        tot += counts[c]

    # Exact thick/thin classification & thin runs
    row_to_ret = [-1] * n
    thin_runs = []
    m = 0
    mark = -1
    tcounts = [0] * 256
    for c, pos, w, keep in runs:
        if keep:
            if pos <= p < pos + w:
                mark = m + (p - pos)
            thin_runs.append((c, m, w))
            for off in range(w):
                row_to_ret[pos + off] = m + off
            tcounts[c] += w
            m += w

    # Compute exact endpoint elimination / bands via thick strips (same semantics as Open's bands)
    # Build tile map on [0, n) using thick strips + thin gaps (identical to Open's tiles/trace)
    cur = C[:]
    # Maps in F-order: for each strip (c, pos, w), LF maps [pos, pos+w) -> [cur[c], cur[c]+w)
    # Wait: Open traces backward in LF from F-sorted retained rows y!
    # For verification, construct exact LF on [0, n) and build bands of P(y)
    LF = [0] * n
    for c, pos, w, keep in runs:
        dst = cur[c]
        for off in range(w):
            LF[pos + off] = dst + off
        cur[c] = dst + w

    tC = [0] * 256
    t = 0
    for c in range(256):
        tC[c] = t
        t += tcounts[c]

    # Build Psi_v and ret_to_row
    ret_to_row = [0] * m
    for row in range(n):
        ri = row_to_ret[row]
        if ri >= 0:
            ret_to_row[ri] = row

    tcur = tC[:]
    Psi_v = [0] * m
    for c, mpos, w in thin_runs:
        dst = tcur[c]
        for off in range(w):
            Psi_v[dst + off] = mpos + off
        tcur[c] = dst + w

    # Trace bands in F-order of S
    bands = []  # (end_y, delta, code, word_len)
    words = []  # index = code - 256
    word_to_code = {}
    y = 0
    while y < m:
        # Trace y
        z0 = Psi_v[y]
        row = ret_to_row[z0]
        c_lit = L[row]
        # Walk LF until hitting retained
        sp = bytearray([c_lit])
        x = LF[row]
        while row_to_ret[x] < 0:
            sp.append(L[x])
            x = LF[x]
        exit_z = row_to_ret[x]
        delta = exit_z - y
        # Forward text word is reversed(sp)
        if len(sp) == 1:
            code = c_lit
            wlen = 1
        else:
            w_fwd = bytes(reversed(sp))
            if w_fwd not in word_to_code:
                word_to_code[w_fwd] = 256 + len(words)
                words.append(w_fwd)
            code = word_to_code[w_fwd]
            wlen = len(w_fwd)

        # Extend band while same symbol bucket, same delta, same code
        # (Find symbol bucket end for y)
        bkt_end = tC[c_lit] + tcounts[c_lit]
        y2 = y + 1
        while y2 < bkt_end:
            z_next = Psi_v[y2]
            r_next = ret_to_row[z_next]
            x2 = LF[r_next]
            # Fast check: check if itinerary matches
            if wlen == 1:
                if row_to_ret[x2] == y2 + delta:
                    y2 += 1
                    continue
                else:
                    break
            else:
                sp2 = bytearray([L[r_next]])
                while row_to_ret[x2] < 0:
                    sp2.append(L[x2])
                    x2 = LF[x2]
                if row_to_ret[x2] == y2 + delta and bytes(reversed(sp2)) == words[code - 256]:
                    y2 += 1
                    continue
                else:
                    break
        bands.append((y2, delta, code, wlen))
        y = y2

    return n, m, mark, thin_runs, tC, tcounts, bands, words


def bsse_reconstruct(n, m, mark, thin_runs, tC, tcounts, bands, words, bsize, n_slabs=8):
    """Exact Bucketed Sequential Schur Elimination (BSSE) with Chain-Head Lemma and Slabbed Emission."""
    s = (m + bsize - 1) // bsize
    assert s >= 1

    # Find first band index for each symbol bucket c (for queue-free backward scan!)
    band_cur = [0] * 256
    bi = 0
    for c in range(256):
        if tcounts[c] > 0:
            while bi < len(bands) and bands[bi][0] <= tC[c]:
                bi += 1
            band_cur[c] = bi
    sym_cur = tC[:]

    # Phase 1: Queue-free backward scan of thin_runs -> initial bucket queues + orig_code
    orig_code = [0] * m
    fwd_q = [[] for _ in range(s)]  # merges self_q + out_q: (u_loc, w_global, ln)
    in_q = [[] for _ in range(s)]   # (u_global, w_loc, ln)

    for c, mpos, w in thin_runs:
        y = sym_cur[c]
        b_idx = band_cur[c]
        rem = w
        z = mpos
        while rem > 0:
            b_end, b_delta, b_code, b_len = bands[b_idx]
            take = min(rem, b_end - y)
            for off in range(take):
                u = z + off
                dst = (y + off) + b_delta
                orig_code[u] = b_code
                ju = u // bsize
                jd = dst // bsize
                if ju <= jd:
                    fwd_q[ju].append((u - ju * bsize, dst, b_len))
                else:
                    in_q[jd].append((u, dst - jd * bsize, b_len))
            z += take
            y += take
            rem -= take
            if y == b_end:
                b_idx += 1
        sym_cur[c] = y
        band_cur[c] = b_idx

    # Phase 2: Forward Schur Elimination (k = 0 .. s-1) with Exact Chain-Head Decomposition
    req_u = [[[] for _ in range(s)] for _ in range(s)]  # req_u[k][ju] = list of u_loc
    self_chain = [[] for _ in range(s)]                 # topological (v_loc, next_v_loc, step_ln)
    mark_anchor = mark
    mark_shift = 0
    close_k = -1
    L_cyc = 0
    close_pos = None

    total_routed = 0
    total_self_hops = 0

    for k in range(s):
        base = k * bsize
        sz = min(bsize, m - base)
        nxt_dst = [0] * sz
        nxt_ln = [0] * sz
        for ul, wg, ln in fwd_q[k]:
            nxt_dst[ul] = wg
            nxt_ln[ul] = ln

        visited = [False] * sz
        # Process every entry from in_q[k] (none if k == s-1!)
        for u_glob, v0, ln1 in in_q[k]:
            ju = u_glob // bsize
            req_u[k][ju].append(u_glob - ju * bsize)
            cur = v0
            acc_ln = 0
            hit_mark_at = -1
            while True:
                visited[cur] = True
                if base + cur == mark_anchor:
                    hit_mark_at = acc_ln
                w_glob = nxt_dst[cur]
                step_ln = nxt_ln[cur]
                acc_ln += step_ln
                jw = w_glob // bsize
                if jw != k:
                    # Exited bucket k to jw > k!
                    if hit_mark_at >= 0:
                        mark_anchor = w_glob
                        mark_shift += (acc_ln - hit_mark_at)
                    total_routed += 1
                    tot_ln = ln1 + acc_ln
                    if ju <= jw:
                        fwd_q[ju].append((u_glob - ju * bsize, w_glob, tot_ln))
                    else:
                        in_q[jw].append((u_glob, w_glob - jw * bsize, tot_ln))
                    break
                else:
                    nxt_loc = w_glob - base
                    self_chain[k].append((cur, nxt_loc, step_ln))
                    total_self_hops += 1
                    cur = nxt_loc

        # Check if mark_anchor lies in bucket k and was NOT visited (marked cycle closed inside B_k!)
        if mark_anchor // bsize == k:
            mloc = mark_anchor - base
            if not visited[mloc]:
                # Walk the closed marked cycle inside B_k!
                close_k = k
                close_pos = [-1] * sz
                cur = mloc
                rel = 0
                cycle_nodes = []
                while True:
                    cycle_nodes.append((cur, rel))
                    w_glob = nxt_dst[cur]
                    step_ln = nxt_ln[cur]
                    rel += step_ln
                    assert w_glob // bsize == k, "Closed cycle escaped bucket!"
                    cur = w_glob - base
                    if cur == mloc:
                        break
                L_cyc = rel
                for node, rdist in cycle_nodes:
                    close_pos[node] = (rdist + mark_shift) % L_cyc
                break

    assert close_k >= 0 and L_cyc > 0, "Marked cycle never closed!"

    # Phase 3: Reverse Position Assignment (j = close_k down to 0) + Output Slab Bucketing
    resp_p = [[[] for _ in range(s)] for _ in range(s)]
    slab_sz = (L_cyc + n_slabs - 1) // n_slabs
    out_q_slab = [[] for _ in range(n_slabs)]

    for j in range(close_k, -1, -1):
        base = j * bsize
        sz = min(bsize, m - base)
        if j == close_k:
            pos = close_pos
        else:
            pos = [-1] * sz
            # Replay in_q[j] consuming resp_p[j][ju] in exact FIFO order
            rptr = [0] * s
            for u_glob, v0, ln1 in in_q[j]:
                ju = u_glob // bsize
                if ju > close_k:
                    continue
                p_u = resp_p[j][ju][rptr[ju]]
                rptr[ju] += 1
                if p_u >= 0:
                    pos[v0] = (p_u + ln1) % L_cyc
            # Propagate along topological self_chain[j] in one linear pass!
            for u_loc, w_loc, step_ln in self_chain[j]:
                p_u = pos[u_loc]
                if p_u >= 0:
                    pos[w_loc] = (p_u + step_ln) % L_cyc

        # Answer requests from earlier buckets k < j
        for k in range(j):
            rq = req_u[k][j]
            if rq:
                resp_p[k][j] = [pos[ul] for ul in rq]

        # Emit (fwd_start, code) into L2-resident output slabs
        for u_loc in range(sz):
            bp = pos[u_loc]
            if bp >= 0:
                code = orig_code[base + u_loc]
                wlen = 1 if code < 256 else len(words[code - 256])
                fwd_start = L_cyc - bp - wlen
                assert 0 <= fwd_start <= L_cyc - wlen
                sl = fwd_start // slab_sz
                out_q_slab[sl].append((fwd_start - sl * slab_sz, code))

    # Phase 4: L2-Slabbed Output Emission
    out = bytearray(L_cyc)
    for sl in range(n_slabs):
        sl_base = sl * slab_sz
        for off, code in out_q_slab[sl]:
            idx = sl_base + off
            if code < 256:
                out[idx] = code
            else:
                w = words[code - 256]
                out[idx:idx + len(w)] = w

    if L_cyc < n:
        reps = (n + L_cyc - 1) // L_cyc
        res = (bytes(out) * reps)[:n]
    else:
        res = bytes(out)
    return res, {"s": s, "close_k": close_k, "routed": total_routed, "self_hops": total_self_hops, "L_cyc": L_cyc}


def run_small_exhaustive():
    cases = 0
    early_closures = 0
    short_periods = 0
    t0 = time.time()
    # Binary n <= 8
    for n in range(1, 9):
        for bits in product((0, 1), repeat=n):
            L = bytes(bits)
            for p in range(n):
                ref = dense_inverse_bwt(L, p)
                for lim in (1, 2, 4, 16):
                    st = build_bands_exact(L, p, lim)
                    for bsize in (1, 2, 3, 4, 8):
                        got, info = bsse_reconstruct(*st, bsize=bsize, n_slabs=2)
                        assert got == ref, f"FAIL bin n={n} L={list(L)} p={p} lim={lim} bsize={bsize}: got={list(got)} ref={list(ref)}"
                        cases += 1
                        if info["close_k"] < info["s"] - 1:
                            early_closures += 1
                        if info["L_cyc"] < n:
                            short_periods += 1
    # Ternary n <= 5
    for n in range(1, 6):
        for syms in product((0, 1, 2), repeat=n):
            L = bytes(syms)
            for p in range(n):
                ref = dense_inverse_bwt(L, p)
                for lim in (1, 2, 4, 16):
                    st = build_bands_exact(L, p, lim)
                    for bsize in (1, 2, 3, 4):
                        got, info = bsse_reconstruct(*st, bsize=bsize, n_slabs=2)
                        assert got == ref, f"FAIL tern n={n} L={list(L)} p={p} lim={lim} bsize={bsize}"
                        cases += 1
                        if info["close_k"] < info["s"] - 1:
                            early_closures += 1
                        if info["L_cyc"] < n:
                            short_periods += 1
    dt = time.time() - t0
    print(f"SMALL EXHAUSTIVE PASS: {cases:,} / {cases:,} cases ({early_closures:,} early-bucket closures, {short_periods:,} short-period/multi-cycle) in {dt:.2f}s")


def run_six_blocks():
    manifest = json.load(open(os.path.join(DATA_DIR, "manifest.json")))
    tot_bytes = 0
    print("\n=== SIX AUTHENTICATED WIKIDATA BLOCKS (bsize=16384, LIMIT=16) ===")
    for b in manifest["blocks"]:
        L = open(os.path.join(DATA_DIR, b["file"]), "rb").read()
        p = b["primary_index"]
        ref = dense_inverse_bwt(L, p)
        st = build_bands_exact(L, p, 16)
        got, info = bsse_reconstruct(*st, bsize=16384, n_slabs=8)
        assert got == ref, f"MISMATCH on {b['file']}!"
        m = st[1]
        tot_bytes += len(ref)
        print(f"  {b['file']:12s}: n={len(ref)}, m={m:6d}, s={info['s']:2d}, routed={info['routed']:6d} ({info['routed']/m:.3f}m), self_hops={info['self_hops']:6d} ({info['self_hops']/m:.3f}m), BYTE-EXACT PASS")
    print(f"TOTAL VERIFIED BYTES: {tot_bytes:,} / {tot_bytes:,} (100% BYTE-EXACT)")


if __name__ == "__main__":
    run_small_exhaustive()
    run_six_blocks()
