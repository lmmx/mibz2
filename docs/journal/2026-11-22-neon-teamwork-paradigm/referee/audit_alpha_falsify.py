#!/usr/bin/env python3
"""Deep falsifier for Alpha's Lazard-BWT:
1. Test whether max_live_r <= r_0 can be VIOLATED (valid BWT and arbitrary column).
2. Test whether BOTH mode can be forced to Omega(n) steps & symbols even when r << n (middle-delimiter attack).
3. Verify exactness and stats on all 6 Wikidata blocks.
"""
import itertools
import random
import sys

sys.path.insert(0, "/tmp/teamwork-a")
import verify_full_and_falsify as va


def test_r_monotonicity():
    print("=== 1. TESTING ALPHA'S CLAIM: max_live_r <= r_0 ===")
    violations_bwt = []
    violations_arb = []
    # Check all binary strings n <= 10 (valid BWT) and ternary strings n <= 6 (valid BWT)
    for n in range(2, 11):
        for mask in range(1 << n):
            T = bytes(((mask >> i) & 1) + 97 for i in range(n))
            L, p = va.bwt_encode(T)
            for mode in ("min", "max", "both"):
                got, st = va.lazard_decode(L, p, mode=mode, measure=True)
                assert got == T
                if st["max_live_r"] > st["r"]:
                    violations_bwt.append((mode, T, st["r"], st["max_live_r"]))
                    if len(violations_bwt) >= 5:
                        break
            if len(violations_bwt) >= 5:
                break
        if len(violations_bwt) >= 5:
            break

    # Check ternary valid BWTs n <= 6 and random valid BWTs over larger alphabets!
    rng = random.Random(42)
    for trial in range(500):
        n = rng.randint(10, 80)
        sig = rng.choice([3, 4, 8, 16, 32])
        T = bytes(65 + rng.randrange(sig) for _ in range(n))
        L, p = va.bwt_encode(T)
        for mode in ("min", "max", "both"):
            got, st = va.lazard_decode(L, p, mode=mode, measure=True)
            assert got == T
            if st["max_live_r"] > st["r"]:
                violations_bwt.append((mode, f"rand(n={n},sig={sig})", st["r"], st["max_live_r"]))

    # Check arbitrary ternary columns n <= 6
    for n in range(2, 7):
        for tup in itertools.product((97, 98, 99), repeat=n):
            L = bytes(tup)
            for mode in ("min", "max", "both"):
                _, st = va.lazard_decode(L, 0, mode=mode, measure=True)
                if st["max_live_r"] > st["r"]:
                    violations_arb.append((mode, L, st["r"], st["max_live_r"]))
                    if len(violations_arb) >= 5:
                        break
            if len(violations_arb) >= 5:
                break
        if len(violations_arb) >= 5:
            break

    print(f"  Valid BWT violations of max_live_r <= r_0: {len(violations_bwt)}")
    for v in violations_bwt[:5]:
        print("   ", v)
    print(f"  Arb col violations of max_live_r <= r_0:   {len(violations_arb)}")
    for v in violations_arb[:5]:
        print("   ", v)


def test_middle_delimiter_blowup():
    print("\n=== 2. ADVERSARIAL BLOW-UP ATTACK ON ALPHA (ALL MODES: min, max, both) ===")
    # Construct valid BWTs with r << n where BOTH min and max peel a middle-alphabet run one-by-one!
    # Alphabet: 'A'..'H' (low, 65..72), 'M' (middle run symbol, 77), 'S'..'Z' (high, 83..90)
    # Build M_cnt blocks: each block has a unique low tag (2 chars from A..H), then 'M'^K, then unique high tag (2 chars from S..Z)!
    rng = random.Random(123)
    for K in [4, 16, 64]:
        M_cnt = 64
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        rng.shuffle(low_tags)
        rng.shuffle(high_tags)
        chunks = []
        for i in range(M_cnt):
            chunks.append(low_tags[i] + bytes([77] * K) + high_tags[i])
        T = b"".join(chunks)
        L, p = va.bwt_encode(T)
        print(f"  Middle-run K={K:2d}, n={len(T):5d}:")
        for mode in ("min", "max", "both"):
            got, st = va.lazard_decode(L, p, mode=mode, measure=True)
            assert got == T
            r = st["r"]
            print(f"    [{mode:4s}] r={r:4d} (r/n={r/len(T):.4f}) | rounds={st['rounds']:4d} steps={st['steps']:5d} "
                  f"tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:5.2f}r) "
                  f"max_live_r={st['max_live_r']:4d} ({st['max_live_r']/r:.2f}r) "
                  f"ux={st['ux_runs']:5d} ({st['ux_runs']/r:5.2f}r) h={st['slp_height']:4d}")


def test_wikidata_alpha():
    print("\n=== 3. RE-MEASURING ALPHA ON ALL 6 WIKIDATA BLOCKS ===")
    va.run_wikidata()


if __name__ == "__main__":
    test_r_monotonicity()
    test_middle_delimiter_blowup()
    test_wikidata_alpha()
