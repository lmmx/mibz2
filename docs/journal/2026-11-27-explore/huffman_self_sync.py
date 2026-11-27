#!/usr/bin/env python3
"""Self-synchronisation of bzip2's Huffman stage, measured on a real corpus.

Usage: python3 huffman_self_sync.py FILE.bz2 [--starts N] [--seed S] [--json OUT]

For every block it parses the header (tables, selectors) and decodes the true symbol
boundaries exactly, then asks, for speculative decoders started at a wrong place:

  A. oracle-table sync: start at a random bit, decode with the table the true decoder uses at
     each bit (the best a speculative decoder could do); how many codewords / bits until its
     codeword boundary coincides with a true one?  Once it coincides it stays coincident.
  B. guessed-table sync: the same with one fixed table guessed for the whole run; first true
     boundary reached, and whether the true table there is the guessed one.
  C. selector structure: tables a block, how often consecutive 50-symbol groups share a table.
  D. phase error: start exactly on a true boundary, but believe the symbol index is off by d
     (so the selector schedule is shifted); does the decoder ever re-lock onto the true path
     with the true index?  This is the obstruction particular to bzip2.

Exactness check: the decoded symbol count and EOB position of every block are checked against
the next 48-bit magic; the script fails if any block does not end at a magic.
"""

import argparse
import json
import sys

import numpy as np

BLOCK_MAGIC = 0x314159265359
END_MAGIC = 0x177245385090
W = 20  # bzip2's maximum code length


class Bits:
    """An MSB-first bit reader over a whole file."""

    def __init__(self, data: bytes):
        self.bits = np.unpackbits(np.frombuffer(data, dtype=np.uint8))
        self.n = len(self.bits)
        self.p = 0

    def get(self, k: int) -> int:
        """Read k bits as an unsigned integer."""
        v = 0
        for b in self.bits[self.p : self.p + k]:
            v = (v << 1) | int(b)
        self.p += k
        return v


def peek_array(bits: np.ndarray, start: int, end: int) -> np.ndarray:
    """peek[i] = the W bits at start+i, as an integer (zero-padded past the input)."""
    seg = bits[start : end + W].astype(np.uint32)
    if len(seg) < end - start + W:
        seg = np.concatenate([seg, np.zeros(end - start + W - len(seg), np.uint32)])
    n = end - start
    out = np.zeros(n, np.uint32)
    for i in range(W):
        out |= seg[i : i + n] << (W - 1 - i)
    return out


def canonical(lengths):
    """(len_table, sym_table) of size 2^W: code length 0 means no codeword (invalid)."""
    lt = np.zeros(1 << W, np.int32)
    st = np.zeros(1 << W, np.int32)
    code = 0
    for ln in range(1, W + 1):
        for s, l in enumerate(lengths):
            if l == ln:
                lo = code << (W - ln)
                hi = (code + 1) << (W - ln)
                if hi > (1 << W):
                    raise ValueError("over-full code")
                lt[lo:hi] = ln
                st[lo:hi] = s
                code += 1
        code <<= 1
    return lt, st


def parse_block(br: Bits):
    """Header of the block whose magic was just read; returns (eob, tables, selectors)."""
    br.get(32)  # crc
    if br.get(1):
        raise ValueError("randomised block")
    br.get(24)  # orig_ptr
    used = br.get(16)
    n_in_use = 0
    for i in range(16):
        if used & (0x8000 >> i):
            n_in_use += bin(br.get(16)).count("1")
    alpha = n_in_use + 2
    n_groups = br.get(3)
    n_sel = br.get(15)
    mtf = list(range(n_groups))
    sels = []
    for _ in range(n_sel):
        j = 0
        while br.get(1):
            j += 1
        v = mtf.pop(j)
        mtf.insert(0, v)
        sels.append(v)
    tables = []
    for _ in range(n_groups):
        cur = br.get(5)
        lens = []
        for _ in range(alpha):
            while br.get(1):
                cur += -1 if br.get(1) else 1
            lens.append(cur)
        tables.append(canonical(lens))
    return alpha - 1, tables, np.array(sels, np.int64)


