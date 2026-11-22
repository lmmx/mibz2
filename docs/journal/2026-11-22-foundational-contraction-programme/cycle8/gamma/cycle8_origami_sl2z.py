#!/usr/bin/env python3
"""
Cycle 8 — Doorway 1: The Origami Lift & SL(2, Z) Shear / Cylinder Decompositions
across ALL 6 authenticated Wikidata blocks (/tmp/mibz2-blocks/block-*.L).
"""
import hashlib
import json
import os

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


def build_lf(data):
    n = len(data)
    counts = [0] * 256
    for b in data:
        counts[b] += 1
    C = [0] * 256
    acc = 0
    for c in range(256):
        C[c] = acc
        acc += counts[c]
    occ = C[:]
    LF = [0] * n
    for i, b in enumerate(data):
        LF[i] = occ[b]
        occ[b] += 1
    return LF, C, counts


def analyze_permutation(perm, limit=16):
    n = len(perm)
    r_strips = 1
    cur_w = 1
    m_16 = 0
    for i in range(n - 1):
        if perm[i + 1] == perm[i] + 1:
            cur_w += 1
        else:
            if cur_w <= limit:
                m_16 += cur_w
            r_strips += 1
            cur_w = 1
    if cur_w <= limit:
        m_16 += cur_w

    vis = bytearray(n)
    num_cycles = max_cyc = fixed_pts = short16_rows = 0
    for i in range(n):
        if vis[i]:
            continue
        cur = i
        clen = 0
        while not vis[cur]:
            vis[cur] = 1
            cur = perm[cur]
            clen += 1
        num_cycles += 1
        if clen > max_cyc:
            max_cyc = clen
        if clen == 1:
            fixed_pts += 1
        if clen <= 16:
            short16_rows += clen
    return {'r': r_strips, 'm_16': m_16, 'cycles': num_cycles, 'max_cyc': max_cyc, 'fixed': fixed_pts, 'short16': short16_rows}


def compute_commutator_stats(LF):
    n = len(LF)
    FL = [0] * n
    for i, v in enumerate(LF):
        FL[v] = i
    k_map = {}
    for x in range(n):
        y = LF[(FL[(x - 1) % n] + 1) % n]
        if y != x:
            k_map[x] = y
    vis = set()
    cyc_lens = []
    for x in k_map:
        if x in vis:
            continue
        cur = x
        cl = 0
        while cur not in vis:
            vis.add(cur)
            cur = k_map[cur]
            cl += 1
        cyc_lens.append(cl)
    R = len(k_map)
    c = len(cyc_lens)
    return R, c, 1 + (R - c) // 2, max(cyc_lens) if cyc_lens else 0


def compute_sheared_out1_and_r1(perm, limit=16):
    n = len(perm)
    strips = []
    lo = 0
    for i in range(n - 1):
        if perm[i + 1] != perm[i] + 1:
            strips.append((lo, i + 1, perm[lo] - lo))
            lo = i + 1
    strips.append((lo, n, perm[lo] - lo))
    q0 = len(strips)
    j_order = sorted(range(q0), key=lambda u: strips[u][0] + strips[u][2])
    out_deg = [0] * q0
    r1_rows = out1_rows = 0
    ii = jj = 0
    while ii < q0 and jj < q0:
        v = ii
        u = j_order[jj]
        v_lo, v_hi = strips[v][0], strips[v][1]
        u_tlo, u_thi = strips[u][0] + strips[u][2], strips[u][1] + strips[u][2]
        if u_thi <= v_lo:
            jj += 1
        elif v_hi <= u_tlo:
            ii += 1
        else:
            out_deg[u] += 1
            if u_tlo == v_lo and u_thi == v_hi and u != v and (u_thi - u_tlo) <= limit:
                r1_rows += (u_thi - u_tlo)
            if u_thi < v_hi:
                jj += 1
            elif v_hi < u_thi:
                ii += 1
            else:
                ii += 1
                jj += 1
    for u, (slo, shi, _) in enumerate(strips):
        if shi - slo <= limit and out_deg[u] == 1:
            out1_rows += (shi - slo)
    m16 = sum(hi - lo for lo, hi, _ in strips if hi - lo <= limit)
    return q0, m16, out1_rows, r1_rows


def main():
    blocks = load_blocks()
    blks_lf = []
    for blk, data in blocks:
        LF, _, _ = build_lf(data)
        R, c, genus, max_cl = compute_commutator_stats(LF)
        blks_lf.append((blk, data, LF))
        print(f"  {blk['file']:11s}: moved(k)={R:6d} cone_pts={c:2d} genus={genus:5d} max_cone={max_cl:6d}")
    for k in (0, 1, -1, 2, -2):
        s_m16 = s_o1 = s_r1 = 0
        for blk, data, LF in blks_lf:
            n = len(data)
            perm = LF if k == 0 else [(v + k) % n for v in LF]
            _, m16, o1, r1 = compute_sheared_out1_and_r1(perm, limit=16)
            s_m16 += m16; s_o1 += o1; s_r1 += r1
        print(f"  Shear (1, {k:+d}): out_deg==1={100*s_o1/s_m16:4.1f}% exact-R1={100*s_r1/s_m16:4.1f}%")


if __name__ == "__main__":
    main()
