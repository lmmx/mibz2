#!/usr/bin/env python3
"""N-19: Constructive verification of exact (T1) decoders in Lambda''s neighbourhood
that write width-1 bytes by neither a link nor a rigid-placed fact, plus 6-block
quantitative ceiling for histogram-completion (max_c h1(c) / h1).

Mechanisms verified:
  1. Lambda' + fill_remaining(c*) (histogram-completion / omission write):
     - Minimal witness (n=3, k=1): L="cab", p=0 -> h1=3, |B|=0, c1=2 (row 1 omitted).
     - Amplified family (k=1..6): valid single-cycle columns with h1=n whose last k
       LF-orbit rows are all width-1 runs of 'a', giving exact T1 decoding with
       c1 = h1 - k and |B| = 0 using only H and one forward chain.
     - 6-block quantitative ceiling: max_c h1(c) / h1 on origin/data/admitted-bwt.
  2. Lambda' + copy(dst, src, len) (output-to-output positional copy write):
     - Concrete valid single-cycle column with a repeated substring whose second
       occurrence contains k >= 2 width-1 rows; one copy writes all k width-1 bytes
       without any link or rigid fact through those rows (even charging 1 for copy,
       c1 = h1 - k + 1 < h1 with |B| = 0).
  3. Lambda' + E1 (scattered class-fact write):
     - Completes round-2's 18-row witness L="Paaabxbccddeeff$MZ", p=0 into a full
       byte-exact decoder: one E1 class fact on S={1, 3} writes interior width-1
       rows 4 and 6 with zero rigid facts or links through them.
"""

from __future__ import annotations

import sys
from collections import Counter
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from measure_cycle2 import compute_lf, load_admitted_blocks, orbit_maps  # noqa: E402


def bwt_from_text(T: bytes) -> tuple[bytes, int, list[int], list[int], list[int]]:
    n = len(T)
    rots = sorted(range(n), key=lambda j: T[j:] + T[:j])
    # Check primitivity (all rotations distinct)
    assert len({T[j:] + T[:j] for j in range(n)}) == n, "Text is not primitive"
    L = bytes(T[(j - 1) % n] for j in rots)
    p = rots.index(0)
    lf = compute_lf(L)
    T2, row_of_pos, lf_pos = orbit_maps(L, p, lf)
    assert T2 == T
    return L, p, lf, row_of_pos, lf_pos


def w1_rows(L: bytes) -> set[int]:
    n = len(L)
    out = set()
    for i in range(n):
        if (i == 0 or L[i - 1] != L[i]) and (i + 1 == n or L[i + 1] != L[i]):
            out.add(i)
    return out


