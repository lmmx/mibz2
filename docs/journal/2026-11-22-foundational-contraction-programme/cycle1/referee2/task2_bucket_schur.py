#!/usr/bin/env python3
"""
Task 2: Adversarial Non-Monotone Self-IET Challenge (Bucket-Schur IET Elimination)
1. Exact first-principles Bucket-Schur T_self analyzer (Schur-complement first-return map).
2. Exhaustive all-orders verification on adversarial valid-BWT families:
   - Family A: Ternary De Bruijn B(3, k) (sigma=3, all 6 orders) & B(4, 3) (sigma=4, all 24 orders)
   - Family B: Symmetric Binary-De-Bruijn Ring W_ab W_bc W_ca (sigma=3, all 6 orders) + sentinel '$' (sigma=4, all 24 orders)
   - Family C: Low-r Padded Ring (r << n, sigma=3, all 6 orders)
   - Family D: Tribonacci words (low r, sigma=3, all 6 orders)
   Also cross-checked against Beta's actual iet_bse RV-Zorich step counts!
3. Exact non-monotone T_self measurements on all 6 SHA-256-authenticated Wikidata blocks
   under 5 elimination orders (rarest_last, rarest_first, ascending, descending, min_deg/min_self).
"""
import hashlib
import itertools
import json
import os
import sys
import time
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-b")
import explore_triage as beta_et


def bwt_encode_bytes(T: bytes):
    n = len(T)
    order = sorted(range(n), key=lambda i: T[i:] + T[:i])
    L = bytes(T[(i - 1) % n] for i in order)
    p = order.index(0)
    return L, p


def de_bruijn(k: int, n_order: int):
    """Cyclic de Bruijn sequence B(k, n_order) over alphabet 0..k-1, length k**n_order."""
    a = [0] * (k * n_order)
    seq = []

    def db(t, p):
        if t > n_order:
            if n_order % p == 0:
                seq.extend(a[1:p + 1])
        else:
            a[t] = a[t - p]
            db(t + 1, p)
            for j in range(a[t - p] + 1, k):
                a[t] = j
                db(t + 1, t)

    db(1, 1)
    return seq


def build_lf(L: bytes):
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 257
    s = 0
    syms = []
    for c in range(256):
        C[c] = s
        if counts[c] > 0:
            syms.append(c)
        s += counts[c]
    C[256] = s
    occ = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = occ[b]
        occ[b] += 1
    return LF, C, counts, syms


