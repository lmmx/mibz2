#!/tmp/solver-venv/bin/python
"""
ROUND 3 PRIORITY 3: Bounded-Model Exact Communication Complexity Verification of Gamma's Hairpin-Closed Phi_BWT^(2).

EXPLICIT BOUNDED COMPUTATIONAL MODEL & QUANTIFIED ALGORITHM CLASS:
- Class: All exact deterministic (2p - 1)-round two-party communication protocols (Bob speaks first,
  messages M_1: B->A, M_2: A->B, ..., M_{2p-1}: B->A, each of <= S bits; Alice outputs target
  f(x, y) with ZERO error). This strictly subsumes all exact deterministic p-pass READ-ONLY
  streaming decoders with S-bit working RAM reading I_0 = I_{0,Bob} || I_{0,Alice} sequentially.
- IMPORTANT SCOPE LIMIT: Finite-instance communication/SAT checks verify this bounded read-only
  protocol model on small (N, t, p) and do NOT constitute a general RAM dependent-load lower bound,
  nor do they apply to read-write streaming / external-memory decoders (BDMR, Lazard-BWT, MF-BSE).

Verifies on Hairpin-Closed Phi_BWT^(2) for:
  1. (N=2, t=2, p=1), (N=3, t=2, p=1), (N=4, t=2, p=1)  [1-round Bob->Alice exact min S, full v_t & 1-bit predicate]
  2. (N=2, t=4, p=1) and (N=2, t=4, p=2)                [1-round & 3-round protocol SAT synthesis + DRAT UNSAT!]
  3. (N=3, t=4, p=1)                                    [1-round exact min S = 36x36 matrix]
  4. Verifies Gamma's zero-communication split (L = L_bob || L_alice, zero seam merge, constant C, p=0)
     directly on the solver instances.
"""
import itertools
import json
import math
import os
import sys
import time

from pysat.solvers import Cadical195

sys.path.insert(0, "/tmp/teamwork-c")
import verify_phi_bwt as vp

OUT_DIR = "/tmp/teamwork-referee/solver_audit"
os.makedirs(OUT_DIR, exist_ok=True)


def build_hairpin_instance(N, t, pis):
    """Build Hairpin-Closed Phi_BWT^(2) maps (f_A, f_B) and compute target v_t = pi_t(...pi_1(0)...)."""
    assert t % 2 == 0 and len(pis) == t
    M_side = (t + 1) * N
    half = t // 2
    invs = []
    for pm in pis:
        inv = [0] * N
        for i, v in enumerate(pm):
            inv[v] = i
        invs.append(inv)
    f_A = [0] * M_side
    f_B = [0] * M_side
    for k in range(half):
        for x in range(N):
            f_A[k * N + x] = k * N + pis[2 * k][x]
    for x in range(N):
        f_A[half * N + x] = half * N + x
    for k in range(half):
        li = t - 2 - 2 * k
        for x in range(N):
            f_A[(half + 1 + k) * N + x] = (half + 1 + k) * N + invs[li][x]
    for k in range(half):
        for x in range(N):
            f_B[k * N + x] = (k + 1) * N + pis[2 * k + 1][x]
    for k in range(half):
        li = t - 1 - 2 * k
        for x in range(N):
            f_B[(half + k) * N + x] = (half + 1 + k) * N + invs[li][x]
    for x in range(N):
        f_B[t * N + x] = (x + 1) % N

    v = 0
    for pm in pis:
        v = pm[v]
    return f_A, f_B, v


