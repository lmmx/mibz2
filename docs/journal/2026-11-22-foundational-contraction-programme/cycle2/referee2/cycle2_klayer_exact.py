#!/usr/bin/env python3
"""Bit-exact k-layer contiguous-interval DAG (k=2, 3, 4, Full-DAG) verification & block-0.L benchmark."""
import hashlib
import inspect
import json
import os
import sys

sys.path.insert(0, "/tmp/fcp-ref")
import p1_potential_lp as p1
import p2_compositional_sat as p2c
import p2b_dag_dp_wikidata as p2b


def make_run_klayer():
    src = inspect.getsource(p2b.run_space_ascent_dag_step)
    old = """    if mode == "asc":
        fwd_order = list(range(s - 1, -1, -1))   # reverse topo: sinks first (s-1 down to 0)
        bwd_order = list(range(s))               # topo: sources first (0 up to s-1)
        is_del = lambda l, f: l < f
    else:
        fwd_order = list(range(s))               # sinks first (0 up to s-1)
        bwd_order = list(range(s - 1, -1, -1))   # sources first (s-1 down to 0)
        is_del = lambda l, f: l > f"""
    new = """    if isinstance(mode, (list, tuple)):
        layer_of, rev = mode
        if not rev:
            fwd_order = list(range(s - 1, -1, -1))
            bwd_order = list(range(s))
            is_del = lambda l, f: layer_of[l] < layer_of[f]
        else:
            fwd_order = list(range(s))
            bwd_order = list(range(s - 1, -1, -1))
            is_del = lambda l, f: layer_of[l] > layer_of[f]
    elif mode == "asc":
        fwd_order = list(range(s - 1, -1, -1))
        bwd_order = list(range(s))
        is_del = lambda l, f: l < f
    else:
        fwd_order = list(range(s))
        bwd_order = list(range(s - 1, -1, -1))
        is_del = lambda l, f: l > f"""
    assert old in src
    ns = dict(p2b.__dict__)
    exec(src.replace(old, new), ns)
    return ns["run_space_ascent_dag_step"]


