#!/usr/bin/env python3
"""The selector-count obstruction: can a chunk started mid-block find its symbol index.

Usage: python3 huffman_index_hypotheses.py FILE.bz2 [--chunks N] [--radius R] [--seed S]

A within-block speculative scanner knows a block's start and end bit (a magic finder gives
the next block's magic), its selectors and its tables, but not the symbol index at a chunk's
first bit, and the index picks the table.  Two questions:

  E1. estimate: predict the group at bit c from the selectors alone, each group costing
      50 x its table's expected code length (sum 2^-l * l over its codewords, Kraft-normalised),
      scaled to the block's known data length.  Error in groups (true - predicted).
  E2. collapse: start every index hypothesis s in [s_hat - R, s_hat + R] at bit c, decode each,
      and count the distinct (bit, index) states after h bits.  The state determines the rest
      of the block, so one surviving state means the chunk's output is exact from there; the
      true state is always among the survivors unless the true index is outside the window.
"""

import argparse
import sys

import numpy as np

sys.path.insert(0, __import__("os").path.dirname(__file__))
from huffman_self_sync import (  # noqa: E402
    BLOCK_MAGIC,
    END_MAGIC,
    W,
    Bits,
    parse_block,
    peek_array,
)

HORIZONS = (256, 1024, 4096, 16384)


def expected_len(lt):
    """A table's expected code length, Kraft-normalised."""
    # code lengths of the codewords: lt is filled per 2^W entry; recover from runs
    lens = []
    i = 0
    n = len(lt)
    while i < n:
        l = int(lt[i])
        if l == 0:
            i += 1
            continue
        lens.append(l)
        i += 1 << (W - l)
    lens = np.array(lens, float)
    w = 2.0**-lens
    return float((w * lens).sum() / w.sum())