def verify_mechanism1_histogram_completion() -> None:
    print("=== N-19 MECHANISM 1: Lambda' + fill_remaining(c*) (HISTOGRAM COMPLETION) ===")

    # 1a. Minimal witness: L="cab", p=0, n=3, h1=3, |B|=0, c1=2
    L = b"cab"
    p = 0
    lf = compute_lf(L)
    T_true, row_of_pos, lf_pos = orbit_maps(L, p, lf)
    assert lf == [2, 0, 1] and T_true == b"abc"
    j1 = w1_rows(L)
    assert j1 == {0, 1, 2}

    H = Counter(L)
    out = [None] * len(L)
    written = Counter()
    x = p
    links = 0
    # Stop 1 step early: execute n-1 = 2 links (rows 0 and 2); skip row 1 completely
    visited = set()
    for step in range(len(L) - 1):
        j = (len(L) - 1 - step) % len(L)
        out[j] = L[x]
        written[L[x]] += 1
        visited.add(x)
        links += 1
        x = lf[x]
    rem = H - written
    assert len(rem) == 1
    fill_sym, fill_cnt = next(iter(rem.items()))
    assert fill_cnt == 1
    for j in range(len(L)):
        if out[j] is None:
            out[j] = fill_sym
    assert bytes(out) == T_true
    omitted_rows = set(range(len(L))) - visited
    assert omitted_rows == {1} and omitted_rows <= j1
    print(
        f"  [1a] Minimal witness L={L.decode()!r}, p={p} -> T={T_true.decode()!r}: "
        f"h1={len(j1)}, |B|=0, c1={links} < h1; omitted w1 row={sorted(omitted_rows)} (byte={chr(fill_sym)!r})"
    )

    # 1b. Amplified family for k = 1..6:
    # Construct primitive text T_k starting with 'a'^k followed by distinct delimiters
    # separating 'a'^j blocks so that ALL n rows of L are width-1 (h1 = n),
    # and the last k LF-orbit rows (emitting T[0..k-1] = 'a'^k) are all width-1 'a' runs.
    # For T = 'a'^k + 'z' + d1 + 'a'^(k-1) + 'b' + d2 + 'a'^(k-2) + 'b' + ... + d_{k-1} + 'a' + 'b' + d_k:
    # Wait: even simpler, let's build T_k and verify directly!
    print("  [1b] Amplified family (single chain stopping k steps early, |B|=0, H-completion):")
    for k in range(1, 7):
        # We want:
        #   T[0..k) = 'a'^k, followed by 'z',
        #   and for each j = 1..k-1, a separator block (unique byte > 'z' or < 'a') + 'a'^j + 'b'
        #   so rot(k-j) starting with 'a'^j 'z' is separated from rot(k-j+1) starting with 'a'^(j-1) 'z'
        #   AND ensure every symbol in L has run width 1 (h1 = n) or compute exact c1 vs h1.
        parts = [b"a" * k, b"z"]
        # Use unique separator bytes starting from ord('c') .. ord('y')
        sep_code = ord("c")
        for j in range(k - 1, 0, -1):
            # Insert sep + 'a'^j + 'b' so rotations starting with 'a'^j + 'b' separate 'a'^(j+1) from 'a'^j + 'z'
            # To avoid adjacent 'a's in L from the 'a'^j inside the separator itself, use distinct symbols
            # Actually, for the separator we only need a rotation starting with 'a'^j + 'b' whose PRECEDING byte != 'a'!
            # Putting sep + 'a'^j + 'b' puts 'a' at rot(inside) too; instead we only need 'a'^j + 'b' for j >= 1,
            # and any extra wide 'a' runs outside the last k steps only affect non-J1 rows!
            # Better yet: use ONLY ONE chain of rigid facts (Open-style maximal wide-run composes) OR check h1!
            parts.append(bytes([sep_code]))
            sep_code += 1
            parts.append(b"a" * j)
            parts.append(bytes([ord("b")]))
        # Use distinct letters for the 'b' markers too so no 'b' run has width >= 2, or just check!
        # Wait: even cleaner: use byte values 100+ for separators, and check if we can make ALL n rows width-1!
        # Actually, for separating 'a'^k 'z', 'a'^(k-1) 'z', ..., 'a' 'z', notice:
        #   rot(0) = a^k z ... (preceded by T[-1] != 'a')
        #   rot(1) = a^(k-1) z ... (preceded by 'a')  <-- emits T[0]
        #   rot(2) = a^(k-2) z ... (preceded by 'a')  <-- emits T[1]
        #   ...
        #   rot(k) = z ...         (preceded by 'a')  <-- emits T[k-1]
        # We ONLY need ONE non-'a'-preceded rotation between rot(j) and rot(j+1) in lex order!
        # Since rot(j) starts with a^(k-j) z, putting sep_j + a^(k-j) + 'y' (with 'a' < 'y' < 'z')
        # puts a^(k-j) 'y' IMMEDIATELY BEFORE a^(k-j) 'z' with L-byte = sep_j != 'a'!
        # To keep all 'a's inside the separator isolated in L too, make the separator 'a'^(k-j) use
        # slightly higher follower byte? Wait: a^(k-j-1) 'y' sorts between a^(k-j-1) 'x' and a^(k-j-1) 'z'!
        # Let's test directly and count exact c1 (links on visited rows, with wide runs contracted for free!)
        # Actually: even simpler — with plain LF walk (1 link per visited row), we win iff
        #   (n - k) < h1  <==>  w_wide < k!
        # Check whether our simple construction has w_wide == 0 (i.e. h1 == n) or w_wide < k!
        # Build T with separators: sep_1 + a^(k-1) + b_1, ..., where b_j are distinct < 'z'
        # Wait: why would any wide run occur at all if all bytes except 'a' are UNIQUE?
        # Every non-'a' byte appears once -> width 1!
        # An 'a' appears in L at rot(pos+1) where T[pos] == 'a'.
        # If the sorted order of {rot(pos+1) : T[pos]=='a'} has a non-'a'-preceded rotation between EVERY pair,
        # then EVERY 'a' run in L has width 1, so h1 == n!!
        # Let's achieve that! Put T[0..k) = 'a'^k followed by 'z'.
        # Then the k rotations preceded by 'a' are:
        #   a^(k-1)z, a^(k-2)z, ..., az, z.
        # Note that the (k-1) rotations a^(k-1)z, ..., az are ALL distinct lengths of 'a' followed by 'z'!
        # Between a^m z and a^(m-1) z we just need a rotation starting with a^m 'b' preceded by a unique byte u_m!
        # Wait: if we insert u_m + a^m + b_m, THAT adds m more 'a's whose preceded rotations are a^(m-1)b_m, ..., b_m!
        # Instead, insert ONLY ONE 'a'-free separator? No, we need a rotation starting with 'a'^m!
        # Wait: if we ONLY need to separate a^m z from a^(m-1) z, ANY string starting with a^m 'y' works,
        # AND if we choose unique bytes y_m so a^(m-1) y_m is ALSO separated? Wait: let's just check what
        # happens with T = 'a'^k + 'z' + ''.join(u_j + 'a' + v_j ...):
        # For k=2: T = "aaz" -> rot(1)="az" (L=a), rot(2)="z" (L=a) -> DIFFERENT F-buckets ('a' vs 'z')!
        #   So T="aazc" ALREADY has h1=4=n and k=2!!
        # For k=3: T = "aaaz" -> rot(1)="aaz"(L=a), rot(2)="az"(L=a), rot(3)="z"(L=a).
        #   Between "aaz" and "az" put "ab" preceded by 'c': T = "aaazcab"!
        #   Check rot("ab") -> L='c', rot("b") -> L='a' (in 'b' bucket, separated from 'z'!).
        #   ALL 7 rows of T="aaazcab" have width 1 (h1 = 7 = n)!!
        # In general for k: between a^m z and a^(m-1) z, put u_m + a^(m-1) + ... wait, for k=3 we only needed "cab" (ONE 'a'!)
        # Wait: for k=4 ("aaaaz"), rot(1)="aaaz", rot(2)="aaz", rot(3)="az", rot(4)="z".
        #   Between "aaaz" and "aaz" we need something between "aaaz" and "aaz": e.g. "aab" (two 'a's: "aab" has L=u, "ab" has L=a, "b" has L=a)!
        #   "ab" sits between "aaz" and "az", wait: "aaz" < "ab" < "az"! That puts "ab"(L=a) NEXT TO "az"(L=a)!
        #   Aha! So put u_1 + "ac" (L=u_1) between "ab" and "az"!
        #   Even better: ANY two adjacent 'a's in L can be separated by inserting ONE unique delimiter + matching prefix,
        #   OR we can compute exact minimal Open-style rigid cost on the first n-k steps!
        # Let's build exact h1=n strings greedily by inserting a separating token whenever two 'a's are adjacent in L!
        base = bytearray(b"a" * k + b"z")
        next_sym = 128  # distinct separator bytes (> 'z' or use bytes in 1..96 except 'a')
        # Actually use symbols < 'a' (say 1..95) for unique followers so they sort BEFORE 'z' and BEFORE 'a' if needed,
        # and > 'z' (128..250) for unique leaders so they don't interfere with 'a' bucket!
        lead = 128
        foll = ord("b")
        # Construct T starting with 'a'^k + 'z' + minimal separating gadgets so ALL k rows of T[0..k) are width-1
        # AND every other 'a' run in L is width-1 (so h1 == n!)
        for _ in range(50):
            T_cur = bytes(base)
            L_cur, p_cur, lf_cur, row_of_pos_cur, _ = bwt_from_text(T_cur)
            # Find if any two adjacent rows in L_cur are equal
            adj = None
            for idx in range(len(L_cur) - 1):
                if L_cur[idx] == L_cur[idx + 1]:
                    adj = idx
                    break
            if adj is None:
                break
            # Find the two rotations at adj and adj+1
            rots = sorted(range(len(T_cur)), key=lambda j: T_cur[j:] + T_cur[:j])
            r1 = T_cur[rots[adj] :] + T_cur[: rots[adj]]
            r2 = T_cur[rots[adj + 1] :] + T_cur[: rots[adj + 1]]
            # Find first index where r1 and r2 differ
            d = 0
            while d < len(r1) and r1[d] == r2[d]:
                d += 1
            # Insert lead + r1[:d] + byte strictly between r1[d] and r2[d] (or r1[:d+1] + higher byte)
            # Since r1[:d] == r2[:d] and r1[d] < r2[d], r1[:d+1] + b'\xff' sorts strictly between r1 and r2!
            # Wait: r1[:d+1] might equal r1's prefix; better: r1[:d] + bytes([r1[d]]) + ...
            # Wait if r1[d] == ord('a') and r2[d] > ord('a'): r1[:d] + b'b' (if r2[d] > b'b') sorts strictly between!
            if r1[d] + 1 < r2[d]:
                mid_byte = (r1[d] + r2[d]) // 2
                gadget = bytes([lead]) + r1[:d] + bytes([mid_byte])
            else:
                # r2[d] == r1[d] + 1: take r1[:d+1] + (r1[d+1] + 1)
                mid_byte = min(254, r1[d + 1] + 1) if d + 1 < len(r1) else 254
                gadget = bytes([lead]) + r1[: d + 1] + bytes([mid_byte])
            lead += 1
            base.extend(gadget)

        T_k = bytes(base)
        L_k, p_k, lf_k, row_of_pos_k, lf_pos_k = bwt_from_text(T_k)
        j1_k = w1_rows(L_k)
        # Verify the last k orbit steps emit T_k[0..k) == 'a'^k and ALL k rows are in J1
        omitted_k = [row_of_pos_k[j] for j in range(k)]
        assert all(L_k[r] == ord("a") for r in omitted_k)
        assert all(r in j1_k for r in omitted_k), f"k={k}: omitted rows not all width-1"
        # Run exact decoder stopping k steps early + histogram completion
        H_k = Counter(L_k)
        out_k = [None] * len(L_k)
        written_k = Counter()
        x = p_k
        links_k = 0
        for step in range(len(L_k) - k):
            j = (len(L_k) - 1 - step) % len(L_k)
            out_k[j] = L_k[x]
            written_k[L_k[x]] += 1
            links_k += 1
            x = lf_k[x]
        rem_k = H_k - written_k
        assert len(rem_k) == 1 and rem_k[ord("a")] == k
        for j in range(len(L_k)):
            if out_k[j] is None:
                out_k[j] = ord("a")
        assert bytes(out_k) == T_k
        # Check exact gap: links_k = n - k; if h1 == n then c1 = h1 - k!
        # Even if some non-omitted wide runs exist, check whether links_k < len(j1_k)
        assert len(j1_k) == len(L_k), f"k={k}: expected h1==n, got h1={len(j1_k)}, n={len(L_k)}"
        print(
            f"    k={k}: n={len(L_k):>3}, h1={len(j1_k):>3} (all w1), |B|=0, "
            f"c1={links_k:>3} = h1 - {k} (byte-exact T1 verified)"
        )

    # 1c. Quantitative ceiling on the 6 admitted blocks: max_c h1(c) / h1
    print("  [1c] Quantitative ceiling on 6 admitted blocks (max_c h1(c) / h1):")
    for fname, p_b, L_b in load_admitted_blocks():
        n_b = len(L_b)
        cnt_w1 = Counter()
        i = 0
        while i < n_b:
            j = i + 1
            b = L_b[i]
            while j < n_b and L_b[j] == b:
                j += 1
            if j - i == 1:
                cnt_w1[b] += 1
            i = j
        h1_b = sum(cnt_w1.values())
        top3 = cnt_w1.most_common(3)
        top_str = ", ".join(f"{chr(b)!r}:{c}({100*c/h1_b:.2f}%)" for b, c in top3)
        print(f"    {fname:<12} h1={h1_b:>6,} | top-3 w1 bytes: {top_str}")


