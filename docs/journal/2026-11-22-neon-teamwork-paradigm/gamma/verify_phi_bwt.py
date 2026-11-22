#!/usr/bin/env python3
"""
Exact verification of the BWT Permutation-Universality Constructions:
1. Phi_BWT(pi, b, K): single-cycle permutation embedding over alphabet size b+2,
   with TC^0 / O(L)-pass zero-chase acquisition of I0(T_pi).
2. Phi_BWT_bin(pi, K): binary alphabet (|Sigma| = 2) embedding.
3. Phi_BWT_2party(f_A, f_B, b, K): two-party contiguous-split embedding where
   L[0..n/2) depends ONLY on f_B and L[n/2..n) depends ONLY on f_A (zero communication!).
"""
import itertools
import math
import random


def naive_bwt(seq):
    """Exact BWT (L, p) by sorting all rotations of sequence of ints."""
    n = len(seq)
    rots = [(seq[i:] + seq[:i], i) for i in range(n)]
    rots.sort(key=lambda x: x[0])
    L = [rot[-1] for rot, _ in rots]
    p = next(idx for idx, (_, start) in enumerate(rots) if start == 0)
    return L, p


def compute_LF_int(L, sigma):
    cnt = [0] * sigma
    for x in L:
        cnt[x] += 1
    C = [0] * sigma
    s = 0
    for c in range(sigma):
        C[c] = s
        s += cnt[c]
    cur = C[:]
    LF = [0] * len(L)
    for i, x in enumerate(L):
        LF[i] = cur[x]
        cur[x] += 1
    return LF, C


def digits_base_b(x, b, L_dig):
    d = [0] * L_dig
    for i in range(L_dig - 1, -1, -1):
        d[i] = x % b
        x //= b
    return d


def build_T_pi(pi, b, K, L_dig=None):
    """Alphabet: # = 0, $ = 1, digits 0..b-1 encoded as 2..b+1."""
    M = len(pi)
    if L_dig is None:
        L_dig = max(1, math.ceil(math.log(M, b))) if M > 1 else 1
    T = []
    u = 0
    seen = set()
    for _ in range(M):
        assert u not in seen, "pi must be a single cycle"
        seen.add(u)
        T.append(0)  # '#'
        for _ in range(K - 1):
            T.append(1)  # '$'
        for dig in digits_base_b(u, b, L_dig):
            T.append(2 + dig)
        u = pi[u]
    assert u == 0 and len(seen) == M
    return T, L_dig


def tc0_construct_L_pi(pred, b, K, L_dig):
    """Zero-chase TC^0 / O(L)-pass direct construction of BWT column L from pred = pi^{-1}."""
    M = len(pred)
    # 1. '#' block (M rows): row x has L = d_L(pred[x])
    L_hash = [2 + digits_base_b(pred[x], b, L_dig)[-1] for x in range(M)]
    # 2. '$' blocks ((K-1)*M rows):
    #    r = K-1: M copies of '#' (0)
    #    r = K-2..1: (K-2)*M copies of '$' (1)
    L_dollar = []
    if K >= 2:
        L_dollar.extend([0] * M)
        if K >= 3:
            L_dollar.extend([1] * ((K - 2) * M))
    # 3. Digit blocks (L_dig * M rows):
    #    Pairs (s, x) for s in 1..L_dig, x in 0..M-1, y = pred[x]
    #    key(s, x) = (d_{L-s+1}(y), ..., d_L(y), 0='#', x)
    #    L-char = d_{L-s}(y) if s < L_dig else ('$' if K >= 2 else '#')
    items = []
    for x in range(M):
        y = pred[x]
        digs = digits_base_b(y, b, L_dig)
        for s in range(1, L_dig + 1):
            # suffix of length s, followed by sentinel -1 (representing '#' < digits 0..b-1)
            key = tuple(digs[L_dig - s :]) + (-1, x)
            l_char = (2 + digs[L_dig - s - 1]) if s < L_dig else (1 if K >= 2 else 0)
            items.append((key, l_char))
    items.sort(key=lambda t: t[0])
    L_digits = [l_char for _, l_char in items]
    return L_hash + L_dollar + L_digits, 0


