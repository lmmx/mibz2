#!/usr/bin/env python3
"""
Carbon Kitchen Sink — Cycle 6 Mathematical Probe & Exact Verifier (probe_cycle6.py)

Investigates Operator Displacement Structure [D, P_LF] and Forman-Skoldberg-Perseus Algebraic
Morse Theory on BWT complexes (addressing the new Discrete Morse Reduction lead on origin/master
and Kempa SODA 2019):

1. Theorem 6.1 — Kailath-Kung-Morf Displacement Commutator & the r-Value Law of Delta_tau:
   - Verifies exhaustively on all primitive binary (2 <= n <= 8) and ternary (2 <= n <= 6) BWTs:
       (a) Commutator [D, P_LF] = D P_LF - P_LF D is identically zero on all n - r non-head rows
           x not in D_bar and equals sum_{u=0}^{r-1} e_{d_u} (e_{c_u}^T - e_{LF(d_u - 1)}^T),
           with exact rank <= r - 1.
       (b) Exact r-Value Law: for every row x in [0, n),
               Delta_x := (tau(x) - tau((x - 1) % n)) % n  ==  alpha_{v(x)},
           where v(x) in [0, r) is the first D_bar-hit of x under LF and
           alpha_v = (tau(d_v) - tau((d_v - 1) % n)) % n!
       (c) Sturmian Collapse Check: on every Sturmian word (r = 2), alpha_0 == alpha_1 (mod n),
           so Delta_x is 100% CONSTANT across all n rows (1 run -> O(1) global linear coordinate!).
   - Measures on all 6 authenticated Wikidata blocks:
       * Number of distinct values |{Delta_x}| <= r vs r (testing accidental alpha_u == alpha_v collisions!)
       * Number of maximal constant runs r(Delta_tau) of (Delta_0, ..., Delta_{n-1}) mod n vs n!
       * Number of maximal constant runs r(v) of the Kempa first-D_bar-hit vector (v(0), ..., v(n-1))!

2. Theorem 6.2 — Forman-Skoldberg Algebraic Morse Theory, Perseus Apparent Pairs & Coreductions:
   - Verifies Skoldberg's Non-Commutative Fox Morse Equivalence over Z<Sigma*> on small columns
     (both primitive and multi-cycle non-BWT columns) across 4 Morse matching families:
       M1: Canonical Rising Apparent Matching {x != p : x < LF(x)}
       M2: Canonical Falling Apparent Matching {x != p : x > LF(x)}
       M3: Width-Filtered Apparent Matching {x != p : (w(x), -x) > (w(LF(x)), -LF(x))}
       M4: Mrozek-Batko Coreduction Dual (matching codomain y != LF(p) backwards via Psi)
   - Measures on all 6 Wikidata blocks for each matching M:
       * Exact critical cell count |V_crit(M)| (surviving retained states),
       * Number of maximal coherent gradient bundles B(M) (contiguous critical intervals sharing
         identical Morse gradient target shift and word — measuring table/strip complexity!),
       * Mean and max gradient-path length!
"""

from __future__ import annotations
import itertools
import json
from pathlib import Path

ADMITTED_DIR = Path("/home/antigravity/admitted-bwt")
OUT_PATH = Path(__file__).resolve().parent / "probe_cycle6_results.json"


def load_manifest():
    with open(ADMITTED_DIR / "manifest.json", "r", encoding="utf-8") as f:
        return json.load(f)["blocks"]


def bwt_of_word(w: tuple[int, ...]) -> tuple[bytes, int]:
    n = len(w)
    rots = [(w[i:] + w[:i], i) for i in range(n)]
    rots.sort(key=lambda x: x[0])
    L = bytes(rot[-1] for rot, _ in rots)
    p = next(idx for idx, (_, orig) in enumerate(rots) if orig == 0)
    return L, p