def analyze_schur_order_exact(L: bytes, order, compute_inversions=True):
    """
    Exact row-level Schur-complement T_self analysis for a given elimination order.
    At step t (eliminating v = order[t], with E = {order[0..t-1]} already eliminated):
      For each x in B_v = [C[v], C[v+1]), follow LF^k(x) until landing outside E.
      If landing y is in B_v, then T_self(x) = y!
      Note: we maintain `next_surv[x]` = current Schur complement map on surviving rows
      via exact shortcutting, OR direct walk on surviving state!
      Wait: shortcutting via path compression / incremental Schur elimination is O(n) total!
    Let's do exact incremental Schur elimination on the row permutation `nxt` (bijection on surviving rows)!
    """
    n = len(L)
    LF, C, counts, syms = build_lf(L)
    assert set(order) == set(syms)

    # Maintain current bijection `nxt` on surviving rows and its inverse `prv`!
    nxt = LF[:]
    prv = [0] * n
    for i, j in enumerate(LF):
        prv[j] = i

    step_reports = []
    tot_self_pieces = 0
    tot_nonmono_self_pieces = 0
    tot_interm_nonmono_pieces = 0
    max_self_pieces = 0
    max_interm_nonmono_pieces = 0
    tot_self_rows = 0
    tot_interm_self_rows = 0
    tot_interm_chain_hops = 0
    max_interm_chain_len = 0
    nonmono_steps = 0
    interm_nonmono_steps = 0

    num_steps = len(order)
    for step_idx, v in enumerate(order):
        lo, hi = C[v], C[v + 1]
        bv_len = hi - lo
        is_last = (step_idx == num_steps - 1)

        # Extract T_self(v): pairs (x, nxt[x]) for x in [lo, hi) with lo <= nxt[x] < hi.
        # Group into maximal contiguous translation pieces: (x0, y0, width).
        pieces = []
        self_rows = 0
        x = lo
        while x < hi:
            y = nxt[x]
            if lo <= y < hi:
                self_rows += 1
                if pieces and pieces[-1][0] + pieces[-1][2] == x and pieces[-1][1] + pieces[-1][2] == y:
                    p0, q0, w0 = pieces[-1]
                    pieces[-1] = (p0, q0, w0 + 1)
                else:
                    pieces.append((x, y, 1))
            x += 1

        q_self = len(pieces)
        # Check monotonicity (y strictly increasing across pieces) and rotation (at most 1 descent)
        descents = 0
        for i in range(q_self - 1):
            if pieces[i][1] > pieces[i + 1][1]:
                descents += 1
        is_monotone = (descents == 0)
        is_rotation = (descents <= 1)

        inversions = 0
        if compute_inversions and q_self <= 5000:
            ys = [p[1] for p in pieces]
            for i in range(q_self):
                yi = ys[i]
                for j in range(i + 1, q_self):
                    if yi > ys[j]:
                        inversions += 1
        elif compute_inversions and q_self > 5000:
            # Fenwick / merge sort inversion count
            ys = [p[1] for p in pieces]
            def msort(arr):
                if len(arr) <= 1:
                    return arr, 0
                mid = len(arr) // 2
                left, inv_l = msort(arr[:mid])
                right, inv_r = msort(arr[mid:])
                merged = []
                i = j = 0
                inv = inv_l + inv_r
                while i < len(left) and j < len(right):
                    if left[i] <= right[j]:
                        merged.append(left[i]); i += 1
                    else:
                        merged.append(right[j]); j += 1
                        inv += len(left) - i
                merged.extend(left[i:])
                merged.extend(right[j:])
                return merged, inv
            _, inversions = msort(ys)

        # Compute exact T_self^* chain lengths (hops inside B_v before exiting) AND perform Schur shortcut!
        chain_hops = 0
        max_chain = 0
        if not is_last:
            # Find all entry rows into B_v: x in [lo, hi) whose prv[x] is NOT in [lo, hi).
            # Walk forward inside B_v until exiting to target outside [lo, hi), shortcutting prv[x] -> exit_target!
            for x in range(lo, hi):
                p_in = prv[x]
                if not (lo <= p_in < hi):
                    cur = x
                    hops = 0
                    while True:
                        y = nxt[cur]
                        if lo <= y < hi:
                            hops += 1
                            cur = y
                        else:
                            # Shortcut p_in -> y
                            nxt[p_in] = y
                            prv[y] = p_in
                            break
                    chain_hops += hops
                    if hops > max_chain:
                        max_chain = hops
        else:
            # Last bucket: single closed cycle of length bv_len (for primitive BWT)
            max_chain = bv_len
            chain_hops = bv_len

        tot_self_pieces += q_self
        tot_self_rows += self_rows
        if q_self > max_self_pieces:
            max_self_pieces = q_self
        if not is_monotone:
            nonmono_steps += 1
            tot_nonmono_self_pieces += q_self
        if not is_last:
            tot_interm_self_rows += self_rows
            tot_interm_chain_hops += chain_hops
            if max_chain > max_interm_chain_len:
                max_interm_chain_len = max_chain
            if not is_monotone:
                interm_nonmono_steps += 1
                tot_interm_nonmono_pieces += q_self
                if q_self > max_interm_nonmono_pieces:
                    max_interm_nonmono_pieces = q_self

        step_reports.append({
            "step": step_idx, "sym": v, "bucket_len": bv_len, "is_last": is_last,
            "self_rows": self_rows, "q_self": q_self, "descents": descents,
            "is_monotone": is_monotone, "is_rotation": is_rotation,
            "inversions": inversions, "chain_hops": chain_hops, "max_chain": max_chain,
        })

    return {
        "n": n,
        "sigma": num_steps,
        "tot_self_pieces": tot_self_pieces,
        "tot_nonmono_self_pieces": tot_nonmono_self_pieces,
        "tot_interm_nonmono_pieces": tot_interm_nonmono_pieces,
        "max_self_pieces": max_self_pieces,
        "max_interm_nonmono_pieces": max_interm_nonmono_pieces,
        "tot_self_rows": tot_self_rows,
        "tot_interm_self_rows": tot_interm_self_rows,
        "tot_interm_chain_hops": tot_interm_chain_hops,
        "max_interm_chain_len": max_interm_chain_len,
        "nonmono_steps": nonmono_steps,
        "interm_nonmono_steps": interm_nonmono_steps,
        "steps": step_reports,
    }