def analyse_block(bits, start, eob, tables, sels, rng, n_starts):
    """Decode one block's true path and run measurements A-D on it."""
    nT = len(tables)
    # decode the true path over a generous range: a block's data never exceeds 20 bits/sym
    max_syms = 50 * len(sels)
    end = min(start + W * max_syms + 64, len(bits) - 0)
    peek = peek_array(bits, start, end)
    L = np.stack([t[0][peek] for t in tables])  # (nT, nbits) code length at each bit
    S = np.stack([t[1][peek] for t in tables])
    Ll = [row.tolist() for row in L]
    Sl = [row.tolist() for row in S]
    sel = sels.tolist()
    p, i = 0, 0
    bounds = []
    while True:
        t = sel[i // 50]
        l = Ll[t][p]
        if l == 0:
            raise ValueError("invalid code on the true path")
        bounds.append(p)
        s = Sl[t][p]
        p += l
        i += 1
        if s == eob:
            break
    data_end = p
    n_syms = i
    nb = data_end
    isb = np.zeros(nb + 1, bool)
    bidx = np.full(nb + 1, -1, np.int64)
    barr = np.array(bounds, np.int64)
    isb[barr] = True
    bidx[barr] = np.arange(n_syms)
    # covering symbol of each bit position, its group and the oracle table there
    cover = np.searchsorted(barr, np.arange(nb), side="right") - 1
    otab = sels[cover // 50]
    L = L[:, : nb + W]
    # ---- A: oracle-table sync from every sampled non-boundary bit
    cand = np.flatnonzero(~isb[:nb])
    starts = rng.choice(cand, size=min(n_starts, len(cand)), replace=False)
    nxtA = np.arange(nb) + L[otab, np.arange(nb)]
    lenA = L[otab, np.arange(nb)]
    pos = starts.copy()
    steps = np.zeros(len(pos), np.int64)
    done = np.zeros(len(pos), bool)
    dead = np.zeros(len(pos), bool)
    for k in range(1, 4001):
        act = ~(done | dead)
        if not act.any():
            break
        pa = pos[act]
        dead_now = lenA[pa] == 0
        pa = np.where(dead_now, pa, nxtA[pa])
        idx = np.flatnonzero(act)
        dead[idx[dead_now]] = True
        over = pa >= nb
        pa = np.minimum(pa, nb)
        pos[idx] = pa
        steps[idx] = k
        hit = isb[pa] & ~over & ~dead_now
        done[idx[hit]] = True
        dead[idx[over & ~dead_now]] = True  # ran off the block's data unsynced
    A = dict(
        syms=steps[done].tolist(),
        bits=(pos[done] - starts[done]).tolist(),
        n=len(starts),
        dead=int(dead.sum()),
        unsynced=int((~done & ~dead).sum()),
    )
    # ---- B: fixed guessed table t, first true boundary reached, and whether table matches
    B = []
    for t in range(nT):
        lt = L[t, :nb]
        nxt = np.arange(nb) + lt
        pos = starts.copy()
        steps = np.zeros(len(pos), np.int64)
        done = np.zeros(len(pos), bool)
        dead = np.zeros(len(pos), bool)
        for k in range(1, 4001):
            act = ~(done | dead)
            if not act.any():
                break
            idx = np.flatnonzero(act)
            pa = pos[idx]
            dn = lt[np.minimum(pa, nb - 1)] == 0
            pa = np.where(dn, pa, nxt[np.minimum(pa, nb - 1)])
            dead[idx[dn]] = True
            over = pa >= nb
            pa = np.minimum(pa, nb)
            pos[idx] = pa
            steps[idx] = k
            hit = isb[pa] & ~over & ~dn
            done[idx[hit]] = True
            dead[idx[over & ~dn]] = True
        right = done & (otab[np.minimum(pos, nb - 1)] == t)
        B.append(
            dict(
                synced=int(done.sum()),
                right_table=int(right.sum()),
                dead=int(dead.sum()),
                syms=steps[done].tolist(),
            )
        )
    # ---- C: selector structure
    C = dict(
        n_tables=nT,
        n_sel=len(sel),
        same_next=int((sels[1:] == sels[:-1]).sum()),
        pairs=len(sel) - 1,
        n_syms=n_syms,
        data_bits=nb,
    )
    # ---- D: phase error on a true boundary: believed index = true + d
    nD = min(n_starts // 4, n_syms // 2)
    d_choices = np.array([1, 7, 25, 49, 50, -1, -25, -50])
    srt = rng.integers(0, max(1, n_syms - 6000), size=nD)
    d = rng.choice(d_choices, size=nD)
    p = barr[srt].copy()
    s_bel = srt + d
    ok = s_bel >= 0
    p, s_bel, srt, d = p[ok], s_bel[ok], srt[ok], d[ok]
    nselm = len(sel) - 1
    relock = np.zeros(len(p), np.int64) - 1
    alive = np.ones(len(p), bool)
    on_true = np.ones(len(p), bool)  # still on true boundaries (with wrong index)
    left_true = np.zeros(len(p), bool)
    for k in range(1, 5001):
        act = alive & (relock < 0)
        if not act.any():
            break
        idx = np.flatnonzero(act)
        pa = p[idx]
        g = np.minimum(s_bel[idx] // 50, nselm)
        t = sels[g]
        l = L[t, np.minimum(pa, nb - 1)]
        dn = (l == 0) | (pa >= nb)
        alive[idx[dn]] = False
        pa = pa + l
        p[idx] = pa
        s_bel[idx] += 1
        inb = pa < nb
        tb = np.zeros(len(idx), bool)
        tb[inb] = isb[pa[inb]]
        left_true[idx[~tb]] = True
        good = tb & (bidx[np.minimum(pa, nb)] == s_bel[idx]) & ~dn
        relock[idx[good]] = k
    D = dict(
        n=len(p),
        relocked=int((relock >= 0).sum()),
        relocked_without_leaving=int(((relock >= 0) & ~left_true).sum()),
        relock_syms=relock[relock >= 0].tolist(),
        died=int((~alive).sum()),
        by_d={
            int(x): [int((d == x).sum()), int(((relock >= 0) & (d == x)).sum())] for x in d_choices
        },
    )
    return dict(A=A, B=B, C=C, D=D), data_end


def main():
    """Walk every stream and block of the file, then summarise."""
    ap = argparse.ArgumentParser()
    ap.add_argument("file")
    ap.add_argument("--starts", type=int, default=2000)
    ap.add_argument("--seed", type=int, default=20261127)
    ap.add_argument("--json")
    ap.add_argument("--max-blocks", type=int, default=0)
    a = ap.parse_args()
    data = open(a.file, "rb").read()
    br = Bits(data)
    rng = np.random.default_rng(a.seed)
    results = []
    while br.p + 32 <= br.n:
        if br.get(24) != 0x425A68:
            raise SystemExit(f"no stream header at bit {br.p - 24}")
        br.get(8)
        while True:
            magic = br.get(48)
            if magic == END_MAGIC:
                br.get(32)
                br.p = (br.p + 7) // 8 * 8
                break
            if magic != BLOCK_MAGIC:
                raise SystemExit(f"bad magic at bit {br.p - 48}")
            eob, tables, sels = parse_block(br)
            r, used = analyse_block(br.bits, br.p, eob, tables, sels, rng, a.starts)
            br.p += used
            results.append(r)
            if len(results) % 25 == 0:
                print(f"{len(results)} blocks", file=sys.stderr)
            if a.max_blocks and len(results) >= a.max_blocks:
                break
        if a.max_blocks and len(results) >= a.max_blocks:
            break
    summarise(results)
    if a.json:
        json.dump(results, open(a.json, "w"))


def pct(v, qs=(50, 90, 99, 99.9)):
    """Format percentiles, max and mean of v."""
    v = np.asarray(v)
    return (
        " ".join(f"p{q}={np.percentile(v, q):.0f}" for q in qs)
        + f" max={v.max():.0f} mean={v.mean():.2f}"
    )


def summarise(rs):
    """Print the corpus-wide totals of measurements A-D."""
    print(f"blocks: {len(rs)}")
    A_s = sum((r["A"]["syms"] for r in rs), [])
    A_b = sum((r["A"]["bits"] for r in rs), [])
    nA = sum(r["A"]["n"] for r in rs)
    print(
        f"A oracle-table sync: {len(A_s)}/{nA} synced, dead {sum(r['A']['dead'] for r in rs)}, "
        f"unsynced {sum(r['A']['unsynced'] for r in rs)}"
    )
    print(f"  codewords to sync: {pct(A_s)}")
    print(f"  bits to sync:      {pct(A_b)}")
    nsy = sum(r["C"]["n_syms"] for r in rs)
    nbi = sum(r["C"]["data_bits"] for r in rs)
    print(f"  mean bits/symbol {nbi / nsy:.3f}; symbols {nsy}; data bits {nbi}")
    syn = sum(sum(b["synced"] for b in r["B"]) for r in rs)
    rt = sum(sum(b["right_table"] for b in r["B"]) for r in rs)
    tot = sum(len(r["B"]) * r["A"]["n"] for r in rs)
    Bs = sum((b["syms"] for r in rs for b in r["B"]), [])
    print(
        f"B guessed-table: synced {syn}/{tot}, at a boundary of the guessed table {rt}; codewords {pct(Bs)}"
    )
    # best-of-tables per start is not tracked per start; report per-block mean right-table share
    share = [sum(b["right_table"] for b in r["B"]) / r["A"]["n"] for r in rs]
    print(f"  right-table syncs per start (sum over guesses), mean {np.mean(share):.3f}")
    nt = [r["C"]["n_tables"] for r in rs]
    print(f"C tables a block: {dict(zip(*np.unique(nt, return_counts=True)))}")
    same = sum(r["C"]["same_next"] for r in rs)
    pairs = sum(r["C"]["pairs"] for r in rs)
    print(f"  consecutive groups sharing a table: {same}/{pairs} = {same / pairs:.4f}")
    nD = sum(r["D"]["n"] for r in rs)
    rl = sum(r["D"]["relocked"] for r in rs)
    rlw = sum(r["D"]["relocked_without_leaving"] for r in rs)
    died = sum(r["D"]["died"] for r in rs)
    print(
        f"D phase error: relocked {rl}/{nD} within 5000 syms ({rlw} without ever leaving the true boundaries), died {died}"
    )
    byd = {}
    for r in rs:
        for k, (n, ok) in r["D"]["by_d"].items():
            byd.setdefault(k, [0, 0])
            byd[k][0] += n
            byd[k][1] += ok
    print(f"  by d: {byd}")
    rs_ = sum((r["D"]["relock_syms"] for r in rs), [])
    if rs_:
        print(f"  symbols to relock: {pct(rs_)}")


if __name__ == "__main__":
    main()
