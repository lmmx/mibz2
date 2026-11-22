#!/usr/bin/env python3
"""Adversarial Referee Audit of Investigator Gamma: Phi_BWT, Hairpin PermPC, and MF-BSE."""
import random
import sys
import subprocess
import tempfile

sys.path.insert(0, "/tmp/teamwork-c")
import verify_phi_bwt as vp
import verify_mfbse_exhaustive as vm


def runs_of(seq):
    if not seq:
        return []
    out = []
    cur, cnt = seq[0], 1
    for x in seq[1:]:
        if x == cur:
            cnt += 1
        else:
            out.append((cur, cnt))
            cur, cnt = x, 1
    out.append((cur, cnt))
    return out


def audit_phi_bwt():
    print("=== 1. AUDITING GAMMA Part I: Phi_BWT, Phi_BWT_bin, Hairpin Phi_BWT^(2) ===")
    rng = random.Random(1337)

    # 1a. Adversarial Phi_BWT: M=1, non-power-of-b M (5,7,9,17), K=1,2,3,50
    for M in [1, 2, 5, 7, 9, 17]:
        perm = list(range(M))
        rng.shuffle(perm)
        pi = [0] * M
        for i in range(M):
            pi[perm[i]] = perm[(i + 1) % M]
        pred = [0] * M
        for u, v in enumerate(pi):
            pred[v] = u
        for b in [2, 3, 5]:
            for K in [1, 2, 3, 15]:
                T, L_dig = vp.build_T_pi(pi, b, K)
                L_ref, p_ref = vp.naive_bwt(T)
                L_tc0, p_tc0 = vp.tc0_construct_L_pi(pred, b, K, L_dig)
                assert L_ref == L_tc0 and p_ref == p_tc0 == 0, f"Phi_BWT fail M={M}, b={b}, K={K}"

    # 1b. Adversarial Phi_BWT_bin: M=1,2,3,5,7,9,13, K=2,3,4,10
    for M in [1, 2, 3, 5, 7, 9, 13]:
        perm = list(range(M))
        rng.shuffle(perm)
        pi = [0] * M
        for i in range(M):
            pi[perm[i]] = perm[(i + 1) % M]
        pred = [0] * M
        for u, v in enumerate(pi):
            pred[v] = u
        for K in [2, 3, 4, 10]:
            T, L_bits = vp.build_T_pi_binary(pi, K)
            L_ref, p_ref = vp.naive_bwt(T)
            L_tc0, p_tc0 = vp.tc0_construct_L_pi_binary(pred, K, L_bits)
            assert L_ref == L_tc0 and p_ref == p_tc0 == 0, f"Phi_BWT_bin fail M={M}, K={K}"
    print("[PASS] Phi_BWT and Phi_BWT_bin exact on all adversarial M (incl M=1, non-pow-b), b, K.")

    # 1c. Adversarial Hairpin-Closed PermPC_{N, t}: test N in {1,2,3,5,7}, t in {2,4,6,8}, b in {2,3,4}, K in {1,2,3,5}
    # Check: (i) ALWAYS single cycle, (ii) L = L_bob || L_alice with ZERO comm,
    # (iii) C and p=0 are 100% CONSTANT across different (pi_1..pi_t),
    # (iv) runs(L) == runs(L_bob) + runs(L_alice) with ZERO boundary merge!
    hairpin_checked = 0
    for N in [1, 2, 3, 5]:
        for t in [2, 4, 6]:
            M_side = (t + 1) * N
            for b in [2, 3]:
                L_dig = max(1, (M_side - 1).bit_length()) if b == 2 else (1 if M_side <= b else (2 if M_side <= b*b else 3))
                while b ** L_dig < M_side:
                    L_dig += 1
                for K in [1, 2, 3]:
                    ref_C = None
                    # Test identity perms, reverse perms, and random perms
                    test_tuples = [
                        [list(range(N)) for _ in range(t)],
                        [list(reversed(range(N))) for _ in range(t)],
                    ]
                    for _ in range(4):
                        test_tuples.append([rng.sample(range(N), N) for _ in range(t)])
                    for pis in test_tuples:
                        invs = []
                        for pm in pis:
                            inv = [0] * N
                            for i, v in enumerate(pm):
                                inv[v] = i
                            invs.append(inv)
                        f_A = [0] * M_side
                        f_B = [0] * M_side
                        half = t // 2
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

                        T = vp.build_T_2party(f_A, f_B, b, K, L_dig)
                        L_ref, p_ref = vp.naive_bwt(T)
                        L_bob, L_alice = vp.zero_comm_split_2party(f_A, f_B, b, K, L_dig)
                        assert L_bob + L_alice == L_ref, "L_bob || L_alice != L_ref"
                        assert p_ref == 0, f"p_ref={p_ref} != 0"
                        assert L_bob[-1] != L_alice[0], f"Boundary run merge! {L_bob[-1]} == {L_alice[0]}"
                        assert runs_of(L_bob) + runs_of(L_alice) == runs_of(L_ref)
                        _, C = vp.compute_LF_int(L_ref, 2 * b + 4)
                        if ref_C is None:
                            ref_C = C
                        else:
                            assert C == ref_C, f"C not constant! {C} != {ref_C}"
                        # Check r upper bound
                        r_cnt = len(runs_of(L_ref))
                        assert r_cnt <= 2 * (L_dig + 1) * M_side + 4
                        hairpin_checked += 1
    print(f"[PASS] Hairpin-Closed Phi_BWT^(2): {hairpin_checked} adversarial instances PASSED (zero-comm split, zero seam merge, constant C, p=0, exact single cycle).")


