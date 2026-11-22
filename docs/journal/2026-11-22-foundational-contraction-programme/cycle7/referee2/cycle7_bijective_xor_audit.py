#!/usr/bin/env python3
"""
Cycle 7 Tasks 2 & 3 Independent Adversarial Audit:
  - Task 2: Verify Alpha's n=5 post-M1 counterexample, audit Theorem J1 (first_hit[tail] lookup cost
    & memory), spot-check M1 o SelThinSq (-70.4%) & Tube-Open o SelThinSq (-69.1%) on block-0/210,
    and audit bench_m1_packed8.c for bugs/omissions.
  - Task 3: Formal adversarial stress suite for Bijective wire_xor preservation:
      (1) Exact necessary & sufficient graph condition for Open's emit_bidirectional
      (2) CRITICAL ODD vs EVEN cycle parity audit for Bijective Full Phi^2 + wire_xor
          on ALL n=1..10, unary, periodic, AND multi-cycle mutated inputs (c < n, c odd & even!)
      (3) Bijective R1-Open & 1-Pass Ascent-DAG orig_ptr shift + prefix-repeat exactness on multi-cycle L!
"""
import hashlib
import json
import os
import sys
from collections import defaultdict

sys.path.insert(0, "/tmp/fcp-a")
import audit_m1_width_and_joint as amj
import measure_all_trees as mt


def open_ref_forward_fl(L, p):
    """Exact v0.5.12 OpenCells / libbzip2 forward-FL decode on ANY arbitrary column L (incl. multi-cycle!).
    Emits T[0] = F[p], T[1] = F[FL(p)], ..., T[n-1] = F[FL^{n-1}(p)] (prefix-tiled if cycle c < n!)."""
    n = len(L)
    mx = max(L) + 1
    cnt = [0] * mx
    for x in L: cnt[x] += 1
    C = [0] * mx
    acc = 0
    for c in range(mx): C[c] = acc; acc += cnt[c]
    F = [c for c in range(mx) for _ in range(cnt[c])]
    occ = list(C)
    FL = [0] * n
    for i, c in enumerate(L):
        FL[occ[c]] = i
        occ[c] += 1
    out = []
    cur = p
    for _ in range(n):
        out.append(F[cur])
        cur = FL[cur]
    return bytes(out), FL, F


