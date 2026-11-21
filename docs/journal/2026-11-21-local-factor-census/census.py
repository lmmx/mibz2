"""The local-factor census (RH-shape lead C, docs/journal/2026-11-21-the-shape-of-the-rh-proof.md).

usage: python3 census.py <dir holding manifest.json and block-*.L>   (branch data/admitted-bwt)

For every retained row (a row of a maximal run of at most LIMIT = 16 rows, as Open keeps them),
its local configuration at depth k is (offset in its run; symbol and length of the k maximal runs
from its run, in row order). For each k = 1..4 and m = 4, 8, 16 it reports:

  configs   distinct configurations among the retained rows
  top       share of retained rows in the CAT most populous configurations
  det       share of retained rows in output-determining configurations (every row of the
            configuration emits the same next m bytes) that fit the catalogue: the CAT most
            populous determining configurations -- the kill rule's figure
  det2      as det, configurations of at least 2 rows only (a 1-row configuration determines
            its output trivially)
  modal2    diagnostic: in configurations of at least 2 rows, the share of their rows whose
            word is the configuration's modal word (a catalogue's best single guess)
  rel95     diagnostic: as det, with "determining" relaxed to: the modal m-byte word covers
            at least 95% of the configuration's rows

Catalogue: 256 KiB at 32 bytes an entry (a 16-byte fragment and up to 16 bytes of key and
table slot) = CAT = 8192 entries.

Emission: exact text order by LF inversion (stable rank, as in the data README). Row x emits
T[pos(x)], the byte L[x]; "fwd" is the next m bytes in decoder output order, T[pos(x)..pos(x)+m),
i.e. L[x], L[Psi x], ...; "bwd" (diagnostic) is the LF direction, L[x], L[LF x], ...
Windows wrap cyclically at the block's end (at most 15 rows per block are affected).
"""
import hashlib, json, sys
import numpy as np

LIMIT = 16
CAT = 256 * 1024 // 32
KS = (1, 2, 3, 4)
MS = (4, 8, 16)


def load(d, b):
    raw = open(f"{d}/{b['file']}", "rb").read()
    h = hashlib.sha256(raw).hexdigest()
    assert h == b["sha256"], (b["file"], h)
    L = np.frombuffer(raw, np.uint8)
    assert len(L) == b["n"]
    return L


def text_positions(L, p):
    """pos[x] = text position of the byte L[x]; T = the block's bytes (costmap.py's walk)."""
    n = len(L)
    order = np.argsort(L, kind="stable")
    LF = np.empty(n, np.int64)
    LF[order] = np.arange(n)
    rows = np.empty(n, np.int64)
    r = p
    lf = LF.tolist()
    for k in range(n):
        rows[k] = r
        r = lf[r]
    assert r == p, "LF is not one cycle"
    pos = np.empty(n, np.int64)
    pos[rows] = n - 1 - np.arange(n)
    T = L[rows][::-1].copy()
    return pos, T