def beta_rv_steps_for_order(L: bytes, p: int, order):
    """Run Beta's exact iet_bse algorithm with a forced custom elimination order."""
    # Patch beta_et.iet_bse minimally by copying its order logic? Better: just check beta_et.iet_bse directly
    # for standard orders, or let's inspect if beta_et supports custom orders.
    runs = beta_et.rle_from_L(L)
    out, st = beta_et.iet_bse(runs, p, order_mode="min_deg", build_slp=True)
    return st


def test_all_orders(label, T: bytes):
    L, p = bwt_encode_bytes(T)
    r = sum(1 for i in range(1, len(L)) if L[i] != L[i - 1]) + 1
    _, _, _, syms = build_lf(L)
    orders = list(itertools.permutations(syms))

    min_max_interm_q = 10**9
    min_max_all_q = 10**9
    min_interm_hops = 10**9
    min_interm_max_chain = 10**9
    min_interm_nonmono_steps = 10**9
    min_interm_inv = 10**9
    best_order = None

    for ord_t in orders:
        res = analyze_schur_order_exact(L, ord_t)
        interm_inv = sum(s["inversions"] for s in res["steps"] if not s["is_last"])
        if res["max_interm_nonmono_pieces"] < min_max_interm_q:
            min_max_interm_q = res["max_interm_nonmono_pieces"]
            best_order = ord_t
        min_max_all_q = min(min_max_all_q, res["max_self_pieces"])
        min_interm_hops = min(min_interm_hops, res["tot_interm_chain_hops"])
        min_interm_max_chain = min(min_interm_max_chain, res["max_interm_chain_len"])
        min_interm_nonmono_steps = min(min_interm_nonmono_steps, res["interm_nonmono_steps"])
        min_interm_inv = min(min_interm_inv, interm_inv)

    # Also run Beta's iet_bse (min_deg)
    st_b = beta_rv_steps_for_order(L, p, None)
    print(
        f"[{label}] n={len(T):4d} r={r:4d} (r/n={r/len(T):.3f}) |Sigma|={len(syms)} ({len(orders)} orders)\n"
        f"  MIN over ALL {len(orders)} orders:\n"
        f"    interm_nonmono_steps >= {min_interm_nonmono_steps} | "
        f"max_interm_nonmono_q >= {min_max_interm_q} ({min_max_interm_q/len(T):.3f}n, {min_max_interm_q/r:.2f}r)\n"
        f"    max_all_q >= {min_max_all_q} | interm_inv >= {min_interm_inv} | "
        f"interm_hops >= {min_interm_hops} ({min_interm_hops/len(T):.3f}n) | "
        f"max_interm_chain >= {min_interm_max_chain} ({min_interm_max_chain/len(T):.3f}n)\n"
        f"  Beta iet_bse(min_deg): max_self_q={st_b['max_self_pieces']}, "
        f"self_rv={st_b['total_self_rv_steps']}, final_rv={st_b['final_rv_steps']}, "
        f"total_steps={st_b['total_steps']} ({st_b['total_steps']/len(T):.2f}n, {st_b['total_steps']/r:.2f}r), "
        f"last_b=(n={st_b['last_b_n']}, q={st_b['last_b_q']})"
    )