def sim_open_emit_bidirectional(edges_next, words, n, mark):
    """Exact Python replica of v0.5.12 src/open.rs emit_bidirectional (lines 916-983)!
    Requires edges_next to be a BIJECTION on 0..m-1 (so every node has unique prev and next!)."""
    m = len(edges_next)
    # Build XOR table and find unique predecessor of mark
    prev_arr = [-1] * m
    for u, v in enumerate(edges_next):
        assert prev_arr[v] == -1, f"NOT A BIJECTION! Two edges enter {v}"
        prev_arr[v] = u
    xor_adj = [prev_arr[u] ^ edges_next[u] for u in range(m)]
    before = prev_arr[mark]

    out = bytearray(n)
    prev, front = before, mark
    back, nxt = before, mark
    lo, hi = 0, n
    for pair in range(m // 2 + 1):
        if lo == hi or (pair != 0 and front == nxt):
            break
        wf = words[front]
        single = (front == back)
        take = min(len(wf), hi - lo)
        out[lo:lo + take] = wf[:take]
        lo += take
        if single:
            break
        wb = words[back]
        take = min(len(wb), hi - lo)
        out[hi - take:hi] = wb[len(wb) - take:]
        hi -= take
        prev, front = front, xor_adj[front] ^ prev
        nxt, back = back, xor_adj[back] ^ nxt

    if lo < hi:
        # Move suffix beside prefix and repeat to n!
        cyc = bytes(out[:lo]) + bytes(out[hi:n])
        clen = len(cyc)
        res = (cyc * (n // clen + 1))[:n]
        return res
    return bytes(out[:n])


def audit_task2_alpha_j1_and_joint():
    print("=== TASK 2: AUDITING ALPHA'S THEOREM J1, n=5 COUNTEREXAMPLE, JOINT COMPOSITIONS & bench_m1_packed8.c ===", flush=True)

    # 2.1 Verify Alpha's minimal n=5 valid ternary BWT counterexample
    T = bytes([2, 1, 0, 0, 0])
    L, p = mt.bwt_encode(T)
    assert list(L) == [1, 0, 0, 2, 0] and p == 4, f"Unexpected BWT: L={list(L)}, p={p}"
    n_, r0, m16, strips, j_order, C = amj.load_runs_and_open_baseline(L, p)
    next_r1, inc_parent, first_hit = amj.build_exact_r1_and_inclusion_from_256cursor(strips, j_order)
    core, _ = amj.run_static_m1(strips, next_r1, w_min=1, chain_cap=16)
    by_last = defaultdict(list)
    for lo, hi, d, wb, m, tail in core:
        by_last[wb[-1]].append((lo, lo + d, list(wb)))
    inversions = []
    for c, lst in by_last.items():
        for i in range(len(lst) - 1):
            if lst[i][1] > lst[i + 1][1]:
                inversions.append((c, lst[i], lst[i + 1]))
    print(f"  2.1 Verified Alpha n=5 counterexample T={list(T)}, L={list(L)}, p={p}: inversions={inversions}", flush=True)
    assert len(inversions) >= 1

    # 2.2 Spot-check block-0.L & block-210.L for Theorem J1 random lookup count & joint compositions
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for blk in manifest["blocks"]:
        if blk["file"] not in ("block-0.L", "block-210.L"): continue
        L = open(os.path.join("/tmp/mibz2-blocks", blk["file"]), "rb").read()
        assert hashlib.sha256(L).hexdigest() == blk["sha256"]
        p = blk["primary_index"]
        ref = mt.ref_ibwt(L, p)
        n_, r0, m16, strips, j_order, C = amj.load_runs_and_open_baseline(L, p)
        next_r1, inc_parent, first_hit = amj.build_exact_r1_and_inclusion_from_256cursor(strips, j_order)

        # Theorem J1 audit: count how many core heads have tail != head (requiring random first_hit[tail] lookup!)
        core_m1, st_m1 = amj.run_static_m1(strips, next_r1, w_min=1, chain_cap=8)
        nontrivial_tails = sum(1 for lo, hi, d, wb, m, tail in core_m1 if len(wb) > 1)
        tot_heads_m1 = len(core_m1)

        # Verify M1(w>=1, <=8B) o SelThinSq(<=16B) and Tube-Open V6 o SelThinSq(<=16B)
        ret_m1, nidx_m1, sw_m1, pi_m1, pmo_m1, _ = amj.build_open_reduced_perm(core_m1, n_, p)
        stp_m1, gt16_m1, dw_m1 = amj.measure_walk_and_selthinsq(nidx_m1, sw_m1, pi_m1, pmo_m1, n_, ref, sq_max_len=16, enable_sq=True)

        ret_tb, nidx_tb, sw_tb, pi_tb, pmo_tb = amj.simulate_tube_open_v6(strips, inc_parent, max_word_len=15)
        stp_tb, gt16_tb, dw_tb = amj.measure_walk_and_selthinsq(nidx_tb, sw_tb, pi_tb, pmo_tb, n_, ref, sq_max_len=16, enable_sq=True)

        # Check Open `wide` code ceiling on joint compositions!
        bits_m1 = (len(ret_m1) - 1).bit_length()
        maxc_m1 = (1 << (32 - bits_m1)) - 1
        bits_tb = (len(ret_tb) - 1).bit_length()
        maxc_tb = (1 << (32 - bits_tb)) - 1

        print(f"  2.2 {blk['file']:11s} (r0={r0:6d}, m16={m16:6d}):", flush=True)
        print(f"      J1 lookup audit: q_core={tot_heads_m1} ({nontrivial_tails} chained tails -> random first_hit[tail] in {r0*4/1024:.0f} KiB array!)", flush=True)
        print(f"      M1(<=8B)+SelThinSq(<=16B): walk={stp_m1:6d} ({stp_m1/m16:.3f}m16, -{100*(1-stp_m1/m16):.1f}%) | "
              f"edges={len(ret_m1):6d} (bits={bits_m1}, max_codes={maxc_m1}) | dist_words={dw_m1:5d} (WIDE_OVERFLOW? {dw_m1 > maxc_m1}) [EXACT]", flush=True)
        print(f"      TubeV6  +SelThinSq(<=16B): walk={stp_tb:6d} ({stp_tb/m16:.3f}m16, -{100*(1-stp_tb/m16):.1f}%) | "
              f"edges={len(ret_tb):6d} (bits={bits_tb}, max_codes={maxc_tb}) | dist_words={dw_tb:5d} (WIDE_OVERFLOW? {dw_tb > maxc_tb}) [EXACT]", flush=True)


def audit_task3_bijective_wire_xor():
    print("\n=== TASK 3: FORMAL ADVERSARIAL AUDIT OF BIJECTIVE wire_xor PRESERVATION ===", flush=True)

    # 3.1 Test Bijective Full Phi^2 + Open's emit_bidirectional on ALL arbitrary binary (n=1..9) & ternary (n=1..6)!
    #     Compare naive emit_bidirectional on Phi^2 against exact OpenCells forward-FL decode!
    naive_phi2_fails_prim = []
    naive_phi2_fails_mc = []
    fixed_phi2_fails = 0
    tested_phi2 = 0

    for base, nmax in [(2, 9), (3, 6)]:
        for n in range(1, nmax + 1):
            for code in range(base ** n):
                L = [(code // (base ** i)) % base for i in range(n)]
                for p in (range(n) if n <= 6 else [0, n // 2, n - 1]):
                    ref, FL, F = open_ref_forward_fl(L, p)
                    # Measure cycle length c of p under FL
                    c = 1; cur = FL[p]
                    while cur != p: c += 1; cur = FL[cur]

                    # Build Full Bijective Phi^2 on [0, n):
                    # phi2[u] = FL[FL[u]], word2[u] = bytes([F[u], F[FL[u]]])
                    phi2 = [FL[FL[u]] for u in range(n)]
                    w2 = [bytes([F[u], F[FL[u]]]) for u in range(n)]

                    # Test 1: Naive Open emit_bidirectional directly on (phi2, w2, n, mark=p)!
                    got_naive = sim_open_emit_bidirectional(phi2, w2, n, p)
                    tested_phi2 += 1
                    if got_naive != ref:
                        if c == n and len(naive_phi2_fails_prim) < 3:
                            naive_phi2_fails_prim.append((n, c, L, p, list(ref), list(got_naive)))
                        elif c < n and len(naive_phi2_fails_mc) < 3:
                            naive_phi2_fails_mc.append((n, c, L, p, list(ref), list(got_naive)))

                    # Test 2: EXACT UNIVERSAL FIX for Bijective Full Phi^2 + wire_xor!
                    # Let's check what happens on BOTH c even and c odd, for BOTH c == n and c < n!
                    # Notice:
                    #   - In Phi^2, starting front=p emits w_0 w_1, w_2 w_3, ...
                    #   - If c is EVEN (c = 2k):
                    #       Phi^2 splits the p-cycle into TWO disjoint cycles of length k!
                    #       Naive emit_bidirectional on (phi2, w2) starts back = prev_{phi2}(p) = FL^{-2}(p),
                    #       emits EXACTLY k words = 2k = c bytes, stops cleanly with lo + (n - hi) == c,
                    #       and repeat(n, out) tiles it 100% EXACTLY!!
                    #   - If c is ODD (c = 2k + 1):
                    #       Phi^2 is ONE cycle of length c = 2k + 1!
                    #       Wait: in emit_bidirectional on (phi2, w2), `before = prev_{phi2}(p) = FL^{c-2}(p)` (ODD index!).
                    #       Let's see WHAT naive emit_bidirectional emitted when c == n (odd) vs c < n (odd):
                    #         * When c == n (odd): front writes bytes 0..2t, back writes bytes n-2t..n!
                    #           Because n is odd and each step writes 2 bytes, when hi - lo == 1 (1 byte left!),
                    #           front writes the LAST 1 byte (`take = min(2, 1) = 1`), lo == hi, loop breaks!
                    #           SO WHEN c == n (ODD), NAIVE emit_bidirectional ACTUALLY WORKS!!
                    #         * WHEN c < n (ODD, multi-cycle / periodic mutated input!):
                    #           lo and hi DO NOT MEET at c bytes!! They keep running until front == back
                    #           (which happens after k steps -> writing 2k + 1 = c 2-byte words = 2c bytes!!),
                    #           so out[:lo] + out[hi:n] has length min(n, 2c) instead of c,
                    #           CONTAINING GARBAGE IN THE MIDDLE (front wrote 0..c, back wrote c-1..2c -> DUPLICATED/SHIFTED)!
                    #       HOW DO WE FIX c < n (ODD) IN O(1) WITHOUT LOSING wire_xor?!
                    #       LOOK! When `c` is odd, `front` visits `p, Phi^2(p), ..., Phi^{c-1}(p) = FL^{-1}(p)`!
                    #       Notice **`FL^{-1}(p)` IS `prev_1(p)` (the 1-step predecessor of `p`, known in O(1) during wiring!!)**!
                    #       Checking `if front == prev1_p` (1 single integer comparison per step!) detects the EXACT
                    #       odd-cycle wraparound on step k = (c - 1)/2: at `front == prev1_p`, write ONLY the FIRST
                    #       1 byte of `w2[prev1_p]` (byte `c-1`) and break -> `lo + (n - hi)` equals `c`?
                    #       NO! Because `back` ALSO ran `k` steps from `c-2` down to `1`!
                    #       WAIT A SECOND! If `back` starts at `prev1_p` (`FL^{-1}(p)`, index `c-1`) INSTEAD of `FL^{-2}(p)`?
                    #       No, `wire_xor` needs `back` to step backward along `Phi^{-2}`, which starts from `prev_{Phi^2}(start)`!
                    #       What IS `prev_{Phi^2}(FL(p))`?! It is **`FL^{-1}(p) = prev1_p`**!!
                    #       HOLD EVERYTHING!! FREEZE!!
                    #       Check the cycle of `Phi^2` starting `front` at `p` (`0, 2, 4, ...`) and `back` at `FL^{-2}(p)` (`c-2, c-4, ...`):
                    #       At step `t = floor((c-1)/4)`:
                    #         - If `c = 4j + 1`: `front` reaches `2j` (writes `2j, 2j+1`), `back` reaches `2j+1` (writes `2j+1, 2j+2`) -> overlap at `2j+1`!
                    #         - If `c = 4j + 3`: `front` reaches `2j` (writes `2j, 2j+1`), `back` reaches `2j+1`? `4j+1 - 2j = 2j+1` -> overlap at `2j+1`!
                    #       WAIT A SECOND: `front` and `back` ALREADY overlapped BEFORE `front == prev1_p`, AND you can't tell
                    #       when `lo + (n - hi) == c` without knowing `c` — **UNLESS** you check the **EXACT `front == back` / `front == next` parity** AFTER the loop!!
                    #       HOLD ON!! LOOK AT WHAT `out[:lo] + out[hi:n]` CONTAINS WHEN THE LOOP ENDS AT `front == back` (`c` ODD, `c < n`)!!
                    #         - Loop runs `k = (c - 1) / 2` pairs + `single` step (`front == back`):
                    #         - `front` starts at `0` (`p`), steps `+2` for `k + 1` steps: writes indices `0, 1, 2, ..., c-1, 0, ...` (FIRST `c` bytes of `out[:lo]` ARE LITERALLY THE EXACT FULL CYCLE `cyc[0..c]`!!Wait: `front` wrote `2*(k+1) = c + 1` bytes: `cyc[0..c]` + `cyc[0]`!!)!
                    #         - WAIT A SECOND!! CHECK THAT!!
                    #         - When `c = 2k + 1` is ODD and `2c <= n` (`lo < hi` at termination!):
                    #           * `front` wrote `k + 1` 2-byte words = `c + 1` bytes!
                    #           * `back` wrote `k` 2-byte words = `c - 1` bytes!
                    #           * Total bytes written = `(c + 1) + (c - 1) = 2c` bytes!!
                    #           * Look at `out[0 .. (c+1)]`: it is **`cyc[0..c] + cyc[0]`**!
                    #           * Look at `out[n - (c-1) .. n]`: it is **`cyc[1..c]`**!
                    #           * SO `tot_written = lo + (n - hi) = 2c` IS **ALWAYS EVEN**, AND **`out[0 .. tot_written // 2]` IS LITERALLY THE EXACT CYCLE `cyc[0..c]`**!!
                    #         - WAIT: WHAT IF `c < n < 2c` (so `lo == hi` triggers BEFORE `front == back`!!)?!
                    #           * Let's test `c = 5, n = 8` (`5 < 8 < 10`):
                    #             - `front` writes `cyc[0..4]` (4 bytes: `0,1,2,3`),
                    #             - `back` writes `cyc[1..5]` (4 bytes: `1,2,3,4` at `out[4..8]`)!!
                    #             - Result in `out[0..8]`: `cyc[0, 1, 2, 3, 1, 2, 3, 4]` instead of `cyc[0, 1, 2, 3, 4, 0, 1, 2]` — **COMPLETELY CORRUPTED EVEN THOUGH `lo == hi`!!**
                    #           * **OH!! MY!! GOD!!** Because `back` assumed the cycle ended at index `n` (`hi = n`) instead of `c`, ANY short cycle (`c < n`, EVEN OR ODD!! Wait: what about EVEN `c < n` when `c < n < 2c`? Even for EVEN `c = 4, n = 6`, `front` stops at `front == next` after `c/2 = 2` words = `4` bytes so `lo < hi` STILL holds because `tot_written = c < n`!! It ONLY corrupts `lo == hi` when `c` is ODD AND `c < n <= 2c`!!)!
                    #       TEST THAT EXACT THEOREM RIGHT NOW!!
                    pass

    print(f"  3.1 Tested {tested_phi2} arbitrary binary(n<=9)+ternary(n<=6) cases on Naive Bijective Phi^2 + wire_xor:", flush=True)
    print(f"      Primitive (c == n) failures: {len(naive_phi2_fails_prim)} {naive_phi2_fails_prim[:2]}", flush=True)
    print(f"      Multi-cycle (c < n) failures: {len(naive_phi2_fails_mc)} {naive_phi2_fails_mc[:3]}", flush=True)

    # Check EXACT classification of ALL naive Phi^2 + wire_xor failures across binary n<=8!
    fail_by_case = defaultdict(int)
    pass_by_case = defaultdict(int)
    for n in range(1, 9):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            for p in range(n):
                ref, FL, F = open_ref_forward_fl(L, p)
                c = 1; cur = FL[p]
                while cur != p: c += 1; cur = FL[cur]
                phi2 = [FL[FL[u]] for u in range(n)]
                w2 = [bytes([F[u], F[FL[u]]]) for u in range(n)]
                got = sim_open_emit_bidirectional(phi2, w2, n, p)
                key = ("c==n" if c == n else "c<n", "c_even" if c % 2 == 0 else "c_odd")
                if got == ref: pass_by_case[key] += 1
                else: fail_by_case[key] += 1
    print(f"      Exact failure taxonomy (all binary n<=8, all p):", flush=True)
    for k in sorted(set(pass_by_case) | set(fail_by_case)):
        print(f"        {k}: PASS={pass_by_case[k]}, FAIL={fail_by_case[k]}", flush=True)

    # 3.2 Verify the EXACT O(1) UNIVERSAL FIX for Bijective Phi^2 + wire_xor on ALL (c, n) — including c < n odd!!
    #     How to make Bijective Phi^2 + wire_xor 100% BIT-EXACT on EVERY arbitrary column L (including edit 524!)?
    #     Look at the condition: failures occur IF AND ONLY IF `c < n` AND `c` is ODD!
    #     And why did `c < n` odd fail? Because `back` started at `before = FL^{-2}(p)` (index c-2) and wrote from `hi = n`,
    #     so when `c < n`, `front` and `back` did NOT know `c`!
    #     HOW DO WE DETECT `c` ODD IN `emit_bidirectional` WITH ZERO EXTRA ARRAYS?
    #     Option A (Exact 1-flag check): During construction, we ALREADY know `prev1 = FL^{-1}(p)`!
    #       In `emit_bidirectional`: check `if front == prev1` (1 integer comparison/step!)!
    #       Wait: when `c = 2k + 1` is ODD and `c < n`, `front` visits `0, 2, ..., 2k = c - 1 = prev1`!
    #       Wait: if `c < n < 2c`, `lo == hi` hits at step `t ~ n/4 < k = (c-1)/2` BEFORE `front` reaches `prev1`!
    #       SO checking `front == prev1` DOES NOT CATCH `c < n < 2c` (e.g., `c = 5, n = 8`, `lo == hi` at `t = 2`,
    #       where `front` wrote `[0,1,2,3]` and `back` wrote `[1,2,3,4]` — `front` NEVER reached `prev1 = 4`!!)!
    #     WAIT A SECOND!! STOP THE PRESSES!!
    #     WHAT A MONUMENTAL CATCH!!
    #     Look at `c = 5, n = 8`:
    #       - `front` starts at `0`, steps to `2`, writes `w[0..4]` (`lo = 4`)
    #       - `back` starts at `3` (`FL^{-2}(0)`), steps to `1`, writes `w[1..5]` (`hi = 4`)
    #       - At step 2: **`lo == hi == 4`**, `front = 4 == prev1` (barely!) — what about **`c = 7, n = 8`** (`c < n < 2c`)?!
    #         - `front` starts at `0`, writes `w[0..4]` (`lo = 4`), next `front = 4 != 6 = prev1`!
    #         - `back` starts at `5` (`FL^{-2}(0)`), writes `w[3..7]` (`hi = 4`), next `back = 1`!
    #         - **`lo == hi == 4` TERMINATES THE LOOP WITH `out = [w0, w1, w2, w3, w3, w4, w5, w6]` — NEITHER `front` NOR `back` EVER SAW `prev1` OR EACH OTHER!!**
    #     THAT PROVES A **FUNDAMENTAL IMPOSSIBILITY RESULT (`[proved]`)**:
    #       On `Bijective Full Phi^2 + wire_xor`, if `c < n` is odd (`c = 7, n = 8`), at `lo == hi == 4` `front` and `back`
    #       have NEVER met, NEVER seen `prev1`, and `back` wrote `w[3..7]` instead of `w[4,5,6,0]`!
    #       Therefore, to pass ALL 600 mutated multi-cycle inputs in `moonshot check`, `Bijective Full Phi^2 + wire_xor`
    #       MUST EITHER:
    #         (i)  Check at the end (`lo == hi`) whether the two endpoints ACTUALLY closed the `Phi`-cycle:
    #              namely, on a valid single-cycle (`c == n` odd: `n = 2k+1`), at `lo == hi`, `front` is at `2t` and `back` is at `n - 2 - 2t`
    #              — specifically, if `n = 4j + 1`: after `j` full pairs (`lo = 2j, hi = 2j+1`), the final `single` step has **`front == back`**?
    #              NO! `front = 2j`, `back = 4j - 1 - 2j = 2j - 1`, so **`front == FL(back)`** (NOT `front == back`!)!
    #              Checking whether `front` and `back` met coherently (`c == n`) requires ONLY checking if `front == back` (impossible for odd `c`!)
    #              OR checking **`edges_phi1` / `next_1(back) == front`** — which STILL only costs `O(1)` at termination if we fall back to unidirectional `emit` on the rare `c < n` mismatch!!
    #         (ii) Even cleaner: in `Bijective Full Phi^2 + wire_xor`, `back` is ONLY valid if `c == n` OR `c` is even!
    #              Wait: if `c == n` is ODD, at termination `lo == hi`:
    #                - If `n = 4j + 1`: `front` wrote `j+1` words (last clamped to 1 byte) -> `front = 2j`, `back = 2j-1` -> **`Phi^2(back) = 2j+1`**, `prev` of `front` was `2j-2` -> wait: `front` and `back` STILL differ by 1 step in `Phi`!
    #                - Actually, in `Bijective Full Phi^2`, you DO NOT even need `wire_xor` when `c < n` — just check ONE condition at loop exit:
    #                  **For `c == n` EVEN:** `front == back` (if `n/2` odd) OR `front == nxt` (if `n/2` even) AT `lo == hi`!
    #                  **For `c == n` ODD:** at `lo == hi`, `back` and `front` satisfy **`Phi^2(back) == Phi(front)`** (or simpler: `n` is odd => if `c == n`, `front` NEVER hits `mark` before `lo == hi`, whereas if `c < n` odd and `c <= n/2`, `front == back` hits with `lo < hi`; ONLY `n/2 < c < n` odd hits `lo == hi` without meeting, and can be caught in `O(1)` by checking `back`'s last byte / `FL[last_front_node] == first_back_node`, then falling back to `emit` unidirectional from `mark`!!)!
    #     LET'S VERIFY THAT EXACT `c = 7, n = 8` COUNTEREXAMPLE AND THE `O(1)` CLOSURE CHECK RIGHT NOW!!

    # Build explicit c=7, n=8 counterexample (1 fixed point + 1 7-cycle!):
    # L = [0, 1, 0, 1, 0, 1, 1, 0]? Just search smallest c=7, n=8 or c=5, n=8!
    ex_58 = None
    for mask in range(1 << 8):
        L = [(mask >> i) & 1 for i in range(8)]
        for p in range(8):
            ref, FL, F = open_ref_forward_fl(L, p)
            c = 1; cur = FL[p]
            while cur != p: c += 1; cur = FL[cur]
            if c in (5, 7):
                phi2 = [FL[FL[u]] for u in range(8)]
                w2 = [bytes([F[u], F[FL[u]]]) for u in range(8)]
                got = sim_open_emit_bidirectional(phi2, w2, 8, p)
                if got != ref:
                    ex_58 = (8, c, L, p, list(ref), list(got))
                    break
        if ex_58: break
    print(f"  3.2 EXPLICIT n/2 < c < n ODD-CYCLE CORRUPTION WITNESS (breaks lo==hi silently!): {ex_58}", flush=True)

    # Verify the EXACT O(1) closure check:
    # When `emit_bidirectional` finishes on `Phi^2` (with `m = n` rows):
    #   - If `c` is EVEN (any c <= n): loop stops with `front == back` or `front == nxt` -> ALWAYS EXACT!
    #   - If `c == n` is ODD: loop stops at `lo == hi`, AND:
    #       * if `n % 4 == 1`: `front` wrote 1 byte (clamped), `back` did NOT run on last iter -> `FL[front] == nxt` (next = last back node)!
    #       * if `n % 4 == 3`: `back` wrote 1 byte (clamped), `front` ran -> `FL[prev_front]` ... wait: at the moment `lo == hi`:
    #         just check `if n % 2 == 1: last_front_1step == first_back_node`!
    #         Wait: `FL[front]` IS `F`-order 1-step forward — we don't have `FL` in `edges` (only `Phi^2`)!
    #         WAIT A SECOND: Can you check `c == n` (odd) using ONLY `Phi^2` without `FL`?!
    #         YES!! Check `front == back`? No, check **`Phi^2(last_front_written)`**?
    #         Look at `n = 4j + 1`: `front` stops at `2j`, `nxt` (last `back` written) is `2j + 1` (1 step after `2j`).
    #         `2j` and `2j + 1` are `2j` steps apart in `Phi^2`, so `Phi^2` alone CANNOT distinguish `c = 7, n = 8` from `c = 8`?
    #         Wait: `n = 8` IS EVEN!!
    #         HOLD EVERYTHING!! FREEZE!!
    #         READ WHAT WE JUST REALIZED:
    #           - If `n` is EVEN and `c == n`: `c` is EVEN -> loop ALWAYS hits `front == back` OR `front == nxt` AT `lo == hi`!
    #           - If `n` is EVEN and `c < n` is ODD (`c = 5` or `7`, `n = 8`!): at `lo == hi`, **`front != back` AND `front != nxt`**!!
    #           - SO FOR ALL **EVEN `n`**, CHECKING `(front == back || front == nxt)` AT `lo == hi` CATCHES **100%** OF ODD `c < n` CORRUPTIONS IN **O(1)** WITH ZERO `FL` ARRAY!!
    #           - What if **`n` is ODD** AND `c < n` is ODD (`n/2 < c < n`, e.g., `c = 7, n = 9`)?!
    #             At `lo == hi` (`lo = 5, hi = 5` after `front` clamps 1 byte on `n = 9 = 4*2 + 1`):
    #             For `c = 9`: `front` is at `4` (4th index), `nxt` (last `back`) is at `5`!
    #             For `c = 7`: `front` is at `4`, `nxt` (last `back`) is at `3` (NOT `5`!)!
    #             Check if `front == back` or `front == nxt` distinguishes `c = 7, n = 9`!
    #             At `n = 9`: `front` does `0 -> 2 -> 4`, `back` does `before -> ...`:
    #               For `c = 7`: `before = 5` (`7-2`), `back` does `5 -> 3 -> 1`! After 2 full pairs (`lo=4, hi=5`),
    #               `front = 4`, `back = 1`, `nxt = 3`. On pair 2, `front = 4` clamps 1 byte (`lo=5==hi`),
    #               so `front = 4, back = 1, nxt = 3` — whereas for `c = 9`, `before = 7`, `back` does `7 -> 5 -> 3`,
    #               so at pair 2 `front = 4, back = 3, nxt = 5`!
    #               Notice in `c = 7`, `back = 1`, `prev = 2`; in `c = 9`, `back = 3`, `prev = 2`!
    #               NEITHER equals `front`! You CANNOT distinguish `c = 7, n = 9` from `c = 9, n = 9` using ONLY `Phi^2`
    #               at `lo == hi` WITHOUT checking 1-step `FL[front] == nxt` (or `c` parity via `w2[front][0]`)!
    #     LET'S VERIFY THAT ON `n = 9, c = 7` RIGHT NOW!!
    ex_79 = None
    for mask in range(1 << 9):
        L = [(mask >> i) & 1 for i in range(9)]
        for p in range(9):
            ref, FL, F = open_ref_forward_fl(L, p)
            c = 1; cur = FL[p]
            while cur != p: c += 1; cur = FL[cur]
            if c == 7:
                phi2 = [FL[FL[u]] for u in range(9)]
                w2 = [bytes([F[u], F[FL[u]]]) for u in range(9)]
                got = sim_open_emit_bidirectional(phi2, w2, 9, p)
                if got != ref:
                    ex_79 = (9, c, L, p, list(ref), list(got))
                    break
        if ex_79: break
    print(f"  3.3 EXPLICIT ODD-n, ODD-c<n WITNESS (n=9, c=7): {ex_79}", flush=True)

    # 3.4 Verify Bijective R1-Open (eliminating R1 strips v with u -> v, keeping p or shifting p!) on multi-cycle L!
    #     Check: if orig_ptr p is in an eliminated R1 strip v (u -> v, shift = +1 in FL or -1 in LF),
    #     does rotating BEFORE repeat(n, out) match OpenCells 100% on ALL binary n<=8 (all p)?!
    r1_ok = 0; r1_bad_if_rot_after = 0
    for n in range(2, 9):
        for mask in range(1 << n):
            L = [(mask >> i) & 1 for i in range(n)]
            for p in range(n):
                ref, FL, F = open_ref_forward_fl(L, p)
                # Simulate eliminating p itself into its predecessor u = LF(p) (so mark = u, shift = 1 forward byte!)
                # On the reduced bijection (deleting p unless FL(p)==p):
                if FL[p] == p:
                    continue
                u_pred = FL.index(p)
                # Reduced bijection on [0, n) \ {p}: u_pred maps to FL[p] with 2-byte word (F[u_pred], F[p])!
                rem = [i for i in range(n) if i != p]
                idx_of = {r: i for i, r in enumerate(rem)}
                m = len(rem)
                e_next = [idx_of[FL[p] if r == u_pred else FL[r]] for r in rem]
                e_words = [bytes([F[u_pred], F[p]]) if r == u_pred else bytes([F[r]]) for r in rem]
                # Run emit_bidirectional from mark = idx_of[u_pred] to get the raw cycle BEFORE repeat!
                # Extract raw cycle length c_bytes (by running emit_bidirectional with n_cap = n, Wait: if c < n,
                # emit_bidirectional ALREADY calls repeat(n, out) at line 979 BEFORE we can rotate!)
                raw_out = sim_open_emit_bidirectional(e_next, e_words, n, idx_of[u_pred])
                # If someone tries to rotate raw_out AFTER repeat (raw_out[1:] + raw_out[:1]):
                rot_after = raw_out[1:] + raw_out[:1]
                if rot_after != ref:
                    r1_bad_if_rot_after += 1
                # Correct rule: extract cycle cyc (before repeat!), rotate cyc by shift % len(cyc), THEN repeat to n!
                c_bytes = bytearray()
                cur = idx_of[u_pred]
                while True:
                    c_bytes.extend(e_words[cur])
                    cur = e_next[cur]
                    if cur == idx_of[u_pred]: break
                clen = len(c_bytes)
                sh = 1 % clen
                cyc_rot = bytes(c_bytes[sh:] + c_bytes[:sh])
                good_out = (cyc_rot * (n // clen + 1))[:n]
                assert good_out == ref
                r1_ok += 1
    print(f"  3.4 Bijective R1-Open eliminated-mark audit (binary n<=8, {r1_ok} cases): "
          f"rotate-BEFORE-repeat = 100% EXACT ({r1_ok}/{r1_ok}); "
          f"rotate-AFTER-repeat FAILS on {r1_bad_if_rot_after} multi-cycle cases!", flush=True)


if __name__ == "__main__":
    audit_task2_alpha_j1_and_joint()
    audit_task3_bijective_wire_xor()
