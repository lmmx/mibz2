#!/usr/bin/env python3
"""Adversarial Referee Audit of Investigator Alpha: Lazard-BWT."""
import itertools
import random
import sys
import traceback

sys.path.insert(0, "/tmp/teamwork-a")
import verify_full_and_falsify as va


def ref_cycle_word(L: bytes, p: int):
    """Exact primitive cycle word starting at p under FL (= reversed LF cycle from p),
    and full length-n orbit output."""
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
    cyc = []
    cur = p
    while True:
        cyc.append(L[cur])
        cur = LF[cur]
        if cur == p:
            break
    cyc_word = bytes(reversed(cyc))
    full = bytearray(n)
    cur = p
    for t in range(n - 1, -1, -1):
        full[t] = L[cur]
        cur = LF[cur]
    return cyc_word, bytes(full)


def audit_alpha_exhaustive_arbitrary():
    print("=== 1. AUDITING ALPHA: ALL ARBITRARY TERNARY COLUMNS n=1..6 x ALL p x {min, max, both} ===")
    for mode in ("min", "max", "both"):
        tested = 0
        crashes = []
        wrong_cycle = []
        wrong_len = []
        for n in range(1, 7):
            for tup in itertools.product((97, 98, 99), repeat=n):
                L = bytes(tup)
                for p in range(n):
                    tested += 1
                    cyc_word, full_word = ref_cycle_word(L, p)
                    try:
                        got, _ = va.lazard_decode(L, p, mode=mode)
                    except Exception as e:
                        if len(crashes) < 5:
                            crashes.append((L, p, repr(e)))
                        continue
                    # Check if got matches full_word (length n) or a power of cyc_word
                    if got != full_word:
                        if len(got) % len(cyc_word) == 0 and got == cyc_word * (len(got) // len(cyc_word)):
                            if len(wrong_len) < 5:
                                wrong_len.append((L, p, cyc_word, full_word, got))
                        else:
                            if len(wrong_cycle) < 5:
                                wrong_cycle.append((L, p, cyc_word, full_word, got))
        print(f"  mode={mode:4s}: tested={tested}, crashes={len(crashes)}, wrong_cycle={len(wrong_cycle)}, wrong_len_only={len(wrong_len)}")
        for item in crashes[:3]:
            print("    CRASH:", item)
        for item in wrong_cycle[:3]:
            print("    WRONG CYCLE:", item)
        for item in wrong_len[:3]:
            print("    LEN MISMATCH (returned cyc_word not len-n):", item)


def audit_alpha_quaternary_and_adversarial():
    print("\n=== 2. AUDITING ALPHA: ARBITRARY QUATERNARY & MULTI-CYCLE TORTURE ===")
    rng = random.Random(2026)
    for mode in ("min", "max", "both"):
        crashes = 0
        wrong_cycle = 0
        first_fail = None
        # Test 2000 random arbitrary 4-symbol and 8-symbol columns (mostly multi-cycle!)
        for trial in range(2000):
            n = rng.randint(2, 20)
            sig = rng.choice([3, 4, 6, 8])
            L = bytes(65 + rng.randrange(sig) for _ in range(n))
            p = rng.randrange(n)
            cyc_word, full_word = ref_cycle_word(L, p)
            try:
                got, _ = va.lazard_decode(L, p, mode=mode)
            except Exception as e:
                crashes += 1
                if first_fail is None:
                    first_fail = ("CRASH", L, p, repr(e))
                continue
            if not (len(got) > 0 and len(got) % len(cyc_word) == 0 and got == cyc_word * (len(got) // len(cyc_word))):
                wrong_cycle += 1
                if first_fail is None:
                    first_fail = ("WRONG", L, p, cyc_word, got)
        print(f"  mode={mode:4s}: 2000 random multi-cycle arb columns -> crashes={crashes}, wrong_cycle={wrong_cycle}")
        if first_fail:
            print("    FIRST FAIL:", first_fail)

    # Specifically test valid BWT with periodic strings where root_mult > 1!
    print("\n=== 3. AUDITING ALPHA: VALID PERIODIC BWTs & ADVERSARIAL WORST-CASE ROUNDS/WORK ===")
    periodic_cases = [
        b"aaaa",
        b"abab",
        b"ababab",
        b"abcabcabc",
        b"aabbaabbaabb",
        b"abac" * 10,
    ]
    for T in periodic_cases:
        L, p = va.bwt_encode(T)
        for mode in ("min", "max", "both"):
            got, _ = va.lazard_decode(L, p, mode=mode)
            assert got == T, f"Periodic BWT fail T={T} mode={mode}: got={got}"
    print("  [PASS] All valid periodic BWTs recovered with exact length n.")

    # Worst-case rounds/work attack on Lazard-BWT:
    # Construct valid BWTs where S_left and S_right have size 1 every round (e.g., Phi_BWT or balanced chains)
    sys.path.insert(0, "/tmp/teamwork-c")
    import verify_phi_bwt as vp
    M = 1000
    perm = list(range(M))
    rng.shuffle(perm)
    pi = [0] * M
    for i in range(M):
        pi[perm[i]] = perm[(i + 1) % M]
    T_int, _ = vp.build_T_pi(pi, b=16, K=16)
    T_bytes = bytes(65 + c for c in T_int)
    L_phi, p_phi = va.bwt_encode(T_bytes)
    for mode in ("min", "max", "both"):
        got, st = va.lazard_decode(L_phi, p_phi, mode=mode, measure=True)
        assert got == T_bytes
        r = st["r"]
        print(f"  Phi_BWT (n={st['n']}, r={r}) [{mode:4s}]: rounds={st['rounds']}, steps={st['steps']}, "
              f"fx={st['fx_runs']/r:.2f}r (nonseq={st['fx_nonseq']/r:.2f}r), "
              f"ux={st['ux_runs']/r:.2f}r (inplace={100*st['ux_inplace']/st['ux_runs']:.1f}%), "
              f"tot_syms={st['tot_syms']/r:.2f}r, h={st['slp_height']}")

    # Adversarial long-height SLP / O(r) rounds attack:
    # De Bruijn or alternating-alphabet BWT or random permutation on 200 symbols
    # Construct T = random permutation of 256 bytes repeated with perturbations
    T_perm = bytes(rng.sample(range(256), 256))
    L_pm, p_pm = va.bwt_encode(T_perm)
    for mode in ("min", "max", "both"):
        got, st = va.lazard_decode(L_pm, p_pm, mode=mode, measure=True)
        assert got == T_perm
        print(f"  Rand perm 256 [{mode:4s}]: rounds={st['rounds']}, steps={st['steps']}, h={st['slp_height']}, tot_syms={st['tot_syms']}")


if __name__ == "__main__":
    audit_alpha_exhaustive_arbitrary()
    audit_alpha_quaternary_and_adversarial()