def verify_mechanism2_copy_write() -> None:
    print("\n=== N-19 MECHANISM 2: Lambda' + copy(dst, src, len) (OUTPUT-TO-OUTPUT COPY) ===")
    # Construct a primitive text T = u + sep1 + u + sep2 where the second copy of u has k >= 2
    # width-1 rows in L, AND every row of L is either in J1 or covered cleanly.
    # Take u = b"pqrs" repeated twice with distinct surrounding context:
    # Let's test T = b"abpqrs1cdpqrs2ef" and inspect its BWT runs!
    # Wait: if u="pqrs" occurs twice, its interior ('q','r','s') may be wide, while its boundary ('p', and possibly 'q') can be width-1!
    # Even simpler: repeat TWO different substrings OR repeat u="pq" (2 bytes) with different right contexts so BOTH 'p' and 'q' are width-1!
    # Check: if T has "Xp q Y" and "Zp q W" with Y < ... < W (another 'q'-prefixed rotation between qY and qW, and another 'p'-prefixed rotation between pqY and pqW?):
    # Wait: put "pA" between "pqY" and "pqW" -> impossible because 'p' only followed by 'q' in both!
    # Instead: "pq1" and "pq2", with "p0" or "pq0" between? No, "pq" followed by '1' and '3', with "pq2" preceded by 'c' != 'p'!
    # Let's check directly: T = b"xab1yab3zab2w"! Wait:
    #   rot("ab1...") has L='x', rot("ab2...") has L='z', rot("ab3...") has L='y' -> 'a's are at rot("b1"), rot("b2"), rot("b3")!
    #   Between "b1" and "b3", "b2" has L='a' too. Put "c" before "b2": T = b"xab1yab3zcb2w"!
    #   Then rot("b1") has L='a', rot("b2") has L='c', rot("b3") has L='a'!
    #   AND between "ab1" and "ab3", put "a2" preceded by 'd': T = b"xab1yab3zcb2da2w" -> let's test!
    T = b"xab1yab3zcb2da2w"
    # Wait: here "ab" occurs at pos 1 ("ab1") and pos 5 ("ab3"), and BOTH bytes of the second "ab" (pos 5='a', pos 6='b')
    # are preceded in T by 'y' and 'a', with emitting rows rot(6)="b3..." (L='a') and rot(7)="3..." (L='b')!
    L, p, lf, row_of_pos, lf_pos = bwt_from_text(T)
    j1 = w1_rows(L)
    # Check all bytes of second copy T[5:7] == b"ab" (matches T[1:3] == b"ab")
    # Even better: make a length-3 repeat "abc" where ALL 3 bytes of the second copy are width-1!
    # T = b"1abc23abc56b47c48a49" -> let's verify!
    # Check T_copy: repeat u = b"abc" at pos 1 and pos 6:
    #   T[1:4] = "abc" (followed by '2'), T[6:9] = "abc" (followed by '5').
    #   Emitting rows of T[6]='a', T[7]='b', T[8]='c' are:
    #     rot(7) = "bc5..." (L='a'), separated from rot(2) = "bc2..." (L='a') by "bc3..." (L='x')!
    #     rot(8) = "c5..."  (L='b'), separated from rot(3) = "c2..."  (L='b') by "c3..."  (L='y')!
    #     rot(9) = "5..."   (L='c'), in F-bucket '5' (completely separate from F-bucket '2')!
    # T has repeat u = b"abc" twice, with unique separators ensuring ALL n rows of L are width-1:
    #   rot("bc2") [L='a'] < rot("bc4") [L='x'] < rot("bc8") [L='a']
    #   rot("c2")  [L='b'] < rot("c3")  [L='y'] < rot("c4")  [L='b'] < rot("c6") [L='w'] < rot("c8") [L='b']
    #   and digits 2 < 25 < 3 < 35 < 4 < 45 < 6 < 65 < 8 separate all 'c'-preceded rows in L.
    T = b"1abc2p2Ayc3q3Bxbc4r4Cwc6s6D7abc8z"
    L, p, lf, row_of_pos, lf_pos = bwt_from_text(T)
    j1 = w1_rows(L)
    # Find where second "abc" sits:
    src = T.index(b"abc")
    dst = T.index(b"abc", src + 1)
    assert T[src : src + 3] == T[dst : dst + 3] == b"abc"
    copied_rows = [row_of_pos[dst + off] for off in range(3)]
    assert all(r in j1 for r in copied_rows), f"Not all copied rows are w1: {copied_rows}, j1={j1}"
    # Check that EVERY row of L is width-1 (h1 == n)!
    assert len(j1) == len(L), f"Expected h1==n, got h1={len(j1)}, n={len(L)}, L={L}"

    # Execute exact decoder:
    # Note: to stop a forward LF walk before T[dst..dst+3) and resume after it WITHOUT paying 3 links,
    # put the second copy of "abc" at the VERY END of the LF walk, i.e., at T[0:3] (or use forward+copy)!
    # Wait: in an acyclic left-to-right decoder, T[dst..dst+3) is at the END of T (say T[n-3:n]),
    # which in LF walk is the START of the walk (steps 0, 1, 2 from p)!
    # If we start the forward LF walk at row y = LF^3(p) (1 positional label P_1 at y, charged 1 event!)
    # and walk n - 3 steps to emit T[0..n-3), then execute ONE copy(dst=n-3, src, 3) (charged 1 event!),
    # total c1 = 1 (P_1 at y) + (n - 3 links) + 1 (copy) = n - 1 = h1 - 1 < h1!
    # Even better: make the repeat length 4 so c1 = h1 - 2, OR put the first copy at T[0:4] and second at T[n-4:n]!
    # Rotate T so the second "abc" is at T[n-3:n] (the START of LF walk: steps 0, 1, 2)!
    rot_shift = (dst + 3) % len(T)
    T_rot = T[rot_shift:] + T[:rot_shift]
    L_r, p_r, lf_r, row_of_pos_r, lf_pos_r = bwt_from_text(T_rot)
    j1_r = w1_rows(L_r)
    assert len(j1_r) == len(L_r)  # same multiset of rotations -> same L, same h1 = n!
    src_r = T_rot.index(b"abc")
    dst_r = len(T_rot) - 3
    assert src_r < dst_r and T_rot[src_r : src_r + 3] == T_rot[dst_r : dst_r + 3] == b"abc"
    copied_rows_r = [row_of_pos_r[dst_r + off] for off in range(3)]
    assert all(r in j1_r for r in copied_rows_r)

    # Exact simulation of Lambda' + copy decoder:
    # Step 1: 1 charged P_1 label placing y = LF^3(p_r) at pos(y) = 3 (writing T[n-4] down to T[0] via n-3 links).
    # Wait: a link at y ALSO places it if y is reached, or P_1 places y (1 event) + (n-3) links?
    # Note: in Record 107, a fact placed by P_1 is NOT ALSO charged a link at its start row!
    # So starting at y via P_1 + emitting prim(y) costs 1 event at y (NOT 2), then n-4 further links -> total n-3 events!
    # Plus 1 charged copy(dst=n-3, src=src_r, len=3) -> total c1 = (n - 3) + 1 = n - 2 = h1 - 2 < h1!
    # (Even if you charged BOTH P_1 AND a link at y AND copy, c1 = n - 1 = h1 - 1 < h1!)
    y = lf_r[lf_r[lf_r[p_r]]]
    out_r = [None] * len(L_r)
    x = y
    visited_r = set()
    link_or_p1_events = 0
    for step in range(3, len(L_r)):
        j = (len(L_r) - 1 - step) % len(L_r)
        out_r[j] = L_r[x]
        visited_r.add(x)
        link_or_p1_events += 1
        x = lf_r[x]
    # Now copy T_rot[src_r : src_r + 3] to T_rot[dst_r : dst_r + 3]
    for off in range(3):
        assert out_r[src_r + off] is not None
        out_r[dst_r + off] = out_r[src_r + off]
    assert bytes(out_r) == T_rot
    unvisited_r = set(range(len(L_r))) - visited_r
    assert unvisited_r == set(copied_rows_r) and unvisited_r <= j1_r
    c1_total = link_or_p1_events + 1  # charging 1 for the copy operation too
    print(
        f"  Witness T={T_rot.decode()!r}, L={L_r.decode()!r}, p={p_r}: "
        f"n={len(L_r)}, h1={len(j1_r)} (all w1), |B|=0"
    )
    print(
        f"  copy(dst={dst_r}, src={src_r}, len=3) writes 'abc' at w1 rows {sorted(unvisited_r)} "
        f"with 0 rigid facts/links through them:"
    )
    print(
        f"  c1 = {link_or_p1_events} (walk/P1) + 1 (copy) = {c1_total} < {len(j1_r)} = h1 - |B| (byte-exact T1 verified)"
    )


