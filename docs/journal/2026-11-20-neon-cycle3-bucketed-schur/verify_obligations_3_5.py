#!/usr/bin/env python3
"""Reproducible verifier for Cycle 3 Triage & Obligation-Register Claims:
1. Band-level second Open cut (OPEN-QUESTIONS.md §1): piece counts of g = P o LF_v
   and g_tilde = LF_v o P vs P's wide bands across all 6 authenticated blocks.
2. Obligation #3 (Phi-overlap coordinates, OPEN-QUESTIONS.md §2):
   - Exact raw Phi-copy counts, O(r log r) overlap graph edges, and cyclic fraction on all 6 blocks;
   - Exhaustive uniqueness test of raw Phi intervals on binary n <= 9 and ternary n <= 6.
3. Obligation #5 (Pre-column MTF/RLE2 representation):
   - Exact bzip2 MTF rank distribution and thin-row locality on all 6 blocks.
"""
import json, os
from collections import Counter, defaultdict, deque
from itertools import product

DATA_DIR = "/home/antigravity/.gemini/jetski/brain/v1/default/jetski_remote_agents_3p/7e9ef9b7-4d8c-42f5-9d61-045e6541ce3d/scratch/admitted-bwt"


def analyze_block(L: bytes, p: int, LIMIT=16):
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

    # --- 1. Open g = P o LF_v and g_tilde = LF_v o P pieces ---
    runs = []
    i = 0
    while i < n:
        c = L[i]
        j = i + 1
        while j < n and L[j] == c:
            j += 1
        w = j - i
        if w > LIMIT and i <= p < j:
            if i < p:
                runs.append((c, i, p - i, False))
            runs.append((c, p, 1, True))
            if p + 1 < j:
                runs.append((c, p + 1, j - p - 1, False))
        else:
            runs.append((c, i, w, w <= LIMIT))
        i = j

    row_to_ret = [-1] * n
    v = bytearray()
    m = 0
    for c, pos, w, keep in runs:
        if keep:
            for off in range(w):
                row_to_ret[pos + off] = m + off
            v.extend([c] * w)
            m += w
    ret_to_row = [0] * m
    for row in range(n):
        ri = row_to_ret[row]
        if ri >= 0:
            ret_to_row[ri] = row

    tcounts = [0] * 256
    for b in v:
        tcounts[b] += 1
    tC = [0] * 256
    t = 0
    for c in range(256):
        tC[c] = t
        t += tcounts[c]
    tcur = tC[:]
    LF_v = [0] * m
    for z in range(m):
        c = v[z]
        LF_v[z] = tcur[c]
        tcur[c] += 1
    Psi_v = [0] * m
    for z in range(m):
        Psi_v[LF_v[z]] = z
    F_sym = [0] * m
    for c in range(256):
        for y in range(tC[c], tC[c] + tcounts[c]):
            F_sym[y] = c

    P = [0] * m
    P_len = [0] * m
    for y in range(m):
        row = LF[ret_to_row[Psi_v[y]]]
        steps = 1
        while row_to_ret[row] < 0:
            row = LF[row]
            steps += 1
        P[y] = row_to_ret[row]
        P_len[y] = steps

    bands = []
    y = 0
    while y < m:
        c, d, ln = F_sym[y], P[y] - y, P_len[y]
        y2 = y + 1
        while y2 < m and F_sym[y2] == c and P[y2] - y2 == d and P_len[y2] == ln:
            y2 += 1
        bands.append(y2 - y)
        y = y2
    wide_rows = sum(w for w in bands if w > 16)
    g = [P[LF_v[z]] for z in range(m)]
    gt = [LF_v[P[y]] for y in range(m)]
    pieces_g = 1 + sum(1 for z in range(1, m) if g[z] != g[z - 1] + 1)
    pieces_gt = 1 + sum(1 for y in range(1, m) if gt[y] != gt[y - 1] + 1)

    # --- 2. Obligation #3: Raw Phi-copy graph from (SA[H], SA[D]) alone ---
    row_to_pos = [0] * n
    x = p
    for step in range(n):
        row_to_pos[x] = step
        x = LF[x]
    mruns = []
    i = 0
    while i < n:
        j = i + 1
        while j < n and L[j] == L[i]:
            j += 1
        mruns.append((i, j - 1, L[i]))
        i = j
    rm = len(mruns)
    cod = sorted(range(rm), key=lambda k: (mruns[k][2], mruns[k][0]))
    cpred = [0] * rm
    for idx in range(1, rm):
        cpred[cod[idx]] = cod[idx - 1]
    ph = [row_to_pos[mruns[k][0]] for k in range(rm)]
    pt = [row_to_pos[mruns[k][1]] for k in range(rm)]
    ord_r = sorted(range(rm), key=lambda k: ph[k])
    tau0 = ph[ord_r[0]]
    raw_copies = []
    for idx in range(rm):
        rid = ord_r[idx]
        tau = ph[rid] - tau0
        nxt = (ph[ord_r[idx + 1]] - tau0) if idx + 1 < rm else n
        ml = nxt - tau - 1
        if ml > 0:
            src = (pt[cpred[rid]] + 1 - tau0) % n
            dst = tau + 1
            if src + ml <= n:
                raw_copies.append((dst, src, ml))
            else:
                f = n - src
                raw_copies.append((dst, src, f))
                raw_copies.append((dst + f, 0, ml - f))
    nd = len(raw_copies)
    dst_s = sorted(enumerate(raw_copies), key=lambda kv: kv[1][0])
    src_s = sorted(enumerate(raw_copies), key=lambda kv: kv[1][1])
    adj = [[] for _ in range(nd)]
    indeg = [0] * nd
    edges = set()
    i_d = i_s = 0
    while i_d < nd and i_s < nd:
        j_id, (d0, _, ld) = dst_s[i_d]
        i_id, (_, s0, ls) = src_s[i_s]
        d1, s1 = d0 + ld, s0 + ls
        if d1 <= s0:
            i_d += 1
        elif s1 <= d0:
            i_s += 1
        else:
            if j_id != i_id and (j_id, i_id) not in edges:
                edges.add((j_id, i_id))
                adj[j_id].append(i_id)
                indeg[i_id] += 1
            if d1 <= s1:
                i_d += 1
            else:
                i_s += 1
    q = deque(k for k in range(nd) if indeg[k] == 0)
    topo_done = 0
    while q:
        u = q.popleft()
        topo_done += 1
        for w_id in adj[u]:
            indeg[w_id] -= 1
            if indeg[w_id] == 0:
                q.append(w_id)

    # --- 3. Obligation #5: Exact bzip2 MTF rank locality ---
    mtf_list = list(range(256))
    run_mtf = []
    i = 0
    while i < n:
        b = L[i]
        idx = mtf_list.index(b)
        if idx > 0:
            mtf_list.pop(idx)
            mtf_list.insert(0, b)
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        run_mtf.append((idx, j - i))
        i = j
    nz_heads = len(run_mtf) - 1  # excluding first run (idx may be > 0, count all runs)
    h1 = sum(1 for mv, _ in run_mtf if mv == 1)
    h3 = sum(1 for mv, _ in run_mtf if mv <= 3)
    h7 = sum(1 for mv, _ in run_mtf if mv <= 7)
    thin_m1 = sum(w for mv, w in run_mtf if w <= 16 and mv == 1)
    thin_m3 = sum(w for mv, w in run_mtf if w <= 16 and mv <= 3)

    return {
        "n": n, "r": rm, "m": m, "B": len(bands), "wide_pct": 100.0 * wide_rows / m,
        "pieces_g": pieces_g, "pieces_gt": pieces_gt,
        "raw_copies": nd, "raw_overlaps": len(edges), "topo_done": topo_done,
        "mtf1_pct": 100.0 * h1 / rm, "mtf3_pct": 100.0 * h3 / rm, "mtf7_pct": 100.0 * h7 / rm,
        "thin_m1_pct": 100.0 * thin_m1 / m, "thin_m3_pct": 100.0 * thin_m3 / m,
    }