def verify_zero_comm_split_sample(N, t, b=2, K=2, max_samples=16):
    """Verify exact Hairpin-Closed BWT zero-comm split on sample (pi_1..pi_t) tuples."""
    perms = list(itertools.permutations(range(N)))
    M_side = (t + 1) * N
    L_dig = max(1, math.ceil(math.log(M_side, b)))
    while b ** L_dig < M_side:
        L_dig += 1
    ref_C = None
    checked = 0
    for tup in itertools.product(perms, repeat=t):
        f_A, f_B, v_t = build_hairpin_instance(N, t, tup)
        T = vp.build_T_2party(f_A, f_B, b, K, L_dig)
        L_ref, p_ref = vp.naive_bwt(T)
        L_bob, L_alice = vp.zero_comm_split_2party(f_A, f_B, b, K, L_dig)
        assert L_bob + L_alice == L_ref and p_ref == 0
        assert L_bob[-1] != L_alice[0]
        _, C = vp.compute_LF_int(L_ref, 2 * b + 4)
        if ref_C is None:
            ref_C = C
        else:
            assert C == ref_C
        # Verify v_t sits at public offset in T
        off = (t // 2) * 2 * (K + L_dig) + K
        DIG_A = b + 4
        got_digits = [sym - DIG_A for sym in T[off:off + L_dig]]
        assert got_digits == vp.digits_base_b((t // 2) * N + v_t, b, L_dig)
        checked += 1
        if checked >= max_samples:
            break
    return checked, M_side, L_dig


def build_comm_matrix(N, t, predicate="full"):
    """
    Build exact 2-party communication matrix M[idx_A][idx_B]:
      Alice owns (pi_1, pi_3, ..., pi_{t-1})  [tuple of t/2 permutations in S_N]
      Bob   owns (pi_2, pi_4, ..., pi_t)      [tuple of t/2 permutations in S_N]
      M[a][b] = v_t (if predicate == 'full') or int(v_t == 0) (if predicate == 'bit0').
    """
    perms = list(itertools.permutations(range(N)))
    half = t // 2
    A_inputs = list(itertools.product(perms, repeat=half))
    B_inputs = list(itertools.product(perms, repeat=half))
    M = [[0] * len(B_inputs) for _ in range(len(A_inputs))]
    for ia, a_tup in enumerate(A_inputs):
        for ib, b_tup in enumerate(B_inputs):
            v = 0
            for k in range(half):
                v = a_tup[k][v]
                v = b_tup[k][v]
            M[ia][ib] = v if predicate == "full" else (1 if v == 0 else 0)
    return A_inputs, B_inputs, M


def exact_1round_bob_to_alice(M, label, save_drat_tag=None):
    """
    Exact 1-round deterministic Bob -> Alice communication complexity:
    Bob maps each column b in B to a message m(b) in {0..K-1} (K = 2^S).
    Alice outputs f(a, b) given (a, m(b)) with ZERO error iff whenever m(b1) == m(b2),
    columns b1 and b2 are IDENTICAL!
    Thus exact minimum #messages K_min == #{distinct columns of M},
    and exact minimum bits S_min == ceil(log2(K_min))!
    We ALSO encode K_min - 1 (and 2^(S_min - 1)) in CaDiCaL 1.9.5 with DRAT proof!
    """
    nA, nB = len(M), len(M[0])
    cols = [tuple(M[a][b] for a in range(nA)) for b in range(nB)]
    uniq_cols = sorted(set(cols))
    K_min = len(uniq_cols)
    S_min = math.ceil(math.log2(K_min)) if K_min > 1 else 0

    # Verify via SAT + DRAT that K = 2^(S_min - 1) messages is UNSAT (for K_min <= 8, avoiding exponential CDCL PHP!)
    drat_lines = 0
    if S_min >= 1 and K_min <= 8:
        K_test = 1 << (S_min - 1)
        nvars = nB * K_test
        def vid(b, c): return b * K_test + c + 1
        clauses = []
        for b in range(nB):
            clauses.append([vid(b, c) for c in range(K_test)])
            for c1 in range(K_test):
                for c2 in range(c1 + 1, K_test):
                    clauses.append([-vid(b, c1), -vid(b, c2)])
        for b1 in range(nB):
            for b2 in range(b1 + 1, nB):
                if cols[b1] != cols[b2]:
                    for c in range(K_test):
                        clauses.append([-vid(b1, c), -vid(b2, c)])
        want_proof = save_drat_tag is not None
        with Cadical195(bootstrap_with=clauses, with_proof=want_proof) as s:
            sat = s.solve()
            assert not sat, "Unexpected SAT for 1-round S_min - 1!"
            if want_proof:
                pr = s.get_proof()
                drat_lines = len(pr)
                with open(os.path.join(OUT_DIR, f"{save_drat_tag}.cnf"), "w") as cf:
                    cf.write(f"p cnf {nvars} {len(clauses)}\n")
                    for cl in clauses:
                        cf.write(" ".join(map(str, cl)) + " 0\n")
                with open(os.path.join(OUT_DIR, f"{save_drat_tag}.drat"), "w") as pf:
                    for ln in pr:
                        pf.write(ln + "\n")
    proof_note = f", DRAT={drat_lines} lines" if drat_lines else f", exact PHP({K_min} > {1<<(S_min-1)}) [proved]"
    print(f"  [1-round B->A] {label:32s}: |A|={nA:2d}, |B|={nB:2d}, distinct_cols={K_min:2d} -> "
          f"exact S_min={S_min} bits (S={S_min-1} impossible{proof_note})", flush=True)
    return {"label": label, "nA": nA, "nB": nB, "K_min": K_min, "S_min": S_min, "drat_lines": drat_lines}


def exact_3round_protocol_sat(M, K1, K2, K3, label, save_drat_tag=None, timeout_s=8.0):
    """
    Exact deterministic 3-round protocol synthesis (2-pass read-only streaming over L_bob || L_alice):
      Round 1 (Bob -> Alice):  m1(b) in 0..K1-1
      Round 2 (Alice -> Bob):  m2(a, m1) in 0..K2-1
      Round 3 (Bob -> Alice):  m3(b, m2) in 0..K3-1  (Bob already knows m1 from b!)
      Output (Alice):          out(a, m1, m3) == M[a][b] for ALL (a, b)!
    Row/column deduplication is exact (identical rows/cols are information-theoretically indistinguishable).
    """
    import threading
    t0 = time.time()
    M_row_dedup = sorted(set(tuple(row) for row in M))
    nA0, nB0 = len(M_row_dedup), len(M_row_dedup[0])
    cols = sorted(set(tuple(M_row_dedup[a][b] for a in range(nA0)) for b in range(nB0)))
    nA, nB = nA0, len(cols)
    Mr = [[cols[b][a] for b in range(nB)] for a in range(nA)]

    nvars = 0
    def nv():
        nonlocal nvars
        nvars += 1
        return nvars
    clauses = []
    def one_hot(vs):
        clauses.append(list(vs))
        for i in range(len(vs)):
            for j in range(i + 1, len(vs)):
                clauses.append([-vs[i], -vs[j]])

    m1 = [[nv() for _ in range(K1)] for _ in range(nB)]
    for b in range(nB):
        one_hot(m1[b])
    m2 = [[[nv() for _ in range(K2)] for _ in range(K1)] for _ in range(nA)]
    for a in range(nA):
        for c1 in range(K1):
            one_hot(m2[a][c1])
    m3 = [[[nv() for _ in range(K3)] for _ in range(K2)] for _ in range(nB)]
    for b in range(nB):
        for c2 in range(K2):
            one_hot(m3[b][c2])

    clauses.append([m1[0][0]])
    for c1 in range(K1):
        clauses.append([m2[0][c1][0]])
    for c2 in range(K2):
        clauses.append([m3[0][c2][0]])

    for b1 in range(nB):
        for b2 in range(b1 + 1, nB):
            diff_a = [a for a in range(nA) if Mr[a][b1] != Mr[a][b2]]
            if not diff_a:
                continue
            for c1 in range(K1):
                s1 = nv()
                clauses.append([-m1[b1][c1], -m1[b2][c1], s1])
                for c2 in range(K2):
                    for c3 in range(K3):
                        h = nv()
                        clauses.append([-s1, -m3[b1][c2][c3], -m3[b2][c2][c3], h])
                        for a in diff_a:
                            clauses.append([-h, -m2[a][c1][c2]])

    want_proof = save_drat_tag is not None
    with Cadical195(bootstrap_with=clauses, with_proof=want_proof) as s:
        timer = threading.Timer(timeout_s, s.interrupt)
        timer.start()
        try:
            sat = s.solve_limited(expect_interrupt=True)
        finally:
            timer.cancel()
        el = time.time() - t0
        if sat is None:
            print(f"  [3-round B->A->B->A] {label:28s} (red {nA}x{nB}, K=({K1},{K2},{K3})): TIMEOUT ({el:.2f}s)", flush=True)
            return None
        if sat:
            mod = set(s.get_model())
            f1 = [next(c for c in range(K1) if m1[b][c] in mod) for b in range(nB)]
            f2 = [[next(c for c in range(K2) if m2[a][c1][c] in mod) for c1 in range(K1)] for a in range(nA)]
            f3 = [[next(c for c in range(K3) if m3[b][c2][c] in mod) for c2 in range(K2)] for b in range(nB)]
            table = {}
            for a in range(nA):
                for b in range(nB):
                    c1 = f1[b]
                    c2 = f2[a][c1]
                    c3 = f3[b][c2]
                    key = (a, c1, c2, c3)
                    if key in table:
                        assert table[key] == Mr[a][b]
                    else:
                        table[key] = Mr[a][b]
            print(f"  [3-round B->A->B->A] {label:28s} (red {nA}x{nB}, K=({K1},{K2},{K3})): SAT (exact protocol verified! {el:.3f}s)", flush=True)
            return True
        else:
            drat_lines = 0
            if want_proof:
                pr = s.get_proof()
                drat_lines = len(pr)
                with open(os.path.join(OUT_DIR, f"{save_drat_tag}.cnf"), "w") as cf:
                    cf.write(f"p cnf {nvars} {len(clauses)}\n")
                    for cl in clauses:
                        cf.write(" ".join(map(str, cl)) + " 0\n")
                with open(os.path.join(OUT_DIR, f"{save_drat_tag}.drat"), "w") as pf:
                    for ln in pr:
                        pf.write(ln + "\n")
            print(f"  [3-round B->A->B->A] {label:28s} (red {nA}x{nB}, K=({K1},{K2},{K3})): UNSAT ({el:.3f}s)"
                  + (f" [DRAT: {drat_lines} lines saved]" if drat_lines else ""), flush=True)
            return False


def main():
    print("=== PRIORITY 3: BOUNDED-MODEL EXACT COMMUNICATION COMPLEXITY ON HAIRPIN-CLOSED Phi_BWT^(2) ===", flush=True)
    print("Model: Exact deterministic (2p-1)-round 2-party comm (subsumes p-pass READ-ONLY streaming, 0 error).\n", flush=True)

    print("1. Verifying Gamma's Zero-Communication Split on Solver Instances:", flush=True)
    for (N, t) in [(2, 2), (3, 2), (2, 4), (3, 4)]:
        chk, mside, ldig = verify_zero_comm_split_sample(N, t, b=2, K=2, max_samples=12)
        print(f"  Hairpin(N={N}, t={t}): {chk} instances verified (M_side={mside}, L_dig={ldig}, L=L_bob||L_alice, 0 seam merge, const C, p=0) [verified]", flush=True)

    print("\n2. Exact 1-Round (p=1 pass) Communication Complexity (Bob -> Alice):", flush=True)
    res_1r = []
    for (N, t, pred, tag) in [
        (2, 2, "full", "p3_1r_N2_t2_full"),
        (3, 2, "full", "p3_1r_N3_t2_full"),
        (3, 2, "bit0", "p3_1r_N3_t2_bit0"),
        (4, 2, "full", None),
        (4, 2, "bit0", "p3_1r_N4_t2_bit0"),
        (2, 4, "full", "p3_1r_N2_t4_full"),
        (3, 4, "full", None),
        (3, 4, "bit0", None),
    ]:
        _, _, M = build_comm_matrix(N, t, predicate=pred)
        r = exact_1round_bob_to_alice(M, f"N={N}, t={t}, pred={pred}", save_drat_tag=tag)
        res_1r.append(r)

    print("\n3. Exact 3-Round (p=2 passes: B->A->B->A) Protocol Synthesis via CaDiCaL 1.9.5:", flush=True)
    # N=2, t=4: S_2 is Abelian (Z_2), so pi_4 o pi_3 o pi_2 o pi_1 is XOR -> 1 bit in 1 round!
    _, _, M_2_4 = build_comm_matrix(2, 4, predicate="full")
    exact_3round_protocol_sat(M_2_4, 2, 1, 1, "N=2, t=4, full (S_2 abelian)")

    # N=3, t=2 (non-Abelian S_3!): 1-round needs S=3 bits (6 cols) full / S=2 bits bit0;
    # In 3 rounds, check S=1 bit/msg K=(2,2,2) vs K=(1,3,3) / K=(1,3,2)!
    _, _, M_3_2 = build_comm_matrix(3, 2, predicate="full")
    exact_3round_protocol_sat(M_3_2, 2, 2, 2, "N=3, t=2, full S=1bit/msg", save_drat_tag="p3_3r_N3_t2_full_1b")
    exact_3round_protocol_sat(M_3_2, 1, 3, 3, "N=3, t=2, full K=(1,3,3)")

    _, _, M_3_2_b0 = build_comm_matrix(3, 2, predicate="bit0")
    exact_3round_protocol_sat(M_3_2_b0, 2, 2, 2, "N=3, t=2, bit0 S=1bit/msg", save_drat_tag="p3_3r_N3_t2_bit0_1b")
    exact_3round_protocol_sat(M_3_2_b0, 1, 3, 2, "N=3, t=2, bit0 K=(1,3,2)")

    # N=3, t=4, p=2 (3 rounds B->A->B->A, t=2p=4!):
    _, _, M_3_4_b0 = build_comm_matrix(3, 4, predicate="bit0")
    exact_3round_protocol_sat(M_3_4_b0, 2, 2, 2, "N=3, t=4, bit0 S=1bit/msg", save_drat_tag="p3_3r_N3_t4_bit0_1b", timeout_s=8.0)

    _, _, M_3_4_full = build_comm_matrix(3, 4, predicate="full")
    exact_3round_protocol_sat(M_3_4_full, 2, 2, 2, "N=3, t=4, full S=1bit/msg", save_drat_tag="p3_3r_N3_t4_full_1b", timeout_s=8.0)
    exact_3round_protocol_sat(M_3_4_full, 1, 3, 6, "N=3, t=4, full K=(1,3,6)", timeout_s=8.0)

    with open(os.path.join(OUT_DIR, "p3_comm_results.json"), "w") as f:
        json.dump(res_1r, f, indent=2)


if __name__ == "__main__":
    main()