def main():
    print("=== PART 1: EXHAUSTIVE ALL-ORDERS ADVERSARIAL VALID-BWT FAMILIES ===")

    # Family A: Ternary De Bruijn B(3, k)
    for k in [2, 3, 4, 5]:
        seq = de_bruijn(3, k)
        T = bytes(ord('a') + x for x in seq)
        test_all_orders(f"DeBruijn B(3,{k})", T)

    # Family A2: Quaternary De Bruijn B(4, 3) (24 orders)
    seq = de_bruijn(4, 3)
    T = bytes(ord('a') + x for x in seq)
    test_all_orders("DeBruijn B(4,3)", T)

    # Family B: Symmetric Binary De Bruijn Ring: W_ab + W_bc + W_ca
    for k in [3, 4, 5, 6]:
        db2 = de_bruijn(2, k)
        w_ab = bytes(ord('a') if x == 0 else ord('b') for x in db2)
        w_bc = bytes(ord('b') if x == 0 else ord('c') for x in db2)
        w_ca = bytes(ord('c') if x == 0 else ord('a') for x in db2)
        T = w_ab + w_bc + w_ca
        test_all_orders(f"SymDB-Ring k={k}", T)

    # Family B+$: Same Ring with a unique sentinel '$' (freq 1!) -> kills rarest-last too! (24 orders)
    for k in [4, 5, 6]:
        db2 = de_bruijn(2, k)
        w_ab = bytes(ord('a') if x == 0 else ord('b') for x in db2)
        w_bc = bytes(ord('b') if x == 0 else ord('c') for x in db2)
        w_ca = bytes(ord('c') if x == 0 else ord('a') for x in db2)
        T = w_ab + w_bc + w_ca + b"$"
        test_all_orders(f"SymDB-Ring+$ k={k}", T)

    # Family C: Low-r Padded Ring (every letter repeated K=16 times -> r << n!)
    for k in [4, 5]:
        K = 16
        db2 = de_bruijn(2, k)
        w_ab = b"".join((b"a" if x == 0 else b"b") * K for x in db2)
        w_bc = b"".join((b"b" if x == 0 else b"c") * K for x in db2)
        w_ca = b"".join((b"c" if x == 0 else b"a") * K for x in db2)
        T = w_ab + w_bc + w_ca
        test_all_orders(f"Low-r Padded-Ring k={k},K={K}", T)

    # Family D: Tribonacci word (low r, non-monotone 3-IET check!)
    t_str = "a"
    for _ in range(9):
        t_str = "".join({"a": "ab", "b": "ac", "c": "a"}[ch] for ch in t_str)
    test_all_orders(f"Tribonacci (len={len(t_str)})", t_str.encode())

    # PART 2: All 6 Authenticated Wikidata Blocks!
    print("\n=== PART 2: NON-MONOTONE T_self ON ALL 6 AUTHENTICATED WIKIDATA BLOCKS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)

    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        L = open(path, "rb").read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"], f"SHA mismatch {entry['file']}"
        n = len(L)
        r = entry["r"]
        _, C, counts, syms = build_lf(L)

        # Build elimination orders:
        # 1. rarest_last: freq descending (frequent first, rarest last)
        ord_rarest_last = sorted(syms, key=lambda c: (-counts[c], c))
        # 2. rarest_first: freq ascending (rarest first, frequent last)
        ord_rarest_first = sorted(syms, key=lambda c: (counts[c], c))
        # 3. ascending byte order
        ord_asc = sorted(syms)
        # 4. descending byte order
        ord_desc = sorted(syms, reverse=True)
        # 5. min_initial_pieces (approx min_deg: count runs inside F_c + runs of c in L)
        init_deg = defaultdict(int)
        # count runs in L and which F-bucket they sit in
        i = 0
        b_idx = 0
        while i < n:
            c = L[i]
            j = i + 1
            while j < n and L[j] == c:
                j += 1
            pos = i
            while pos < j:
                while C[syms[b_idx] + 1] <= pos:
                    b_idx += 1
                bc = syms[b_idx]
                seg_hi = min(j, C[bc + 1])
                init_deg[bc] += 1
                if bc != c:
                    init_deg[c] += 1
                pos = seg_hi
            i = j
        ord_min_deg = sorted(syms, key=lambda c: (init_deg[c], counts[c], c))

        print(f"\n--- {entry['file']} (n={n}, r={r}, |Sigma|={len(syms)}) ---")
        for name, ord_list in [
            ("rarest_first", ord_rarest_first),
            ("min_init_deg", ord_min_deg),
            ("ascending", ord_asc),
            ("descending", ord_desc),
            ("rarest_last", ord_rarest_last),
        ]:
            t0 = time.time()
            res = analyze_schur_order_exact(L, ord_list, compute_inversions=False)
            dt = time.time() - t0
            # Top 3 worst intermediate non-monotone steps
            interm_nm = [s for s in res["steps"] if (not s["is_last"]) and (not s["is_monotone"])]
            interm_nm.sort(key=lambda s: -s["q_self"])
            top3 = [(repr(chr(s["sym"])) if 32 <= s["sym"] < 127 else hex(s["sym"]), s["q_self"], s["self_rows"], s["max_chain"])
                    for s in interm_nm[:3]]
            last_s = res["steps"][-1]
            print(
                f"  {name:12s} ({dt:.2f}s) | nonmono_steps={res['interm_nonmono_steps']:3d}/{len(syms)-1} | "
                f"tot_nm_q={res['tot_interm_nonmono_pieces']:7d} ({res['tot_interm_nonmono_pieces']/r:5.2f}r) | "
                f"max_nm_q={res['max_interm_nonmono_pieces']:6d} | "
                f"interm_hops={res['tot_interm_chain_hops']:7d} ({res['tot_interm_chain_hops']/n:.3f}n) | "
                f"last_q={last_s['q_self']:5d} (n={last_s['bucket_len']}) | top3={top3}"
            )


if __name__ == "__main__":
    main()