def words(T, start, m, step):
    """m-byte words of T from each start, stepping +1 (fwd) or -1 (bwd), cyclic; as 2 x u64."""
    n = len(T)
    idx = (start[:, None] + step * np.arange(m)[None, :]) % n
    w = T[idx].astype(np.uint64)
    out = np.zeros((len(start), 2), np.uint64)
    for j in range(m):
        out[:, j // 8] |= w[:, j] << np.uint64(8 * (j % 8))
    return out


def ids(a):
    return np.unique(a, axis=0, return_inverse=True)[1].ravel()


def score(cfg, emit):
    """cfg, emit: per-row ids. Returns the census row (shares of len(cfg) rows)."""
    N = len(cfg)
    ncfg = cfg.max() + 1
    size = np.bincount(cfg, minlength=ncfg)
    pair = ids(np.stack([cfg, emit], 1))
    pc = np.bincount(pair)
    pair_cfg = np.zeros(len(pc), np.int64)
    pair_cfg[pair] = cfg
    distinct = np.bincount(pair_cfg, minlength=ncfg)
    modal = np.zeros(ncfg, np.int64)
    np.maximum.at(modal, pair_cfg, pc)

    def cat(mask):
        s = np.sort(size[mask])[::-1][:CAT]
        return s.sum() / N

    det = distinct == 1
    return dict(
        configs=int(ncfg),
        top=cat(np.ones(ncfg, bool)),
        det=cat(det),
        det2=cat(det & (size >= 2)),
        det_all=size[det].sum() / N,
        rel95=cat(modal >= 0.95 * size),
        modal2=modal[size >= 2].sum() / max(size[size >= 2].sum(), 1),
    )


def census(L, pos, T, k_list=KS):
    n = len(L)
    brk = np.flatnonzero(np.diff(L)) + 1
    starts = np.concatenate([[0], brk])
    lens = np.diff(np.concatenate([starts, [n]]))
    sym = L[starts].astype(np.int64)
    run_of = np.repeat(np.arange(len(starts)), lens)
    keep = lens[run_of] <= LIMIT
    rows = np.flatnonzero(keep)
    j = run_of[rows]
    off = rows - starts[j]
    R = len(starts)
    # sentinel past the last run
    sym_p = np.concatenate([sym, np.full(4, 256)])
    len_p = np.concatenate([lens, np.zeros(4, np.int64)])
    cfgs = {}
    for k in k_list:
        cols = [off]
        for t in range(k):
            cols += [sym_p[np.minimum(j + t, R)], len_p[np.minimum(j + t, R)]]
        cfgs[k] = np.stack(cols, 1)
    em = {(d, m): words(T, pos[rows], m, s) for d, s in (("fwd", 1), ("bwd", -1)) for m in MS}
    return dict(n=n, runs=R, retained=len(rows), cfgs=cfgs, em=em)


def main():
    d = sys.argv[1]
    man = json.load(open(f"{d}/manifest.json"))
    pooled = {"cfgs": {k: [] for k in KS}, "em": {}}
    out = []
    for b in man["blocks"]:
        L = load(d, b)
        pos, T = text_positions(L, b["primary_index"])
        c = census(L, pos, T)
        name = b["file"][:-2]
        print(f"\n## {name}: n {c['n']}, runs {c['runs']}, retained {c['retained']} "
              f"({c['retained']/c['n']*100:.1f}% of rows); first bytes {bytes(T[:24])!r}")
        for k in KS:
            cid = ids(c["cfgs"][k])
            pooled["cfgs"][k].append(c["cfgs"][k])
            for (dr, m), e in c["em"].items():
                if k == KS[0]:
                    pooled["em"].setdefault((dr, m), []).append(e)
                s = score(cid, ids(e))
                out.append(dict(block=name, k=k, dir=dr, m=m, **s))
        report([o for o in out if o["block"] == name])
    print("\n## pooled: one catalogue for all six blocks")
    rows = []
    for k in KS:
        cid = ids(np.concatenate(pooled["cfgs"][k]))
        for (dr, m), es in pooled["em"].items():
            s = score(cid, ids(np.concatenate(es)))
            rows.append(dict(block="pooled", k=k, dir=dr, m=m, **s))
    report(rows)
    out += rows
    json.dump(out, open("census.json", "w"), indent=0)


def report(rows):
    print("| k | dir | m | configs | top | det | det2 | det (no cap) | rel95 | modal2 |")
    print("|---|---|---|---:|---:|---:|---:|---:|---:|---:|")
    for o in rows:
        print(f"| {o['k']} | {o['dir']} | {o['m']} | {o['configs']} | {o['top']*100:.1f}% | "
              f"{o['det']*100:.1f}% | {o['det2']*100:.1f}% | {o['det_all']*100:.1f}% | "
              f"{o['rel95']*100:.1f}% | {o['modal2']*100:.1f}% |")


if __name__ == "__main__":
    main()