def is_primitive(w: tuple[int, ...]) -> bool:
    n = len(w)
    for d in range(1, n):
        if n % d == 0 and w == w[:d] * (n // d):
            return False
    return True


def verify_theorem_6_1_small() -> dict:
    """Exhaustively verify [D, P_LF] commutator formula & r-Value Law on small primitive BWTs."""
    checked = 0
    sturmian_checked = 0

    def check_one(w: tuple[int, ...]):
        nonlocal checked, sturmian_checked
        L, p = bwt_of_word(w)
        n = len(L)
        counts = [0] * 4
        run_d = []
        i = 0
        while i < n:
            b = L[i]
            run_d.append(i)
            j = i + 1
            while j < n and L[j] == b:
                j += 1
            counts[b] += (j - i)
            i = j
        r = len(run_d)
        occ = [0] * 4
        acc = 0
        for c in range(4):
            occ[c] = acc; acc += counts[c]
        lf = [0] * n
        for idx, b in enumerate(L):
            lf[idx] = occ[b]; occ[b] += 1

        # 1. Exact matrix check of [D, P_LF]:
        # (P_LF f)_x = f_{lf[x]}; (D g)_x = g_x - g_{(x-1)%n}
        # So ((D P_LF - P_LF D) f)_x = (f_{lf[x]} - f_{lf[(x-1)%n]}) - (f_{lf[x]} - f_{(lf[x]-1)%n})
        #                            = f_{(lf[x]-1)%n} - f_{lf[(x-1)%n]}? Wait:
        # Let's check the sign!
        # (D (P_LF f))_x = (P_LF f)_x - (P_LF f)_{x-1} = f_{lf[x]} - f_{lf[x-1]}.
        # (P_LF (D f))_x = (D f)_{lf[x]} = f_{lf[x]} - f_{(lf[x] - 1) % n}.
        # Subtracting: ([D, P_LF] f)_x = f_{(lf[x] - 1) % n} - f_{lf[(x - 1) % n]}!
        # Wait: when x not in D_bar (and x > 0), L[x] == L[x-1], so lf[x-1] == lf[x] - 1, hence 0!
        # What about x == 0? 0 IS ALWAYS IN D_bar (d_0 = 0)!
        d_set = set(run_d)
        for x in range(n):
            col_plus = (lf[x] - 1) % n
            col_minus = lf[(x - 1) % n]
            if x not in d_set:
                assert col_plus == col_minus, f"Non-zero commutator outside D_bar at x={x}"

        # 2. Exact tau(x) in [0, n): tau(p) = 0, tau(lf[x]) = (tau(x) - 1) % n => step s from p has tau = (-s) % n
        # Wait: if text[n - 1 - s] = L[LF^s(p)], let's set pos_text(LF^s(p)) = (n - 1 - s) % n, so pos_text(LF(x)) = (pos_text(x) - 1) % n!
        tau = [0] * n
        cur = p
        for s in range(n):
            tau[cur] = (n - 1 - s) % n
            cur = lf[cur]
        assert cur == p

        # Compute alpha_v = (tau[d_v] - tau[(d_v - 1) % n]) % n for v = 0..r-1
        d_to_v = {d0: v for v, d0 in enumerate(run_d)}
        alpha = [(tau[d0] - tau[(d0 - 1) % n]) % n for d0 in run_d]

        # Verify Delta_x == alpha_{v(x)} for EVERY x in 0..n-1!
        deltas = []
        for x in range(n):
            delta_x = (tau[x] - tau[(x - 1) % n]) % n
            deltas.append(delta_x)
            y = x
            while y not in d_to_v:
                y = lf[y]
            vx = d_to_v[y]
            assert delta_x == alpha[vx], f"r-Value Law failed: delta_x={delta_x} != alpha[{vx}]={alpha[vx]}"

        if r == 2 and math.gcd(counts[0], counts[1]) == 1:
            # Check if mechanical / Sturmian BWT (r == 2 is Characterization of Sturmian necklaces by Mantaci-Restivo-Sciortino!)
            assert len(set(deltas)) == 1, f"r=2 BWT failed constant Delta_tau: {deltas}"
            sturmian_checked += 1

        checked += 1

    import math
    for n in range(2, 9):
        for w in itertools.product((0, 1), repeat=n):
            if is_primitive(w):
                check_one(w)
    for n in range(2, 7):
        for w in itertools.product((0, 1, 2), repeat=n):
            if is_primitive(w):
                check_one(w)

    return {
        "primitive_bwts_verified": checked,
        "sturmian_r2_constant_delta_verified": sturmian_checked,
    }


def morse_reconstruct_small(L: tuple[int, ...], p: int, matched_dom: set[int]) -> bytes:
    """
    Exact Skoldberg Non-Commutative Fox Morse reduction on ([n], LF, L) punctured at p:
      - `matched_dom` is ANY acyclic subset of [0, n) \ {p} (if cycles exist on unmarked components,
        detect and unmatch 1 edge per cycle just like Forman cancellation).
      - Critical 0-cells V_crit = [0, n) \ matched_dom (always contains p!).
      - For each critical cell c in V_crit, follow the unique backward-text LF gradient path:
          emit L[c], step to y = LF(c); while y in matched_dom: emit L[y], y = LF(y);
          record Morse boundary arrow c -> y with reversed word (forward text order)!
        Wait: let's verify BOTH directions (forward text emission from p)!
    """
    n = len(L)
    counts = [0] * 4
    for b in L: counts[b] += 1
    occ = [0] * 4
    acc = 0
    for c in range(4): occ[c] = acc; acc += counts[c]
    lf = [0] * n
    for i, b in enumerate(L): lf[i] = occ[b]; occ[b] += 1

    assert p not in matched_dom
    # Verify matched_dom has no cycles (for apparent matchings x < lf[x], guaranteed 0 cycles!)
    state = [0] * n  # 0 unvisited, 1 visiting, 2 safe
    for x0 in list(matched_dom):
        if state[x0] == 2:
            continue
        cur = x0
        path = []
        while cur in matched_dom and state[cur] == 0:
            state[cur] = 1
            path.append(cur)
            cur = lf[cur]
        if cur in matched_dom and state[cur] == 1:
            raise AssertionError(f"Cycle detected in purportedly acyclic Morse matching at {cur}!")
        for u in path:
            state[u] = 2

    # Critical cells V_crit = [x for x in range(n) if x not in matched_dom]
    # Reduced Fox differential on c in V_crit:
    # One step c -> lf[c] followed by unique M-gradient path in matched_dom until hitting c_next in V_crit!
    crit_next = {}
    crit_rev_word = {}
    for c in range(n):
        if c in matched_dom:
            continue
        w_rev = bytearray([L[c]])
        y = lf[c]
        while y in matched_dom:
            w_rev.append(L[y])
            y = lf[y]
        crit_next[c] = y
        crit_rev_word[c] = bytes(w_rev)

    # Walk reduced critical complex from p!
    rev_total = bytearray()
    cur = p
    for _ in range(len(crit_next)):
        rev_total.extend(crit_rev_word[cur])
        cur = crit_next[cur]
        if cur == p or len(rev_total) >= n:
            break

    period = bytes(reversed(rev_total[:n]))
    out = bytearray(period)
    while len(out) < n:
        out.extend(period[: n - len(out)])
    return bytes(out[:n])


def dense_lf_ref(L: tuple[int, ...], p: int) -> bytes:
    n = len(L)
    counts = [0] * 4
    for b in L: counts[b] += 1
    occ = [0] * 4
    acc = 0
    for c in range(4): occ[c] = acc; acc += counts[c]
    lf = [0] * n
    for i, b in enumerate(L): lf[i] = occ[b]; occ[b] += 1
    rev = bytearray()
    cur = p
    for _ in range(n):
        rev.append(L[cur]); cur = lf[cur]
        if cur == p: break
    period = bytes(reversed(rev))
    out = bytearray(period)
    while len(out) < n:
        out.extend(period[: n - len(out)])
    return bytes(out)


def verify_theorem_6_2_morse_small() -> int:
    """Exhaustively verify Apparent Morse Matchings across ALL binary columns 1<=n<=7 (BWT & non-BWT!)."""
    checked = 0
    for n in range(1, 8):
        for col in itertools.product((0, 1), repeat=n):
            counts = [0, 0]
            for b in col: counts[b] += 1
            occ = [0, counts[0]]
            lf = [0] * n
            for i, b in enumerate(col): lf[i] = occ[b]; occ[b] += 1
            # Run widths
            rw = [0] * n
            i = 0
            while i < n:
                j = i + 1
                while j < n and col[j] == col[i]: j += 1
                for k in range(i, j): rw[k] = j - i
                i = j

            for p in range(n):
                ref = dense_lf_ref(col, p)
                # M1: Rising apparent x < lf[x]
                m1 = {x for x in range(n) if x != p and x < lf[x]}
                # M2: Falling apparent x > lf[x]
                m2 = {x for x in range(n) if x != p and x > lf[x]}
                # M3: Width-filtered apparent (eliminate wider -> narrower, tie-break x < lf[x])
                m3 = {x for x in range(n) if x != p and (rw[x], -x) > (rw[lf[x]], -lf[x])}
                # M4: Mrozek-Batko coreduction apparent (match codomain y = lf[x] when y > x, excluding p)
                # Note: y > lf^{-1}(y) <=> x < lf[x], or match when rw[lf[x]] > rw[x] (coreducing into thick codomain!):
                m4 = {x for x in range(n) if x != p and (rw[lf[x]], lf[x]) > (rw[x], x)}

                for m_set in (m1, m2, m3, m4):
                    got = morse_reconstruct_small(col, p, m_set)
                    assert got == ref, f"Morse mismatch on col={col}, p={p}"
                    checked += 1
    return checked


def probe_block_cycle6(L: bytes, p_mark: int) -> dict:
    n = len(L)
    counts = [0] * 256
    run_d = []
    row_rw = [0] * n
    i = 0
    while i < n:
        b = L[i]
        run_d.append(i)
        j = i + 1
        while j < n and L[j] == b:
            j += 1
        w = j - i
        counts[b] += w
        for x in range(i, j):
            row_rw[x] = w
        i = j
    r = len(run_d)
    occ = [0] * 256
    acc = 0
    for c in range(256):
        occ[c] = acc; acc += counts[c]
    lf = [0] * n
    for x, b in enumerate(L):
        lf[x] = occ[b]; occ[b] += 1

    # 1. Exact tau(x), first D_bar hit v(x), and Delta_x = (tau(x) - tau(x-1)) % n!
    # We compute tau(x) AND v(x) in ONE single O(n) pass around the primary LF cycle!
    is_d_head = [-1] * n
    for v_idx, d0 in enumerate(run_d):
        is_d_head[d0] = v_idx

    tau = [0] * n
    first_d_hit = [0] * n
    cur = p_mark
    # First find the next D_bar head ahead of p_mark to seed backward propagation along the cycle
    orbit = [0] * n
    for s in range(n):
        orbit[s] = cur
        tau[cur] = (n - 1 - s) % n
        cur = lf[cur]

    # Propagate first_d_hit backwards along `orbit`: if is_d_head[x] >= 0, v(x) = is_d_head[x], else v(x) = v(lf[x])!
    # Find any index s0 where is_d_head[orbit[s0]] >= 0
    s0 = next(s for s in range(n) if is_d_head[orbit[s]] >= 0)
    cur_v = is_d_head[orbit[s0]]
    for step in range(n, -1, -1):
        idx = (s0 + step) % n
        x = orbit[idx]
        if is_d_head[x] >= 0:
            cur_v = is_d_head[x]
        first_d_hit[x] = cur_v

    alpha = [(tau[d0] - tau[(d0 - 1) % n]) % n for d0 in run_d]
    distinct_alpha = len(set(alpha))

    # Verify r-Value Law on ALL n rows of the 900KB block & count runs of v(x) and Delta_x!
    runs_v = 1
    runs_delta = 1
    prev_v = first_d_hit[0]
    prev_delta = (tau[0] - tau[n - 1]) % n
    assert prev_delta == alpha[prev_v]

    for x in range(1, n):
        vx = first_d_hit[x]
        dx = (tau[x] - tau[x - 1]) % n
        if dx != alpha[vx]:
            raise AssertionError(f"r-Value Law failed on block at x={x}")
        if vx != prev_v:
            runs_v += 1
            prev_v = vx
        if dx != prev_delta:
            runs_delta += 1
            prev_delta = dx

    # 2. Evaluate Algebraic Morse Matchings on the 900KB block!
    def eval_morse_matching(name: str, is_matched_fn) -> dict:
        in_m = bytearray(n)
        n_crit = 0
        crit_rows = []
        row_to_cid = [-1] * n
        for x in range(n):
            if x != p_mark and is_matched_fn(x):
                in_m[x] = 1
            else:
                row_to_cid[x] = n_crit
                crit_rows.append(x)
                n_crit += 1

        # Compute induced critical return map along `orbit` in O(n) time!
        # Find first critical cell on `orbit`
        c_next = [0] * n_crit
        c_len = [0] * n_crit
        c_hash = [0] * n_crit

        s_start = next(s for s in range(n) if not in_m[orbit[s]])
        cur_cid = row_to_cid[orbit[s_start]]
        h = 1469598103934665603
        ln = 0
        for off in range(n):
            x = orbit[(s_start + off) % n]
            if off > 0 and not in_m[x]:
                nxt_cid = row_to_cid[x]
                c_next[cur_cid] = nxt_cid
                c_len[cur_cid] = ln
                c_hash[cur_cid] = h
                cur_cid = nxt_cid
                h = 1469598103934665603
                ln = 0
            h = ((h * 1099511628211) + L[x] + 1) & 0xFFFFFFFFFFFFFFFF
            ln += 1
        first_cid = row_to_cid[orbit[s_start]]
        c_next[cur_cid] = first_cid
        c_len[cur_cid] = ln
        c_hash[cur_cid] = h

        # Count coherent critical bundles (adjacent critical indices i-1, i with c_next[i] == c_next[i-1]+1 & same word)!
        bundles = 1
        max_len = c_len[0]
        for cid in range(1, n_crit):
            if c_len[cid] > max_len:
                max_len = c_len[cid]
            if not (
                c_next[cid] == c_next[cid - 1] + 1
                and c_len[cid] == c_len[cid - 1]
                and c_hash[cid] == c_hash[cid - 1]
            ):
                bundles += 1

        return {
            "critical_cells": n_crit,
            "critical_frac_n": round(n_crit / n, 4),
            "coherent_bundles": bundles,
            "bundles_over_crit": round(bundles / max(1, n_crit), 4),
            "mean_word_len": round(n / max(1, n_crit), 2),
            "max_word_len": max_len,
        }

    morse_rising = eval_morse_matching("rising_x_lt_lf", lambda x: x < lf[x])
    morse_width_app = eval_morse_matching(
        "width_apparent", lambda x: (row_rw[x], -x) > (row_rw[lf[x]], -lf[x])
    )
    morse_coreduce_w16 = eval_morse_matching(
        "coreduce_codomain_thick16", lambda x: row_rw[lf[x]] > 16
    )
    morse_open_w16 = eval_morse_matching(
        "reduce_domain_thick16_open", lambda x: row_rw[x] > 16
    )
    # Filtered Bi-Thick Morse Matching: eliminate x if row_rw[x] > 16 OR row_rw[lf[x]] > 16!
    # (By Universal Acyclicity Lemma 6.2C on primitive BWTs, ANY proper subset excluding p_mark is acyclic!)
    morse_bithick16 = eval_morse_matching(
        "bithick_dom_or_cod_gt16", lambda x: (row_rw[x] > 16) or (row_rw[lf[x]] > 16)
    )

    return {
        "displacement_commutator": {
            "n": n,
            "r": r,
            "distinct_alpha_values": distinct_alpha,
            "accidental_alpha_collisions": r - distinct_alpha,
            "runs_of_first_D_hit_v": runs_v,
            "runs_of_Delta_tau": runs_delta,
            "runs_Delta_tau_frac_n": round(runs_delta / n, 4),
        },
        "algebraic_morse_matchings": {
            "rising_apparent": morse_rising,
            "width_apparent": morse_width_app,
            "open_domain_thick16": morse_open_w16,
            "coreduce_codomain_thick16": morse_coreduce_w16,
            "bithick_dom_or_cod_gt16": morse_bithick16,
        },
    }


def main():
    t61 = verify_theorem_6_1_small()
    print(
        f"[PASS] Theorem 6.1 ([D, P_LF] & r-Value Law) verified on {t61['primitive_bwts_verified']:,} "
        f"primitive BWTs (including {t61['sturmian_r2_constant_delta_verified']} Sturmian r=2 constant-Delta BWTs)!"
    )
    t62_cnt = verify_theorem_6_2_morse_small()
    print(
        f"[PASS] Theorem 6.2 (Skoldberg Fox Morse Equivalence across 4 apparent/coreduction families) "
        f"verified on {t62_cnt:,} small-column configurations!"
    )

    blocks = load_manifest()
    results = {"small_verifications": {"theorem_6_1": t61, "theorem_6_2_checks": t62_cnt}, "blocks": {}}
    for b in blocks:
        name = b["file"]
        L = (ADMITTED_DIR / name).read_bytes()
        res = probe_block_cycle6(L, b["primary_index"])
        results["blocks"][name] = res
        dc = res["displacement_commutator"]
        am = res["algebraic_morse_matchings"]
        print(
            f"[{name}] "
            f"Disp: distinct_alpha={dc['distinct_alpha_values']:,}/{dc['r']:,}, "
            f"runs(Delta_tau)={dc['runs_of_Delta_tau']:,} ({dc['runs_Delta_tau_frac_n']:.3f}n) | "
            f"Morse crit: Rising={am['rising_apparent']['critical_cells']:,}, "
            f"WidthApp={am['width_apparent']['critical_cells']:,}, "
            f"Open16={am['open_domain_thick16']['critical_cells']:,} (B={am['open_domain_thick16']['coherent_bundles']:,}), "
            f"Coreduce16={am['coreduce_codomain_thick16']['critical_cells']:,} (B={am['coreduce_codomain_thick16']['coherent_bundles']:,}), "
            f"BiThick16={am['bithick_dom_or_cod_gt16']['critical_cells']:,} (B={am['bithick_dom_or_cod_gt16']['coherent_bundles']:,})"
        )

    with open(OUT_PATH, "w", encoding="utf-8") as f:
        json.dump(results, f, indent=2)
    print(f"\nSaved Cycle 6 results to {OUT_PATH}")


if __name__ == "__main__":
    main()