def verify_mechanism3_e1_class_fact() -> None:
    print("\n=== N-19 MECHANISM 3: Lambda' + E1 (SCATTERED CLASS-FACT WRITE, ROUND-2 18-ROW WITNESS) ===")
    L = b"Paaabxbccddeeff$MZ"
    p = 0
    n = len(L)
    lf = compute_lf(L)
    assert lf == [2, 4, 5, 6, 7, 17, 8, 9, 10, 11, 12, 13, 14, 15, 16, 0, 1, 3]
    T_true, row_of_pos, lf_pos = orbit_maps(L, p, lf)
    j1 = w1_rows(L)
    # Trace the two 6-step paths starting at S = {1, 3}
    paths = {}
    words = {}
    for s in (1, 3):
        cur = s
        path = []
        w = bytearray()
        for _ in range(6):
            path.append(cur)
            w.append(L[cur])
            cur = lf[cur]
        paths[s] = (path, cur)
        words[s] = bytes(w)
    assert words[1] == words[3] == b"abcdef"
    assert paths[1] == ([1, 4, 7, 9, 11, 13], 15)
    assert paths[3] == ([3, 6, 8, 10, 12, 14], 16)
    # Rows 4 and 6 are interior width-1 'b' runs!
    assert 4 in j1 and 6 in j1
    assert L[4] == ord("b") and L[6] == ord("b")

    # Complete byte-exact decoder on L using ONE E1 class fact f_E1 = ({1, 3}, h=6, next={15,16}, w="abcdef")
    # plus ordinary single-step links on the remaining 6 rows {0, 2, 5, 17, 15, 16}:
    # Orbit from p=0:
    #   0(P) -> 2(a) -> 5(x) -> 17(Z) -> 3[E1: 3,6,8,10,12,14 -> 16] -> 16(M) -> 1[E1: 1,4,7,9,11,13 -> 15] -> 15($) -> 0
    out = [None] * n
    x = p
    step = 0
    events = []
    # Derive f_E1 once (charged 1 schematic event, NOT attributed to 4 or 6)
    events.append(("E1_derive", (1, 3)))
    while step < n:
        if x in (1, 3):
            # Apply the E1 class fact at start row x (charged 1 link at x; interior rows 4 or 6 NOT touched by rigid compose!)
            events.append(("E1_link", x))
            w = words[x]
            for off, b in enumerate(w):
                j = (n - 1 - (step + off)) % n
                out[j] = b
            x = paths[x][1]
            step += 6
        else:
            events.append(("link", x))
            j = (n - 1 - step) % n
            out[j] = L[x]
            x = lf[x]
            step += 1
    assert x == p and bytes(out) == T_true

    # Count charged events attributed to J1 rows vs h1:
    # Note j1 = {0, 4, 5, 6, 15, 16, 17} -> h1 = 7!
    # Wait: let's check exact j1 and total events!
    j1_sorted = sorted(j1)
    # Links executed: 0, 2, 5, 17, 3(E1), 16, 1(E1), 15 -> 8 links + 1 E1_derive = 9 total events (across all widths),
    # and on J1 rows specifically: {0, 5, 17, 16, 15} = 5 events < 7 = h1 (rows 4 and 6 have ZERO events)!
    # Even better: 1 and 3 are inside the width-3 'a' run [1, 4), so rows 1, 2, 3 are WIDE, while
    # the E1 fact also collapses 12 rows (including w1 rows 4 and 6) into 2 links!
    # Wait: if we want TOTAL c1 < h1 (not just c1|_{J1} < h1), let's check:
    # here h1 = 7, total links = 8 (because wide rows 1, 2, 3 each paid a link).
    # Can we also note BOTH c1|_{J1} = 5 < 7 = h1 (violating G1's row-preserving bound!) AND how
    # extending the shared word to pass through k >= 4 singleton runs makes TOTAL c1 < h1?!
    # YES! Let's ALSO verify an extended E1 witness where TOTAL c1 < h1 (with |B| = 0)!
    print(
        f"  [3a] Round-2 witness L={L.decode()!r}, p=0 -> T={T_true.decode()!r}: "
        f"J1={j1_sorted} (h1={len(j1)}), |B|=0"
    )
    print(
        f"       E1 class fact on S={{1,3}} (w='abcdef') writes interior w1 rows 4, 6 with 0 rigid facts:"
    )
    print(
        f"       c1|_J1 = 5 < 7 = h1 (total events = {len(events)}: 8 links + 1 E1_derive; byte-exact T1 verified)"
    )

    # [3b] Extended E1 witness where TOTAL c1 < h1 (not just c1|_J1 < h1)!
    # Take T with a repeated substring w of length 6 whose 2 occurrences pass through 2*5 = 10 width-1 rows!
    # Even simpler: T = b"1bcdefg23bcdefg4" where 'b','c','d','e','f','g' are separated into singleton runs
    # by intervening single-use gadgets, OR simply use L = "Paaabxbcycdydzdeze..." where EVERY step in w is width-1!
    # Construct by taking L_base = b"Paaabxbccddeeff$MZ" and splitting the 'c','d','e','f' pairs with unique separator bytes
    # inserted into the orbit! Let's build from text directly:
    # Take w = b"bcdef" repeated twice: T = b"1abcdef2" + b"3abcdef4" + separators splitting each of b,c,d,e,f!
    # Check: T = b"AabcdefB" + b"CabcdefD" + b"EcdefG" + b"HdefI" + b"JefK" + b"LfM" + b"NbcdeO"?
    # No: to split the pair of rows emitting 'b' (which are rot("cdefB") and rot("cdefD")), insert "cdefC" preceded by 'X'!
    # Wait: rot("cdefC") has L='X' != 'b' and sorts strictly between "cdefB" and "cdefD"!
    # And its next rotation "defC" has L='c', which sorts strictly between "defB" and "defD" -> wait, that has L='c' (same as the pair!).
    # Instead, insert unique prefix bytes: for each suffix s_i = w[i:], put u_i + w[i:] + 'C' ONLY if...
    # Actually, we want BOTH starts (at 'a') to have the same 5-byte word "bcdef", and rows at 'b','c','d','e','f'
    # to be width-1 in L! In L="Paaabxbccddeeff$MZ", WHY was 'b' width-1? Because 'x' was inserted between the two 'b's,
    # and 'x' came from row 5 = rot("...")!
    # Let's generalize L="Paaabxbccddeeff$MZ" directly in L-space!
    # Start with S = {1, 3} (two 'a's in "Paaa"), followed by 5 pairs of ('b','c','d','e','f') EACH separated by a unique uppercase separator!
    # Wait: in L, each unique uppercase separator maps to its own singleton F-bucket, and we just need LF to be ONE cycle!
    # Let's test a direct L construction:
    #   L = "PaaabXb cYc dZd eWe fVf $MZ" (without spaces) -> wait, uppercase bytes < 'a' in ASCII!
    #   Check ASCII order: '$' (36) < 'A'..'Z' (65..90) < 'a'..'z' (97..122).
    #   In L="Paaabxbccddeeff$MZ", '$' < 'M' < 'P' < 'Z' < 'a' < 'b' < ... < 'f' < 'x'.
    #   Putting separators 'v','w','x','y','z' (> 'f'!) between each pair of 'b','c','d','e','f':
    #   Wait: if we put 5 separators v,w,x,y,z between the 5 pairs b,c,d,e,f, each separator goes to the END of L (rows >= 19),
    #   where we place symbols that link back! Even simpler: just search small permutations of the tail bytes for single-cycle!
    head = list(b"Paaabvbwcwxdxyeyfzf")  # wait: 'a'x3, 'b'v'b', 'c'w'c', 'd'x'd', 'e'y'e', 'f'z'f' -> ALL 10 of b,c,d,e,f are width-1!
    # Actually let's use distinct lowercase > 'f': v,w,x,y,z (5 separators) between b,c,d,e,f pairs:
    # Prefix: P(0), a(1), a(2), a(3), b(4), v(5), b(6), c(7), w(8), c(9), d(10), x(11), d(12), e(13), y(14), e(15), f(16), z(17), f(18)
    # Count of chars so far:
    #   P:1, a:3, b:2, c:2, d:2, e:2, f:2, v:1, w:1, x:1, y:1, z:1 -> 19 chars.
    #   We also need 5 + 2 = 7 uppercase/special chars (< 'a') at the end (rows 19..25) so that
    #   C['a'] = 8 (Wait: we want LF(1)=4, LF(3)=6! Since 'a' is at rows 1,2,3, LF(1)=C['a']+0, LF(2)=C['a']+1, LF(3)=C['a']+2!
    #   Wait: to have LF(1)=4, LF(2)=5, LF(3)=6, we need C['a'] == 4 -> EXACTLY 4 chars < 'a' in L!
    #   Then LF(4) = C['b']+0 = 7 ('c'), LF(6) = C['b']+1 = 8 -> oops, row 8 is 'w', NOT 'c'!
    #   AHA!! Look why rows 7 and 8 had to be 'c' in round-2's witness: LF maps the two 'b's to CONSECUTIVE rows C['b'] and C['b']+1!
    #   Wait: so ONLY the FIRST symbol ('b') can be split by a non-'b' row (because 'a' had 3 copies: row 1 -> 4, row 2 -> 5, row 3 -> 6,
    #   skipping row 5!). To split 'c' too, we just put THREE 'b's (4,5,6? no, 'b' at 4, 'b' at 5, 'b' at 6 -> then 'b' isn't width-1!).
    #   HOLD ON!! Look at that: to split 'c' at rows 7 and 9, we need THREE 'b's (rows 4, x, 6, y, ... — three singleton 'b's!)
    #   mapped to rows 7, 8, 9 (so row 7='c', row 8='w', row 9='c')!
    #   Check! Put 3 singleton 'b's, 3 singleton 'c's, 3 singleton 'd's, ... — NO, then only 2 of the 3 share 'a' and the 3rd shares something else!
    #   WAIT A SECOND! That works!!
    #   Suppose we have 3 singleton 'a's? No, put:
    #     S = {r1, r2} both have 'a' -> map to 2 singleton 'b's (separated by a 3rd 'a' mapping to a separator between the two 'b's),
    #     AND put a 3rd 'b' (between them? No, a 3rd 'b' BEFORE or BETWEEN? Between them so LF(b1)=c1, LF(b_mid)=sep, LF(b2)=c2!)
    #   Wait: if b_mid is between b1 and b2, then b1, sep, b_mid, sep, b2 are ALL THREE width-1 'b's, AND their LF images are c1, sep, c2!
    #   Let's build it for depth 4 ('a' -> 'b' -> 'c' -> 'd' -> 'e') so 2 * 4 = 8 interior width-1 rows are discharged by ONE E1 class fact!
    #   Rows:
    #     Need 4 chars < 'a' -> C['a'] = 4.
    #     Wait: let's just make a tiny helper to find a valid single-cycle L where 2 starts share w="abcde" and ALL 2*4 = 8 interior rows are width-1!
    #   Layout of F-buckets (total n):
    #     [0..4): 4 special chars (< 'a') -> let's put 4 chars < 'a' anywhere!
    #     Wait: if C['a'] = 4, and 'a' occurs 3 times (at 1, 2, 3 -> LF = 4, 5, 6), then:
    #       row 4='b', row 5='b' (NO! row 5 = 'v' so row 4 and 6 are width-1 'b's!), row 6='b'.
    #     WAIT: if we want 'b' to occur 3 times AND all 3 'b's to be width-1, put 'b' at 4, 6, 8? No, we need b1=4, b_mid=5? No, b_mid at 5 would touch 4 and 6!
    #     Put 'a' 5 times? No, put 'a' at 1, 3, 5 (wait: if 'a's are at 1, 3, 5, they don't even have to be adjacent, they can be width-1 too!!):
    #       C['a'] = 4 -> LF(a1)=4, LF(a2)=5, LF(a3)=6, LF(a4)=7, LF(a5)=8!
    #       Put: 4='b', 5='v1', 6='b', 7='v2', 8='b'!!
    #       Then b1=4 (w1!), b2=6 (w1!), b3=8 (w1!)!
    #       And their LF images are C['b'], C['b']+1, C['b']+2 = 9, 10, 11!
    #       Put: 9='c', 10='v3', 11='c'!! Then c1=9 (w1!) and c2=11 (w1!)!
    #       Now path from a1: a1 -> 4('b') -> 9('c') -> ...
    #       and path from a5: a5 -> 8('b') -> 11('c') -> ...
    #       BOTH 4, 8 ('b') AND 9, 11 ('c') are width-1!!
    #       Even better: add 2 more 'b's so we get 5 'b's -> 3 'c's (all w1!) -> 2 'd's (both w1!)!
    #       Wait: with 3 'a's (a1, a2, a3) -> 4('b'), 5('v'), 6('b'), wait: 2 'b's only give 2 consecutive rows C['b'], C['b']+1.
    #       With 5 'a's -> 3 width-1 'b's (4, 6, 8) -> 2 width-1 'c's (9, 11)!
    #       That ALREADY gives 4 interior width-1 rows (4, 8, 9, 11) + 2 start width-1 rows (a1, a5) = 6 width-1 rows on S={a1, a5}
    #       discharged in 1 E1_derive + 2 E1_links = 3 events -> SAVING 3 events, so TOTAL c1 <= h1 - 1 (even if every wide row paid a link)!
    #   Let's construct an exact valid single-cycle instance of this and verify byte-exact T1!
    #   Layout: 16 rows total (0..15):
    #     Symbols < 'a': 4 distinct symbols '$', 'A', 'B', 'C' -> C['a'] = 4.
    #     'a' count = 5 -> C['b'] = 9.
    #     'b' count = 3 -> C['c'] = 12.
    #     'c' count = 2 -> C['v'] = 14.
    #     'v' count = 1, 'w' count = 1 -> total 16? Wait we need 3 separators (at 5, 7, 10) -> let's use 'v', 'w', 'x' (3 separators)!
    #     Total chars: 4 (<'a') + 5 ('a') + 3 ('b') + 2 ('c') + 3 ('v','w','x') = 17!
    #     Place at fixed positions:
    #       4='b', 5='v', 6='b', 7='w', 8='b', 9='c', 10='x', 11='c'
    #     Remaining 9 positions {0,1,2,3, 12,13,14,15,16} hold: 5 'a's + 4 specials ('$', 'A', 'B', 'C')!
    #     If we place the 5 'a's at {0, 2, 12, 14, 16} and the 4 specials at {1, 3, 13, 15}:
    #     EVERY SINGLE ROW OF L IS WIDTH-1 (h1 = 17 = n)!!
    #     Let's find a permutation of {'$', 'A', 'B', 'C'} at {1, 3, 13, 15} giving a SINGLE 17-cycle!
    import itertools

    found = None
    for spec_perm in itertools.permutations(b"$ABC"):
        L_arr = bytearray(17)
        L_arr[0] = ord("a")
        L_arr[1] = spec_perm[0]
        L_arr[2] = ord("a")
        L_arr[3] = spec_perm[1]
        L_arr[4] = ord("b")
        L_arr[5] = ord("v")
        L_arr[6] = ord("b")
        L_arr[7] = ord("w")
        L_arr[8] = ord("b")
        L_arr[9] = ord("c")
        L_arr[10] = ord("x")
        L_arr[11] = ord("c")
        L_arr[12] = ord("a")
        L_arr[13] = spec_perm[2]
        L_arr[14] = ord("a")
        L_arr[15] = spec_perm[3]
        L_arr[16] = ord("a")
        L_cand = bytes(L_arr)
        lf_c = compute_lf(L_cand)
        # Check single 17-cycle
        vis = set()
        cur = 0
        while cur not in vis:
            vis.add(cur)
            cur = lf_c[cur]
        if len(vis) == 17:
            found = (L_cand, lf_c)
            break

    assert found is not None, "Failed to find single-cycle 17-row E1 witness"
    L_e1, lf_e1 = found
    T_e1, _, _ = orbit_maps(L_e1, 0, lf_e1)
    j1_e1 = w1_rows(L_e1)
    assert len(j1_e1) == 17  # ALL 17 rows are width-1!
    # Check the two 3-step paths from S = {0, 16}:
    # 0('a') -> 4('b') -> 9('c') -> 12
    # 16('a') -> 8('b') -> 11('c') -> 13
    p0 = [0, lf_e1[0], lf_e1[lf_e1[0]]]
    p16 = [16, lf_e1[16], lf_e1[lf_e1[16]]]
    assert p0 == [0, 4, 9] and p16 == [16, 8, 11]
    assert bytes(L_e1[r] for r in p0) == bytes(L_e1[r] for r in p16) == b"abc"
    # Run exact decoder with ONE E1 class fact on S={0, 16}, h=3, w="abc":
    out_e1 = [None] * 17
    x = 0
    step = 0
    e1_events = 1  # charge 1 for deriving the E1 class fact
    while step < 17:
        if x in (0, 16):
            e1_events += 1  # charge 1 link at start row x
            for off, b in enumerate(b"abc"):
                out_e1[(17 - 1 - (step + off)) % 17] = b
            x = lf_e1[lf_e1[lf_e1[x]]]
            step += 3
        else:
            e1_events += 1
            out_e1[(17 - 1 - step) % 17] = L_e1[x]
            x = lf_e1[x]
            step += 1
    assert x == 0 and bytes(out_e1) == T_e1
    # Count: 11 ordinary links + 2 E1 links + 1 E1_derive = 14 total events < 17 = h1 (with |B|=0)!
    assert e1_events == 14 and e1_events < len(j1_e1)
    print(
        f"  [3b] All-w1 E1 witness L={L_e1.decode()!r}, p=0 -> T={T_e1.decode()!r}: "
        f"n=17, h1=17 (all w1), |B|=0"
    )
    print(
        f"       One E1 class fact on S={{0,16}} (w='abc') discharges interior w1 rows {{4,9,8,11}} "
        f"with 0 rigid facts:"
    )
    print(
        f"       TOTAL c1 = 11 (links) + 2 (E1 links) + 1 (E1 derive) = {e1_events} < 17 = h1 - |B| (byte-exact T1 verified)"
    )


if __name__ == "__main__":
    verify_mechanism1_histogram_completion()
    verify_mechanism2_copy_write()
    verify_mechanism3_e1_class_fact()
