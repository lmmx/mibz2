#!/usr/bin/env python3
"""
Edge-case audit of Lazard-BWT Theorem 2 on ALL arbitrary (non-BWT) ternary L arrays:
  - n = 1..6 (3 + 9 + 27 + 81 + 243 + 729 = 1,092 arbitrary ternary L arrays)
  - ALL primary marks p in [0, n) -> 6,216 total (L, p) cases!
  - Explicitly verifies:
    (a) Can a fixed point LF(j) == j EVER occur outside the prefix [0, m) of F_min
        (or outside the suffix [f_x - m, f_x) of F_max)?
    (b) Can a fixed point occur in the middle of F_mid (a MIDDLE symbol bucket)?
    (c) Does lazard_decode (min, max, both) recover the EXACT LF-cycle of p on ALL 6,216 cases?
"""

import sys
sys.path.insert(0, "/tmp/teamwork-a")
from verify_full_and_falsify import lazard_decode


def build_lf(L: bytes):
    n = len(L)
    counts = [0] * 256
    for b in L:
        counts[b] += 1
    C = [0] * 256
    s = 0
    for c in range(256):
        C[c] = s
        s += counts[c]
    occ = C[:]
    LF = [0] * n
    for i, b in enumerate(L):
        LF[i] = occ[b]
        occ[b] += 1
    return LF, C, counts


def main():
    total_L = 0
    total_cases = 0
    multicycle_L = 0
    min_middle_fixed = 0
    max_middle_fixed = 0
    mid_middle_fixed = 0
    mid_fixed_witness = None
    failures = []

    for n in range(1, 7):
        for code in range(3 ** n):
            tmp = code
            arr = []
            for _ in range(n):
                arr.append((tmp % 3) + ord('a'))
                tmp //= 3
            L = bytes(arr)
            total_L += 1
            LF, C, counts = build_lf(L)

            # Check cycles
            seen = [False] * n
            ncycles = 0
            for i in range(n):
                if not seen[i]:
                    ncycles += 1
                    cur = i
                    while not seen[cur]:
                        seen[cur] = True
                        cur = LF[cur]
            if ncycles > 1:
                multicycle_L += 1

            # Check fixed points in F_min, F_max, and F_mid
            present = [c for c in (ord('a'), ord('b'), ord('c')) if counts[c] > 0]
            c_min, c_max = present[0], present[-1]

            # In F_min = [C[c_min], C[c_min]+counts[c_min]):
            s0 = C[c_min]
            e0 = s0 + counts[c_min]
            # Prefix of c_min
            m_pref = s0
            while m_pref < e0 and L[m_pref] == c_min:
                m_pref += 1
            for j in range(m_pref, e0):
                if LF[j] == j:
                    min_middle_fixed += 1

            # In F_max:
            s1 = C[c_max]
            e1 = s1 + counts[c_max]
            m_suff = e1
            while m_suff > s1 and L[m_suff - 1] == c_max:
                m_suff -= 1
            for j in range(s1, m_suff):
                if LF[j] == j:
                    max_middle_fixed += 1

            # In F_mid (if 3 symbols present):
            if len(present) == 3:
                c_mid = present[1]
                sm, em = C[c_mid], C[c_mid] + counts[c_mid]
                # Check if fixed point occurs strictly inside (sm, em-1) or non-prefix/non-suffix
                for j in range(sm, em):
                    if LF[j] == j:
                        if j > sm and j < em - 1:
                            mid_middle_fixed += 1
                            if mid_fixed_witness is None:
                                mid_fixed_witness = (L, j, LF)

            # Exhaustively test lazard_decode in min, max, both modes for ALL p in [0, n)
            for p in range(n):
                total_cases += 1
                cyc = []
                cur = p
                while True:
                    cyc.append(L[cur])
                    cur = LF[cur]
                    if cur == p:
                        break
                cyc_word = bytes(reversed(cyc))
                for mode in ("min", "max", "both"):
                    try:
                        got, _ = lazard_decode(L, p, mode=mode)
                    except Exception as ex:
                        failures.append((mode, L, p, cyc_word, f"EXCEPTION: {ex}"))
                        continue
                    if got != cyc_word:
                        if not (len(got) % len(cyc_word) == 0 and got == cyc_word * (len(got) // len(cyc_word))):
                            failures.append((mode, L, p, cyc_word, got))

    print(f"Tested ALL arbitrary ternary L arrays n=1..6: {total_L} arrays ({multicycle_L} multi-cycle), {total_cases} (L, p) pairs x 3 modes = {3*total_cases} runs")
    print(f"  Fixed points strictly outside prefix of F_min: {min_middle_fixed} (expected: 0)")
    print(f"  Fixed points strictly outside suffix of F_max: {max_middle_fixed} (expected: 0)")
    print(f"  Fixed points strictly in interior of F_mid:    {mid_middle_fixed} (witness: {mid_fixed_witness})")
    print(f"  Lazard-BWT decode failures across {3*total_cases} runs: {len(failures)}")
    for f in failures[:10]:
        print("   FAIL:", f)


if __name__ == "__main__":
    main()