def build_T_pi_binary(pi, K):
    """Binary alphabet {0, 1} embedding:
    S = 0^K, id_2(x) = 1 b_1 1 b_2 ... 1 b_L 1 (length 2L+1, never contains '00').
    Requires K >= 2 so 0^K is strictly unique and smallest."""
    M = len(pi)
    L_bits = max(1, math.ceil(math.log2(M))) if M > 1 else 1
    assert K >= 2
    T = []
    u = 0
    seen = set()
    for _ in range(M):
        assert u not in seen
        seen.add(u)
        T.extend([0] * K)
        bits = digits_base_b(u, 2, L_bits)
        for bit in bits:
            T.append(1)
            T.append(bit)
        T.append(1)
        u = pi[u]
    assert u == 0
    return T, L_bits


def tc0_construct_L_pi_binary(pred, K, L_bits):
    """Zero-chase direct construction of binary BWT column L from pred = pi^{-1}."""
    M = len(pred)
    id_len = 2 * L_bits + 1

    def encode_id(x):
        out = []
        for bit in digits_base_b(x, 2, L_bits):
            out.append(1)
            out.append(bit)
        out.append(1)
        return out

    # Every rotation starts at offset o in [0, K + id_len) inside some block S id(x).
    # Because S = 0^K and id never contains '00':
    # The next K + id_len + K symbols uniquely determine lexicographic order
    # using ONLY (x, pred[x])! (For offsets in S: depends on x; for offsets in id(y): depends on (y, pi(y)) = (pred[x], x)!)
    items = []
    for x in range(M):
        y = pred[x]
        id_x = encode_id(x)
        id_y = encode_id(y)
        # 1. Offsets inside S before id(x): o = 0..K-1
        #    Suffix is 0^{K-o} id_x 0^K ... (unique within K + id_len chars!)
        for o in range(K):
            key = tuple([0] * (K - o) + id_x)
            l_char = 1 if o == 0 else 0  # id_y ends with 1!
            items.append((key, l_char))
        # 2. Offsets inside id(y) before S id(x): o = 0..id_len-1
        #    Suffix is id_y[o:] 0^K id_x ... (unique within id_len + K + id_len chars!)
        for o in range(id_len):
            key = tuple(id_y[o:] + [0] * K + id_x)
            l_char = 0 if o == 0 else id_y[o - 1]
            items.append((key, l_char))
    items.sort(key=lambda t: t[0])
    L = [l_char for _, l_char in items]
    return L, 0


def build_T_2party(f_A, f_B, b, K, L_dig):
    """Two-party contiguous-split BWT:
    Alphabet (size 2b+4):
      0 = #_A, 1 = $_A, 2..b+1 = 0_B..(b-1)_B,
      b+2 = #_B, b+3 = $_B, b+4..2b+3 = 0_A..(b-1)_A.
    Text: S_A id_A(u_0) S_B id_B(v_0) ... where v_j = f_A(u_j), u_{j+1} = f_B(v_j).
    """
    M = len(f_A)
    HASH_A, DOL_A, DIG_B = 0, 1, 2
    HASH_B, DOL_B, DIG_A = b + 2, b + 3, b + 4
    T = []
    u = 0
    seen = set()
    for _ in range(M):
        assert u not in seen
        seen.add(u)
        v = f_A[u]
        T.append(HASH_A)
        T.extend([DOL_A] * (K - 1))
        T.extend([DIG_A + d for d in digits_base_b(u, b, L_dig)])
        T.append(HASH_B)
        T.extend([DOL_B] * (K - 1))
        T.extend([DIG_B + d for d in digits_base_b(v, b, L_dig)])
        u = f_B[v]
    assert u == 0 and len(seen) == M
    return T