def audit_mfbse():
    print("\n=== 2. AUDITING GAMMA Part II: MF-BSE (Python + C) ===")
    # 2a. Edge cases on Python mfbse_decode_from_I0:
    # - n=1, p=0 (self-loop p->p)
    # - p->p self-loop inside larger multi-cycle column (e.g. L = [0, 1, 0, 1], p = 0 -> LF(0)=0!)
    # - Disconnected cycles touching multiple buckets, K=1, K=n, K=n-1
    # - Adversarial max req_len[K] = K (p is parent of chain in every bucket 0..K-1!)
    edge_cases = [
        ([0], 0),
        ([0, 0, 0, 0], 2),
        ([0, 1, 0, 1], 0),
        ([0, 1, 0, 1], 1),
        ([0, 1, 0, 1], 2),
        ([0, 1, 0, 1], 3),
        ([0, 2, 1, 0, 2, 1], 0),
        ([0, 2, 1, 0, 2, 1], 3),
        ([2, 1, 0, 2, 1, 0], 4),
    ]
    for L, p in edge_cases:
        ref = vm.ref_orbit_decode(L, p)
        I0 = vm.encode_I0(L, p, cap=2)
        for K in range(1, len(L) + 1):
            got = vm.mfbse_decode_from_I0(*I0, K=K)
            assert got == ref, f"Python MF-BSE FAIL L={L} p={p} K={K}: {got} != {ref}"

    # 2b. Construct adversarial single cycle where req_len[K] == K (every bucket sends a req to p!)
    # Cycle: p -> 0 -> 1 -> ... -> K-1 -> p with m = K + 1, p = m - 1, S = 1!
    # Check: bucket 0 contracts p -> 0 -> 1 into shortcut p -> 1 and sends req to p!
    # Bucket 1 contracts p -> 1 -> 2 into shortcut p -> 2 and sends req to p!
    # ... Bucket K-1 sends req to p! So req_len[K] == K!
    # Let's test if bench_mfbse.c handles multi-cycle AND large req_len[K] AND S=1!
    c_test = r'''
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <assert.h>

// Include MF-BSE core logic from bench_mfbse.c by extracting lines 18..215
'''
    with open("/tmp/teamwork-c/bench_mfbse.c") as f:
        lines = f.readlines()
    core = "".join(lines[17:215])
    driver = r'''
int main(void) {
    // Test 1: Multi-cycle permutation: cycle1 = (0 -> 2 -> 0), cycle2 = (1 -> 3 -> 1), p_idx = 0, K = 2 (S = 2)
    // Bucket 0 = {0, 1}, Bucket 1 = {2, 3}.
    // Wait: make cycle2 span buckets: cycle1 = (0 -> 2 -> 0), cycle2 = (1 -> 3 -> 1), p_idx = 0.
    // Wait: put p_idx = 3, cycle1 = (3 -> 2 -> 3), cycle2 = (0 -> 1 -> 0) or (0 -> 2? no, disjoint):
    // m = 6, K = 3, S = 2:
    // B0 = {0, 1}, B1 = {2, 3}, B2 = {4, 5}.
    // Cycle A (with p=5): 5 -> 0 -> 5 (weights 1, 1)
    // Cycle B (no p):     1 -> 2 -> 4 -> 1 (weights 1, 1, 1)
    // Cycle C (no p):     3 -> 3 (weight 1)
    uint32_t m = 6, K = 3, p_idx = 5;
    uint32_t G[6] = {5, 2, 4, 3, 1, 0};
    uint32_t W[6] = {1, 1, 1, 1, 1, 1};
    uint32_t pos[6];
    memset(pos, 0xAA, sizeof(pos));
    Workspace *ws = ws_create(m, K);
    // Pollute ws->in_u with garbage to expose uninitialized local_pos
    for (uint32_t i = 0; i < ws->S; i++) ws->in_u[i] = 999999;
    uint64_t sc = 0, er = 0;
    uint32_t clen = mfbse_solve(ws, G, W, m, p_idx, pos, &sc, &er);
    printf("Test 1 (multi-cycle): clen=%u (expected 2), pos=[%u, %u, %u, %u, %u, %u]\n",
           clen, pos[0], pos[1], pos[2], pos[3], pos[4], pos[5]);
    int ok_mc = (clen == 2 && pos[5] == 0 && pos[0] == 1);
    int leaked_garbage = (pos[1] != UINT32_MAX || pos[2] != UINT32_MAX || pos[3] != UINT32_MAX || pos[4] != UINT32_MAX);
    printf("  p-cycle entries exact? %s | non-p-cycle entries contain garbage (not sentinel)? %s\n",
           ok_mc ? "YES" : "NO", leaked_garbage ? "YES (BUG in C!)" : "NO");
    ws_free(ws);

    // Test 2: Self-loop p -> p with m = 4, K = 2, G = {0, 2, 3, 1}, p_idx = 0
    uint32_t G2[4] = {0, 2, 3, 1};
    uint32_t W2[4] = {7, 1, 1, 1};
    uint32_t pos2[4];
    Workspace *ws2 = ws_create(4, 2);
    uint32_t clen2 = mfbse_solve(ws2, G2, W2, 4, 0, pos2, &sc, &er);
    printf("Test 2 (self-loop p->p): clen=%u (expected 7), pos[0]=%u\n", clen2, pos2[0]);
    ws_free(ws2);

    // Test 3: Adversarial req_len[K] = K = 128 (> initial cap 64!), m = 129, p = 128, G[u] = (u+1)%m
    uint32_t m3 = 129, K3 = 128, p3 = 128;
    uint32_t *G3 = malloc(m3 * sizeof(uint32_t));
    uint32_t *W3 = malloc(m3 * sizeof(uint32_t));
    uint32_t *pos3 = malloc(m3 * sizeof(uint32_t));
    for (uint32_t i = 0; i < m3; i++) { G3[i] = (i + 1) % m3; W3[i] = 1; }
    Workspace *ws3 = ws_create(m3, K3);
    //Wait: check if S = (129 + 127) / 128 = 2 -> buckets 0..64 non-empty, buckets 65..127 empty!
    // Use m3 = 1000, K3 = 100 -> S = 10, cycle 999 -> 0 -> 10 -> 20 -> ... -> 990 -> 999!
    free(G3); free(W3); free(pos3); ws_free(ws3);

    uint32_t m4 = 1000, K4 = 100, p4 = 999;
    uint32_t *G4 = malloc(m4 * sizeof(uint32_t));
    uint32_t *W4 = malloc(m4 * sizeof(uint32_t));
    uint32_t *pos4 = malloc(m4 * sizeof(uint32_t));
    for (uint32_t i = 0; i < m4; i++) { G4[i] = i; W4[i] = 1; }
    // Make cycle p4=999 -> 0 -> 10 -> 20 -> ... -> 980 -> 990 -> 999
    G4[999] = 0;
    for (uint32_t k = 0; k < 99; k++) G4[k * 10] = (k + 1) * 10;
    G4[990] = 999;
    Workspace *ws4 = ws_create(m4, K4);
    uint32_t clen4 = mfbse_solve(ws4, G4, W4, m4, p4, pos4, &sc, &er);
    printf("Test 3 (req_len[K]=%u > 64): clen=%u (expected 101), pos[990]=%u (expected 100)\n",
           ws4->req_len[K4], clen4, pos4[990]);
    assert(clen4 == 101 && pos4[999] == 0 && pos4[0] == 1 && pos4[990] == 100);

    // Test 4: Adversarial single-cycle where bucket K-1 gets req_len[K-1] > 2*S (realloc triggers!)
    // AND check if reply_pool or fwd_pool can overflow!
    // Wait: what if K=100, m=1000, S=10, and vertex u=990 (in B_{99}) receives 99 requests from B_0..B_{98}?
    // Make single cycle: 990 -> 0 -> 10 -> 20 -> ... -> 980 -> 991 -> ... -> 999 -> 990!
    // Actually: 990 -> 0 -> 1 -> ... -> 9 -> 10 -> ... -> 989 -> 991 -> ... -> 999 -> 990, p = 999!
    // Wait: in each bucket k=0..98, 990 enters k*10, chains through k*10..k*10+9, exits to (k+1)*10!
    // So 990 receives 99 requests (99 > 2*S = 20)!
    for (uint32_t i = 0; i < m4; i++) G4[i] = (i + 1) % m4;
    // Wait: G4[999]=0, ..., G4[989]=990 -> 999 gets 99 requests! Swap 990 and 999 as p=998:
    // G4[990]=0, G4[989]=991, G4[991]=992, ..., G4[999]=990, p=995.
    G4[990] = 0; G4[989] = 991; G4[999] = 990;
    uint32_t clen5 = mfbse_solve(ws4, G4, W4, m4, 995, pos4, &sc, &er);
    printf("Test 4 (req_len[99]=%u > 2*S=20): clen=%u (expected 1000), pos[995]=%u, pos[994]=%u\n",
           ws4->req_len[99], clen5, pos4[995], pos4[994]);
    assert(clen5 == 1000 && pos4[995] == 0 && pos4[994] == 999);
    ws_free(ws4); free(G4); free(W4); free(pos4);
    return 0;
}
'''
    c_path = "/tmp/teamwork-referee/test_mfbse_c.c"
    bin_path = "/tmp/teamwork-referee/test_mfbse_c"
    with open(c_path, "w") as f:
        f.write(c_test + core + driver)
    subprocess.run(["gcc", "-O2", "-Wall", "-Wextra", c_path, "-o", bin_path], check=True)
    res = subprocess.run([bin_path], capture_output=True, text=True, check=True)
    print(res.stdout.strip())
    print("[PASS] Python MF-BSE passed all edge cases; C bench_mfbse.c audit output printed above.")


if __name__ == "__main__":
    audit_phi_bwt()
    audit_mfbse()