def check_raw_phi_uniqueness(alpha, n_max):
    for n in range(2, n_max + 1):
        by_raw = defaultdict(set)
        by_canon = defaultdict(set)
        prim = 0
        for syms in product(range(alpha), repeat=n):
            L = bytes(syms)
            cnt = [0] * alpha
            for b in L:
                cnt[b] += 1
            C = [0] * alpha
            t = 0
            for c in range(alpha):
                C[c] = t
                t += cnt[c]
            cur = C[:]
            LF = [0] * n
            for i, b in enumerate(L):
                LF[i] = cur[b]
                cur[b] += 1
            pos = [-1] * n
            x = 0
            ok = True
            for step in range(n):
                if pos[x] != -1:
                    ok = False
                    break
                pos[x] = step
                x = LF[x]
            if not ok or x != 0:
                continue
            prim += 1
            mruns = []
            i = 0
            while i < n:
                j = i + 1
                while j < n and L[j] == L[i]:
                    j += 1
                mruns.append((i, j - 1, L[i]))
                i = j
            rm = len(mruns)
            cod = sorted(range(rm), key=lambda k: (mruns[k][2], mruns[k][0]))
            cpred = [0] * rm
            for idx in range(1, rm):
                cpred[cod[idx]] = cod[idx - 1]
            ph = [pos[mruns[k][0]] for k in range(rm)]
            pt = [pos[mruns[k][1]] for k in range(rm)]
            ord_r = sorted(range(rm), key=lambda k: ph[k])
            tau0 = ph[ord_r[0]]
            txt = [0] * n
            for row in range(n):
                txt[(pos[row] - tau0) % n] = L[row]
            txt = tuple(txt)
            seen = {}
            canon = tuple(seen.setdefault(ch, len(seen)) for ch in txt)
            rc = []
            for idx in range(rm):
                rid = ord_r[idx]
                tau = ph[rid] - tau0
                nxt = (ph[ord_r[idx + 1]] - tau0) if idx + 1 < rm else n
                ml = nxt - tau - 1
                if ml > 0:
                    src = (pt[cpred[rid]] + 1 - tau0) % n
                    dst = tau + 1
                    if src + ml <= n:
                        rc.append((dst, src, ml))
                    else:
                        f = n - src
                        rc.append((dst, src, f))
                        rc.append((dst + f, 0, ml - f))
            by_raw[tuple(rc)].add(txt)
            by_canon[tuple(rc)].add(canon)
        c_raw = sum(1 for v in by_raw.values() if len(v) > 1)
        c_can = sum(1 for v in by_canon.values() if len(v) > 1)
        print(f"  alpha={alpha} n={n}: prim={prim:3d}, distinct_raw_phi={len(by_raw):3d}, raw_collisions={c_raw:2d}, canon_collisions={c_can:2d}")


