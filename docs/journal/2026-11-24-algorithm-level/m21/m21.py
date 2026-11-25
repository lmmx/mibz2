import json, sys, hashlib, numpy as np
D = sys.argv[1]
man = json.load(open(f"{D}/data/manifest.json"))
K = 3
out = []
for b in man["blocks"]:
    raw = open(f"{D}/data/{b['file']}", "rb").read()
    assert hashlib.sha256(raw).hexdigest() == b["sha256"]
    L = np.frombuffer(raw, dtype=np.uint8); n = len(L); assert n == b["n"]
    p = b["primary_index"]
    cnt = np.bincount(L, minlength=256); C = np.concatenate([[0], np.cumsum(cnt)[:-1]])
    order = np.argsort(L, kind="stable")          # F-row j comes from L-row order[j]
    LF = np.empty(n, np.int64); LF[order] = np.arange(n)
    # walk
    pos = np.empty(n, np.int64); i = p; LFl = LF.tolist(); posl = [0]*n
    for t in range(n):
        posl[i] = (n - t) % n; i = LFl[i]
    assert i == p
    pos = np.array(posl, np.int64)
    assert len(np.unique(pos)) == n, "not a single cycle"
    T = np.empty(n, np.uint8); T[(pos - 1) % n] = L     # L[row] = T[pos(row)-1]
    assert pos[p] == 0 and L[p] == T[n-1]
    runs_r = 1 + int(np.count_nonzero(L[1:] != L[:-1])); assert runs_r == b["r"], (runs_r, b["r"])
    neq_prev = np.ones(n, bool); neq_prev[1:] = L[1:] != L[:-1]
    neq_next = np.ones(n, bool); neq_next[:-1] = L[:-1] != L[1:]
    lone = neq_prev & neq_next; h1 = int(lone.sum())
    # text-position masks: occ[q] true if landmark occurrence starts at text pos q
    T2 = (T.astype(np.int64) << 8) | np.roll(T, -1)
    big = np.bincount(T2, minlength=65536)
    c = int(cnt.argmax()); pr = int(big.argmax())
    rowAt = np.empty(n, np.int64); rowAt[pos] = np.arange(n)
    res = dict(file=b["file"], n=n, r=b["r"], h1=h1, sigma=int((cnt > 0).sum()))
    for name, occ0, lab in [("byte", T == c, repr(bytes([c]))), ("pair", T2 == pr, repr(bytes([pr >> 8, pr & 255])))]:
        occ = occ0.copy(); levels = []; prev_sz = n; prev_mask = np.ones(n, bool); string = lab
        Q = 0; Dsum = 0
        for j in range(1, K + 1):
            if j > 1:
                # extend landmark one byte leftward: most frequent predecessor byte of current occurrences
                qs = np.nonzero(occ)[0]; pb = T[(qs - (j - 1)) % n]
                cj = int(np.bincount(pb, minlength=256).argmax())
                new = np.zeros(n, bool); new[qs[pb == cj]] = True
                occ = new; string = repr(bytes([cj])) + "+" + string
            qs = np.nonzero(occ)[0]; m = len(qs)
            g = np.diff(np.concatenate([qs, [qs[0] + n]]))          # text gaps (cyclic)
            # rounds at this level = number of level-(j-1) points in each gap (R_{j-1} steps)
            cum = np.cumsum(prev_mask); pc = np.concatenate([[0], cum])
            # count of prev points in (q_i, q_{i+1}] cyclic
            a = qs; bnd = np.concatenate([qs[1:], [qs[0] + n]])
            def cntle(x):  # prev points in [0,x] for x possibly >= n
                return np.where(x < n, cum[np.minimum(x, n-1)], cum[-1] + cum[np.maximum(x - n, 0)])
            steps = cntle(bnd) - cntle(a)
            assert steps.sum() == prev_sz
            rows = rowAt[(qs) % n] if False else None
            # landmark rows: row whose suffix starts at q
            Yrows = rowAt[qs]
            lone_in = int(lone[Yrows].sum())
            if name=='byte' and j==1: assert Yrows.min()==C[c] and Yrows.max()==C[c]+cnt[c]-1 and m==cnt[c]
            Q += prev_sz; Dsum += int(steps.max())
            levels.append(dict(k=j, land=string, Y=m, gap_mean=float(g.mean()), gap_p99=float(np.percentile(g, 99)),
                               gap_max=int(g.max()), rounds=int(steps.max()), Q=Q + m, D=Dsum + m,
                               lone_in_Y=lone_in, lone_frac_Y=lone_in / h1, Y_frac=m / n))
            prev_sz = m; prev_mask = occ
        res[name] = levels
    out.append(res)
    print(json.dumps(res), flush=True)
json.dump(out, open(f"{D}/m21.json", "w"), indent=1)