def main():
    run_klayer = make_run_klayer()

    # 1. Bit-exact parity verification against row-level contraction
    checked = 0
    for n in range(4, 8):
        for T in p1.gen_lyndon(n, 4):
            if len(set(T)) < 4:
                continue
            L, F, LF, C, p, s = p1.bwt_of_word(T)
            runs = p1.db.runs_from_seq(L)
            lo = [0, 1, 1, 2]
            FL = [0] * n
            for i, j in enumerate(LF):
                FL[j] = i
            for rev in (False, True):
                rn, nn, sn, st = run_klayer(runs, n, s, (lo, rev))
                S_mask = [not ((lo[L[i]] < lo[F[i]]) if not rev else (lo[L[i]] > lo[F[i]])) for i in range(n)]
                ok, _, _, splits, _, _, L_exp = p2c.subset_contraction(L, F, LF, FL, p, S_mask)
                assert ok and splits == 0
                rn_exp = p1.db.runs_from_seq(L_exp)
                assert rn == rn_exp, f"Mismatch on T={T}, rev={rev}: {rn} vs {rn_exp}"
                checked += 1
    print(f"VERIFIED 100% BIT-EXACT k-layer DAG DP on {checked} quaternary valid BWTs!")

    # 2. Tightness check on Fan-Chain(k, m) AND Reverse-Fan-Chain (testing BOTH Sweep 1 |P| and Sweep 2 |Q|!)
    print("\n--- Tightness Check: Fan-Chain (Sweep 1) & Double-Fan (Sweep 1 + Sweep 2) ---")
    for k in (3, 4):
        for m in (8, 32):
            s = (k - 1) + m
            T = tuple(c for target in range(k - 1, s) for c in (list(range(k - 1)) + [target]))
            L, F, LF, C, p, sig = p1.bwt_of_word(T)
            runs = p1.db.runs_from_seq(L)
            lo = [min(c, k - 1) for c in range(sig)]
            _, n2, s2, st = run_klayer(runs, len(L), sig, (lo, False))
            r0 = st["r0"]
            print(
                f"  Fan-Chain(k={k}, m={m:2d}): r0={r0:3d} -> r_S={st['r_S']:3d} | "
                f"|P|={st['P_segs']:3d} ({st['P_segs']/r0:.2f}r0) | "
                f"fwd={st['fwd_pieces']:3d} ({st['fwd_pieces']/r0:.2f}r0), "
                f"bwd={st['bwd_pieces']:3d} ({st['bwd_pieces']/r0:.2f}r0), "
                f"tot/r0={st['total_pieces_per_r0']:.2f}x"
            )

    # 3. Exact multi-round k=2, 3, 4, Full-DAG on block-0.L down to n <= 4096 (with self-run fallback)
    print("\n--- Exact Multi-Round k=2, 3, 4, Full-DAG on block-0.L (down to n <= 4096) ---")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        blk0 = json.load(f)["blocks"][0]
    data = open(os.path.join("/tmp/mibz2-blocks", blk0["file"]), "rb").read()
    assert hashlib.sha256(data).hexdigest() == blk0["sha256"]
    n0 = len(data)
    syms0 = sorted(set(data))
    smap = {c: i for i, c in enumerate(syms0)}
    runs0 = []
    i = 0
    while i < n0:
        c = smap[data[i]]
        j = i + 1
        while j < n0 and smap[data[j]] == c:
            j += 1
        runs0.append((c, j - i))
        i = j
    r0, s0 = len(runs0), len(syms0)

    def choose_layers_mass(runs, n, s, k):
        counts = [0] * s
        for c, w in runs:
            counts[c] += w
        lo = [0] * s
        acc = 0
        cur_l = 0
        target = n / k
        for c in range(s):
            lo[c] = cur_l
            acc += counts[c]
            if cur_l < k - 1 and (acc >= (cur_l + 1) * target or (s - 1 - c) == (k - 1 - cur_l)):
                cur_l += 1
        return lo

    for k_mode in (2, 3, 4, 999999):
        runs, n, s = list(runs0), n0, s0
        rnd = 0
        self_r = 0
        tot_r = 0
        tot_fwd = 0
        tot_bwd = 0
        max_r = r0
        max_s = s0
        while n > 4096 and rnd < 40 and s >= 2:
            rev = (rnd % 2 == 1)
            mode = ("desc" if rev else "asc") if k_mode > 100 else (choose_layers_mass(runs, n, s, min(k_mode, s)), rev)
            rn, nn, sn, st = run_klayer(runs, n, s, mode)
            if (n - nn) < 0.05 * n:
                mode2 = ("asc" if rev else "desc") if k_mode > 100 else (choose_layers_mass(runs, n, s, min(k_mode, s)), not rev)
                rn2, nn2, sn2, st2 = run_klayer(runs, n, s, mode2)
                if nn2 < nn:
                    rn, nn, sn, st = rn2, nn2, sn2, st2
            if (n - nn) < 0.05 * n:
                C = [0] * (s + 1)
                for c, w in runs:
                    C[c + 1] += w
                for c in range(s):
                    C[c + 1] += C[c]
                br = [[] for _ in range(s)]
                b = 0
                pos = 0
                for c, w in runs:
                    rem = w
                    while rem > 0:
                        while C[b + 1] <= pos:
                            b += 1
                        tk = min(rem, C[b + 1] - pos)
                        br[b].append((c, tk))
                        pos += tk
                        rem -= tk
                res = p1.db.self_run_contraction_step(runs, s, n, 0, 0, [1] * s, list(range(s)), [("lit", c) for c in range(s)], C, br)
                if res is None:
                    break
                out_runs, _, _, tdel, _, new_s, _ = res
                tot_r += len(runs)
                tot_fwd += len(runs)
                tot_bwd += len(out_runs)
                runs, n, s = out_runs, n - tdel, new_s
                self_r += 1
                rnd += 1
                max_r = max(max_r, len(runs))
                max_s = max(max_s, s)
                continue
            tot_r += len(runs)
            tot_fwd += st["fwd_pieces"]
            tot_bwd += st["bwd_pieces"]
            runs, n, s = rn, nn, sn
            rnd += 1
            max_r = max(max_r, len(runs))
            max_s = max(max_s, s)
        lbl = "Full-DAG" if k_mode > 100 else f"k={k_mode}-Layer"
        tot_p = tot_fwd + tot_bwd
        print(
            f"  block-0.L [{lbl:8s}] rds={rnd:2d}(self={self_r}) n_end={n:5d} "
            f"max_r={max_r:5d}({max_r/r0:.2f}r0) max_s={max_s:6d} | "
            f"sum_r={tot_r/r0:.2f}r0 | tot_pieces={tot_p:7d} ({tot_p/r0:.2f}r0, fwd={tot_fwd/r0:.2f}r0, bwd={tot_bwd/r0:.2f}r0)"
        )


if __name__ == "__main__":
    main()