def zero_comm_split_2party(f_A, f_B, b, K, L_dig):
    """Bob computes L_bob = L[0..n/2) using ONLY f_B.
    Alice computes L_alice = L[n/2..n) using ONLY f_A."""
    M = len(f_A)
    HASH_A, DOL_A, DIG_B = 0, 1, 2
    HASH_B, DOL_B, DIG_A = b + 2, b + 3, b + 4

    # --- BOB'S HALF (uses ONLY f_B) ---
    inv_B = [0] * M
    for v, u in enumerate(f_B):
        inv_B[u] = v
    # #_A block: sorted by u = 0..M-1; preceding char is last digit of id_B(inv_B[u])
    bob_hash_A = [DIG_B + digits_base_b(inv_B[u], b, L_dig)[-1] for u in range(M)]
    # $_A block: public constant
    bob_dol_A = []
    if K >= 2:
        bob_dol_A.extend([HASH_A] * M)
        if K >= 3:
            bob_dol_A.extend([DOL_A] * ((K - 2) * M))
    # 0_B..(b-1)_B blocks: rotations starting at suffix of id_B(v) followed by #_A $_A^{K-1} id_A(f_B(v))
    bob_items = []
    for v in range(M):
        digs = digits_base_b(v, b, L_dig)
        u = f_B[v]
        for s in range(1, L_dig + 1):
            # suffix of length s in {0..b-1}, followed by #_A (< 0_B), followed by u
            key = tuple(digs[L_dig - s :]) + (-1, u)
            l_char = (DIG_B + digs[L_dig - s - 1]) if s < L_dig else (DOL_B if K >= 2 else HASH_B)
            bob_items.append((key, l_char))
    bob_items.sort(key=lambda t: t[0])
    L_bob = bob_hash_A + bob_dol_A + [c for _, c in bob_items]

    # --- ALICE'S HALF (uses ONLY f_A) ---
    inv_A = [0] * M
    for u, v in enumerate(f_A):
        inv_A[v] = u
    # #_B block: sorted by v = 0..M-1; preceding char is last digit of id_A(inv_A[v])
    alice_hash_B = [DIG_A + digits_base_b(inv_A[v], b, L_dig)[-1] for v in range(M)]
    # $_B block: public constant (since 0_B < $_B, shorter $_B runs precede longer $_B runs!)
    alice_dol_B = []
    if K >= 2:
        if K >= 3:
            alice_dol_B.extend([DOL_B] * ((K - 2) * M))
        alice_dol_B.extend([HASH_B] * M)
    # 0_A..(b-1)_A blocks: rotations starting at suffix of id_A(u) followed by #_B $_B^{K-1} id_B(f_A(u))
    # Note: #_B < 0_A < ... < (b-1)_A!
    alice_items = []
    for u in range(M):
        digs = digits_base_b(u, b, L_dig)
        v = f_A[u]
        for s in range(1, L_dig + 1):
            key = tuple(digs[L_dig - s :]) + (-1, v)
            l_char = (DIG_A + digs[L_dig - s - 1]) if s < L_dig else (DOL_A if K >= 2 else HASH_A)
            alice_items.append((key, l_char))
    alice_items.sort(key=lambda t: t[0])
    L_alice = alice_hash_B + alice_dol_B + [c for _, c in alice_items]

    return L_bob, L_alice


def all_single_cycles(M):
    """Yield all (M-1)! single-cycle permutations on 0..M-1."""
    for perm in itertools.permutations(range(1, M)):
        pi = [0] * M
        cur = 0
        for nxt in perm:
            pi[cur] = nxt
            cur = nxt
        pi[cur] = 0
        yield pi