def block(bits, start, eob, tables, sels, rng, n_chunks, R):
    """Decode one block's true path and run E1 and E2 at random chunk starts."""
    max_syms = 50 * len(sels)
    end = min(start + W * max_syms + 64, len(bits))
    peek = peek_array(bits, start, end)
    L = np.stack([t[0][peek] for t in tables])
    S = np.stack([t[1][peek] for t in tables])
    Ll = [r.tolist() for r in L]
    Sl = [r.tolist() for r in S]
    sel = sels.tolist()
    p, i, bounds = 0, 0, []
    while True:
        t = sel[i // 50]
        bounds.append(p)
        s = Sl[t][p]
        p += Ll[t][p]
        i += 1
        if s == eob:
            break
    nb, n_syms = p, i
    barr = np.array(bounds, np.int64)
    isb = np.zeros(nb + 1, bool)
    isb[barr] = True
    bidx = np.full(nb + 1, -1, np.int64)
    bidx[barr] = np.arange(n_syms)
    # E1: predicted group boundaries from selectors x expected lengths, scaled to nb
    el = np.array([expected_len(t[0]) for t in tables])
    gbits = 50.0 * el[sels]
    cum = np.concatenate([[0.0], np.cumsum(gbits)])
    cum *= nb / cum[-1]
    out = []
    nsel = len(sel)
    for _ in range(n_chunks):
        c = (
            int(rng.integers(nb // 10, nb - nb // 10 - HORIZONS[-1] - 64))
            if nb > 3 * HORIZONS[-1]
            else None
        )
        if c is None:
            break
        true_s = int(np.searchsorted(barr, c, side="left"))  # index of the first symbol at/after c
        g_true = int(np.searchsorted(barr, c, side="right") - 1) // 50
        g_hat = int(np.searchsorted(cum, c, side="right") - 1)
        frac = (c - cum[g_hat]) / max(1e-9, cum[g_hat + 1] - cum[g_hat])
        s_hat = int(round(50 * (g_hat + frac)))
        hs = np.arange(max(0, s_hat - R), min(n_syms, s_hat + R) + 1)
        pos = np.full(len(hs), c, np.int64)
        sb = hs.copy()
        alive = np.ones(len(hs), bool)
        rec = {h: None for h in HORIZONS}
        crossed = {h: np.zeros(len(hs), bool) for h in HORIZONS}
        st = {h: np.zeros((len(hs), 2), np.int64) for h in HORIZONS}
        while True:
            act = alive & ~crossed[HORIZONS[-1]]
            if not act.any():
                break
            idx = np.flatnonzero(act)
            pa = pos[idx]
            g = np.minimum(sb[idx] // 50, nsel - 1)
            l = L[sels[g], np.minimum(pa, nb - 1)]
            dn = (l == 0) | (pa >= nb)
            alive[idx[dn]] = False
            pos[idx] = pa + l
            sb[idx] += 1
            for h in HORIZONS:
                new = ~crossed[h][idx] & (pos[idx] >= c + h) & ~dn
                j = idx[new]
                crossed[h][j] = True
                st[h][j, 0] = pos[j]
                st[h][j, 1] = sb[j]
        for h in HORIZONS:
            ok = crossed[h]
            states = {tuple(x) for x in st[h][ok]}
            # is the true state among them: a state on a true boundary with its true index
            true_in = any(
                sb_ == bidx[min(p_, nb)] for p_, sb_ in states if p_ <= nb and isb[min(p_, nb)]
            )
            rec[h] = (len(states), bool(true_in))
        out.append(
            dict(
                err=g_true - g_hat,
                s_err=true_s - s_hat,
                n_h=len(hs),
                rec=rec,
                in_window=bool(hs[0] <= true_s <= hs[-1]),
            )
        )
    return out, nb


def main():
    """Walk every stream and block of the file, then summarise."""
    ap = argparse.ArgumentParser()
    ap.add_argument("file")
    ap.add_argument("--chunks", type=int, default=3)
    ap.add_argument("--radius", type=int, default=400)
    ap.add_argument("--seed", type=int, default=20261127)
    ap.add_argument("--max-blocks", type=int, default=0)
    a = ap.parse_args()
    br = Bits(open(a.file, "rb").read())
    rng = np.random.default_rng(a.seed)
    res = []
    nblk = 0
    while br.p + 32 <= br.n and not (a.max_blocks and nblk >= a.max_blocks):
        assert br.get(24) == 0x425A68
        br.get(8)
        while True:
            m = br.get(48)
            if m == END_MAGIC:
                br.get(32)
                br.p = (br.p + 7) // 8 * 8
                break
            assert m == BLOCK_MAGIC, br.p
            eob, tables, sels = parse_block(br)
            r, used = block(br.bits, br.p, eob, tables, sels, rng, a.chunks, a.radius)
            br.p += used
            res += r
            nblk += 1
            if nblk % 25 == 0:
                print(f"{nblk} blocks", file=sys.stderr)
            if a.max_blocks and nblk >= a.max_blocks:
                break
    err = np.array([r["err"] for r in res])
    serr = np.array([r["s_err"] for r in res])
    print(
        f"blocks {nblk}, chunk starts {len(res)}, radius {a.radius} ({2 * a.radius + 1} hypotheses)"
    )
    ae = np.abs(err)
    print(
        "E1 group-index error |g_true - g_hat|: "
        + " ".join(f"p{q}={np.percentile(ae, q):.0f}" for q in (50, 90, 99))
        + f" max={ae.max()} mean={ae.mean():.2f}"
    )
    ase = np.abs(serr)
    print(
        "   symbol-index error |s_true - s_hat|: "
        + " ".join(f"p{q}={np.percentile(ase, q):.0f}" for q in (50, 90, 99))
        + f" max={ase.max()}"
    )
    print(
        f"   true index inside the +-{a.radius} window: {sum(r['in_window'] for r in res)}/{len(res)}"
    )
    for h in HORIZONS:
        n = np.array([r["rec"][h][0] for r in res])
        ti = sum(r["rec"][h][1] for r in res)
        one = int((n == 1).sum())
        print(
            f"E2 after {h:>5} bits: distinct states p50={np.percentile(n, 50):.0f} p90={np.percentile(n, 90):.0f} "
            f"max={n.max()}; collapsed to one: {one}/{len(n)}; true state present: {ti}/{len(n)}"
        )


if __name__ == "__main__":
    main()