if __name__ == "__main__":
    manifest = json.load(open(os.path.join(DATA_DIR, "manifest.json")))
    print("=== 6-BLOCK MEASUREMENTS: BAND SHATTERING (OQ§1), RAW PHI CYCLES (OBL#3), MTF LOCALITY (OBL#5) ===")
    for b in manifest["blocks"]:
        L = open(os.path.join(DATA_DIR, b["file"]), "rb").read()
        d = analyze_block(L, b["primary_index"])
        cyc_pct = 100.0 * (1.0 - d["topo_done"] / d["raw_copies"])
        print(
            f"  {b['file']:11s}: m={d['m']:6d} B={d['B']:4d} (w>16:{d['wide_pct']:4.1f}%) | "
            f"pieces(g/gt)=({d['pieces_g']:6d}/{d['pieces_gt']:6d}) mean_w={d['m']/d['pieces_g']:.2f} | "
            f"Phi raw={d['raw_copies']:5d}({d['raw_copies']/d['r']:.2f}r) ov={d['raw_overlaps']:5d}({d['raw_overlaps']/d['r']:.2f}r) cyc={cyc_pct:4.1f}% | "
            f"MTF<=1/3/7=({d['mtf1_pct']:4.1f}%/{d['mtf3_pct']:4.1f}%/{d['mtf7_pct']:4.1f}%)"
        )
    print("\n=== OBLIGATION #3 SMALL-INSTANCE RAW PHI UNIQUENESS (BINARY n<=9, TERNARY n<=6) ===")
    check_raw_phi_uniqueness(2, 9)
    check_raw_phi_uniqueness(3, 6)