def main():
    cases_phi = 0
    cases_bin = 0
    cases_2p = 0

    # 1. Exhaustive verification of Phi_BWT over all single-cycle permutations M = 2..6
    for M in range(2, 7):
        for pi in all_single_cycles(M):
            pred = [0] * M
            for u, v in enumerate(pi):
                pred[v] = u
            for b in [2, 3, 4]:
                for K in [1, 2, 4]:
                    T, L_dig = build_T_pi(pi, b, K)
                    L_ref, p_ref = naive_bwt(T)
                    L_tc0, p_tc0 = tc0_construct_L_pi(pred, b, K, L_dig)
                    assert L_ref == L_tc0 and p_ref == p_tc0 == 0, (M, pi, b, K)
                    # Verify exact first-return identity LF^{K+L}(x) == pred[x] on [0..M)
                    LF, _ = compute_LF_int(L_ref, b + 2)
                    step = K + L_dig
                    for x in range(M):
                        cur = x
                        for _ in range(step):
                            cur = LF[cur]
                        assert cur == pred[x], f"First return failed: {cur} != {pred[x]}"
                    cases_phi += 1

    # 2. Exhaustive verification of Binary Phi_BWT_bin (|Sigma| = 2) for M = 2..6
    for M in range(2, 7):
        for pi in all_single_cycles(M):
            pred = [0] * M
            for u, v in enumerate(pi):
                pred[v] = u
            for K in [2, 3, 5]:
                T, L_bits = build_T_pi_binary(pi, K)
                L_ref, p_ref = naive_bwt(T)
                L_tc0, p_tc0 = tc0_construct_L_pi_binary(pred, K, L_bits)
                assert L_ref == L_tc0 and p_ref == p_tc0 == 0, (M, pi, K)
                LF, _ = compute_LF_int(L_ref, 2)
                step = K + 2 * L_bits + 1
                for x in range(M):
                    cur = x
                    for _ in range(step):
                        cur = LF[cur]
                    assert cur == pred[x]
                cases_bin += 1

    # 3. Exhaustive verification of Two-Party Contiguous-Split Phi_BWT_2party
    #    Test all pairs (f_A, f_B) forming a single cycle for M = 2..4, plus layered PermPC!
    for M in [2, 3, 4]:
        all_perms = list(itertools.permutations(range(M)))
        for f_A in all_perms:
            for f_B in all_perms:
                # Check if f_B o f_A is a single cycle
                cur = 0
                cnt = 0
                while True:
                    cur = f_B[f_A[cur]]
                    cnt += 1
                    if cur == 0:
                        break
                if cnt != M:
                    continue
                for b in [2, 3]:
                    L_dig = max(1, math.ceil(math.log(M, b)))
                    for K in [1, 2, 3]:
                        T = build_T_2party(f_A, f_B, b, K, L_dig)
                        L_ref, p_ref = naive_bwt(T)
                        L_bob, L_alice = zero_comm_split_2party(f_A, f_B, b, K, L_dig)
                        assert len(L_bob) == len(L_alice) == len(T) // 2
                        assert L_bob + L_alice == L_ref and p_ref == 0
                        cases_2p += 1

    # 4. Verify Hairpin-Closed PermPC_{N, t} single-cycle 2-party BWT (even t >= 2):
    #    Forward layers 0->1->...->t using pi_1..pi_t; hairpin id at t->t';
    #    Backward layers t'->...->0' using pi_t^{-1}..pi_1^{-1}; shift +1 mod N at 0'->0.
    #    ALWAYS a single cycle of length M_tot = 2*t*N, split into Alice (odd layers) & Bob (even layers)!
    rng = random.Random(20261122)
    cases_hairpin = 0
    for t in [2, 4, 6]:  # even t = 2p
        N = 4
        # Alice maps u in {even layers: 0, 2, ..., t-2, and backward odd' -> even': t', (t-2)', ..., 2'}
        # Let's index vertices cleanly:
        #   Forward layers 0, 1, ..., t-1 (layer l has N vertices: l*N + x)
        #   Backward layers t, t+1, ..., 2t-1 (representing t' down to 1', wait: check parity!)
        # Even cleaner: t+1 forward levels 0..t, t+1 backward levels t'..0':
        # Total 2t edges per lap:
        #   step 0 (Alice, pi_1):     A-node (0, x)       -> B-node (1, pi_1(x))
        #   step 1 (Bob,   pi_2):     B-node (1, y)       -> A-node (2, pi_2(y))
        #   ...
        #   step t-1 (Bob, pi_t):     B-node (t-1, y)     -> A-node (t, pi_t(y))   [A-node (t, v_t) holds the answer!]
        #   step t (Alice, pi_t^{-1}? NO! Alice does NOT know pi_t! Insert public id step? Wait:
        #     Check parity at layer t! If t is even, Bob owns pi_t (step t-1).
        #     Wait: to reverse WITHOUT Bob speaking twice in a row, put a public id bridge:
        #     A-node (t, y) --(Alice: id)--> B-node (t', y) --(Bob: pi_t^{-1})--> A-node ((t-1)', pi_t^{-1}(y))
        #     --(Alice: pi_{t-1}^{-1})--> ... --(Alice: pi_1^{-1})--> B-node (0', x) --(Bob: +1 mod N)--> A-node (0, (x+1)%N)!)
        # Check parity for EVERY step!
        #   Forward (t steps, t even):
        #     A(0) --[Alice: pi_1]--> B(1) --[Bob: pi_2]--> A(2) ... B(t-1) --[Bob: pi_t]--> A(t)
        #   Hairpin bridge + Backward (t+2 steps, t even -> total 2t+2 steps = t+1 A-nodes and t+1 B-nodes!):
        #     A(t) --[Alice: id]--> B(t') --[Bob: pi_t^{-1}]--> A((t-1)') --[Alice: pi_{t-1}^{-1}]--> ...
        #     ... --> A(1') --[Alice: pi_1^{-1}]--> B(0') --[Bob: +1 mod N]--> A(0)!!
        # COUNT:
        #   A-nodes (t+1 layers of N): 0, 2, ..., t, (t-1)', (t-3)', ..., 1'  -> total (t/2 + 1) + (t/2) = t + 1 layers = (t+1)*N nodes!
        #   B-nodes (t+1 layers of N): 1, 3, ..., t-1, t', (t-2)', ..., 0'    -> total (t/2) + (t/2 + 1) = t + 1 layers = (t+1)*N nodes!
        #   EVERY Alice edge uses ONLY odd layers pi_1, pi_3, ..., pi_{t-1} (and their inverses) or id!
        #   EVERY Bob edge uses ONLY even layers pi_2, pi_4, ..., pi_t (and their inverses) or (+1 mod N)!
        #   AND f_B o f_A is ALWAYS A SINGLE CYCLE OF LENGTH (t+1)*N!!
        M_side = (t + 1) * N
        b, K = 4, 3
        L_dig = max(1, math.ceil(math.log(M_side, b)))
        for _ in range(15):
            pis = [rng.sample(range(N), N) for _ in range(t)]  # pi_1..pi_t (0-indexed: pis[0]..pis[t-1])
            invs = []
            for p in pis:
                inv = [0] * N
                for i, v in enumerate(p):
                    inv[v] = i
                invs.append(inv)

            # Map A-layers to 0..t:
            #   fwd even layers 0, 2, ..., t -> indices 0, 1, ..., t//2
            #   bwd odd' layers (t-1)', (t-3)', ..., 1' -> indices t//2 + 1, ..., t
            # Map B-layers to 0..t:
            #   fwd odd layers 1, 3, ..., t-1 -> indices 0, 1, ..., t//2 - 1
            #   bwd even' layers t', (t-2)', ..., 0' -> indices t//2, ..., t
            f_A = [0] * M_side
            f_B = [0] * M_side
            half = t // 2
            # Forward Alice: A(2k) [idx k] --pi_{2k+1}--> B(2k+1) [idx k] for k=0..half-1
            for k in range(half):
                p_map = pis[2 * k]
                for x in range(N):
                    f_A[k * N + x] = k * N + p_map[x]
            # Hairpin bridge Alice: A(t) [idx half] --id--> B(t') [idx half]
            for x in range(N):
                f_A[half * N + x] = half * N + x
            # Backward Alice: A((t-1-2k)') [idx half+1+k] --pi_{t-1-2k}^{-1}--> B((t-2-2k)') [idx half+1+k] for k=0..half-1
            for k in range(half):
                layer_idx = t - 2 - 2 * k  # 0-based index into pis: t-2, t-4, ..., 0 (i.e. pi_{t-1}, ..., pi_1)
                inv_map = invs[layer_idx]
                for x in range(N):
                    f_A[(half + 1 + k) * N + x] = (half + 1 + k) * N + inv_map[x]

            # Forward Bob: B(2k+1) [idx k] --pi_{2k+2}--> A(2k+2) [idx k+1] for k=0..half-1
            for k in range(half):
                p_map = pis[2 * k + 1]
                for x in range(N):
                    f_B[k * N + x] = (k + 1) * N + p_map[x]
            # Backward Bob: B((t-2k)') [idx half+k] --pi_{t-2k}^{-1}--> A((t-1-2k)') [idx half+1+k] for k=0..half-1
            for k in range(half):
                layer_idx = t - 1 - 2 * k  # 0-based: t-1, t-3, ..., 1 (i.e. pi_t, ..., pi_2)
                inv_map = invs[layer_idx]
                for x in range(N):
                    f_B[(half + k) * N + x] = (half + 1 + k) * N + inv_map[x]
            # Closing shift Bob: B(0') [idx t] --(+1 mod N)--> A(0) [idx 0]
            for x in range(N):
                f_B[t * N + x] = (x + 1) % N

            # Verify single cycle AND target v_t = pi_t(...pi_1(0)...) at step t//2 of f_B o f_A!
            T = build_T_2party(f_A, f_B, b, K, L_dig)
            L_ref, p_ref = naive_bwt(T)
            L_bob, L_alice = zero_comm_split_2party(f_A, f_B, b, K, L_dig)
            assert L_bob + L_alice == L_ref and p_ref == 0

            # Compute expected v_t = pi_t(...pi_1(0)...)
            v = 0
            for p_map in pis:
                v = p_map[v]
            expected_A_node = half * N + v
            # Locate A-node at cycle step `half` in T: offset = half * 2 * (K + L_dig) + K
            off = half * 2 * (K + L_dig) + K
            DIG_A = b + 4
            got_digits = [sym - DIG_A for sym in T[off : off + L_dig]]
            assert got_digits == digits_base_b(expected_A_node, b, L_dig)
            cases_hairpin += 1

    # 5. Large-scale Wikidata-density check (M=2000, b=16, L=3, K=32 -> n=70000, r/n ~ 0.11)
    M_big = 2000
    perm = list(range(M_big))
    rng.shuffle(perm)
    pi_big = [0] * M_big
    for i in range(M_big):
        pi_big[perm[i]] = perm[(i + 1) % M_big]
    pred_big = [0] * M_big
    for u, v in enumerate(pi_big):
        pred_big[v] = u
    b_big, K_big, L_big = 16, 32, 3
    L_big_col, p_big = tc0_construct_L_pi(pred_big, b_big, K_big, L_big)
    # Count runs and width-1 runs
    runs = 1
    w1 = 0
    cur_len = 1
    for i in range(1, len(L_big_col)):
        if L_big_col[i] == L_big_col[i - 1]:
            cur_len += 1
        else:
            if cur_len == 1:
                w1 += 1
            runs += 1
            cur_len = 1
    if cur_len == 1:
        w1 += 1
    # Verify exact LF inversion recovers T_pi!
    LF_big, _ = compute_LF_int(L_big_col, b_big + 2)
    T_big_ref, _ = build_T_pi(pi_big, b_big, K_big, L_big)
    n_big = len(L_big_col)
    T_big_rec = [0] * n_big
    row = p_big
    for k in range(n_big - 1, -1, -1):
        T_big_rec[k] = L_big_col[row]
        row = LF_big[row]
    assert row == p_big and T_big_rec == T_big_ref

    print(f"[VERIFIED] Phi_BWT (single-cycle, alphabet b+2): {cases_phi} exhaustive cases PASSED")
    print(f"[VERIFIED] Phi_BWT_bin (binary |Sigma|=2):       {cases_bin} exhaustive cases PASSED")
    print(f"[VERIFIED] Phi_BWT_2party (zero-comm split):     {cases_2p} exhaustive cases PASSED")
    print(f"[VERIFIED] Hairpin PermPC single-cycle 2-party:  {cases_hairpin} cases PASSED (t in {{2,4,6}})")
    print(
        f"[VERIFIED] Large Wikidata-regime Phi_BWT (M={M_big}, b={b_big}, L={L_big}, K={K_big}): "
        f"n={n_big}, r={runs}, r/n={runs/n_big:.4f}, w1/r={w1/runs:.4f}, exact LF recovery OK"
    )


if __name__ == "__main__":
    main()
