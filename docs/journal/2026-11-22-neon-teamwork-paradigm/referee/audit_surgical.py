#!/usr/bin/env python3
"""Surgical final verification script:
1. Force Alpha's Lazard-BWT (ALL 3 modes: min, max, AND both!) into Omega(n) steps & Omega(n) symbols even when r << n.
2. Construct larger valid BWT counterexamples to BDMR's r_t <= r_0 (both Variant B greedy and B-Parallel).
3. Verify Beta's Variant B (greedy) claimed numbers (25-26 rounds, 4.06-4.34 r_0, max_r=1.00x) on all 6 Wikidata blocks.
4. Trace minimal binary BWT counterexample T=[1,0,1,0,0] for BDMR r_t > r_0.
"""
import hashlib
import json
import os
import random
import sys

sys.path.insert(0, "/tmp/teamwork-a")
import verify_full_and_falsify as va
sys.path.insert(0, "/tmp/teamwork-b")
import deep_analysis as db


def attack_alpha_all_modes():
    print("=== 1. FORCING ALPHA (min, max, AND both!) TO Omega(n) SYMBOLS & STEPS WITH r << n ===")
    # Key insight: BOTH elim_min(x) and elim_max(x) contract y x^k (attaching x to its LEFT neighbor y!).
    # So if M (middle symbol) has runs M^K followed by BOTH low_tags (< M) and high_tags (> M):
    #   T = concat_i ( M^K low_tag_i  M^K high_tag_i )
    # Then low_tag_i sits to the RIGHT of M^K (peels M^K one M at a time via elim_min!)
    # AND high_tag_i sits to the RIGHT of M^K (peels M^K one M at a time via elim_max!)
    rng = random.Random(31415)
    for K in [4, 16, 64]:
        M_cnt = 32
        low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
        high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
        rng.shuffle(low_tags)
        rng.shuffle(high_tags)
        chunks = []
        for i in range(M_cnt):
            chunks.append(bytes([77] * K) + low_tags[i] + bytes([77] * K) + high_tags[i])
        T = b"".join(chunks)
        L, p = va.bwt_encode(T)
        print(f"  Symmetric-Middle-Run K={K:2d}, n={len(T):5d}:")
        for mode in ("min", "max", "both"):
            got, st = va.lazard_decode(L, p, mode=mode, measure=True)
            assert got == T
            r = st["r"]
            print(f"    [{mode:4s}] r={r:4d} (r/n={r/len(T):.4f}) | rounds={st['rounds']:4d} steps={st['steps']:5d} "
                  f"tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:5.2f}r) "
                  f"ux={st['ux_runs']:5d} ({st['ux_runs']/r:5.2f}r) max_r={st['max_live_r']:4d} h={st['slp_height']:4d}")


def attack_beta_r_blowup():
    print("\n=== 2. MINIMAL & LARGE VALID-BWT COUNTEREXAMPLES TO r_t <= r_0 IN BDMR ===")
    # Trace minimal binary counterexample T = [1, 0, 1, 0, 0]
    T_min = [1, 0, 1, 0, 0]
    L_min, p_min = db.bwt_forward(T_min)
    got, st = db.decode_recomp(L_min, p_min, variant="B", collect_metrics=True)
    assert got == T_min
    print(f"  Minimal binary BWT: T={T_min}, L={L_min}, p={p_min}, r0={st['r0']}, max_r={st['max_r']} ({st['max_r']/st['r0']:.2f}x)")
    print(f"    trace (round, kind, n, r, sigma, del, indep_r): {st['round_trace']}")

    # Try to amplify r_t / r_0 on larger valid BWTs:
    # Idea: Repeat Fibonacci-like or Sturmian blocks with distinct separator symbols, OR
    # construct BWT where many runs straddle bucket boundaries in round 0 AND Sigma_L splits!
    # Try: k copies of Fibonacci-like words with different alphabet pairs (2i, 2i+1)!
    # Wait: if T = fib(a_0, b_0) + fib(a_1, b_1) + ... + fib(a_{m-1}, b_{m-1}):
    # Each fib pair has r_0 ~ 2..3 and grows by 1.5x in the SAME round!
    def fib_word(a, b, target_len):
        x, y = [a], [a, b]
        while len(y) < target_len:
            x, y = y, y + x
        return y

    for m_pairs in [10, 40]:
        T_mfib = []
        for i in range(m_pairs):
            T_mfib.extend(fib_word(2 * i, 2 * i + 1, 55))
        L, p = db.bwt_forward(T_mfib)
        r0 = len(db.runs_from_seq(L))
        got, st = db.decode_recomp(L, p, variant="B", collect_metrics=True)
        assert got == T_mfib
        print(f"  Multi-Fib({m_pairs} pairs, n={len(T_mfib)}): r0={r0:4d}, max_r={st['max_r']:4d} ({st['max_r']/r0:.3f}x), "
              f"sum_r/r0={st['total_run_work']/r0:.2f}x, rounds={st['rounds']}")

    # Also test: many copies of T_min = [1, 0, 1, 0, 0] with distinct alphabets (2i, 2i+1)
    # repeated k times: ([2i+1, 2i, 2i+1, 2i, 2i] * 20)!
    for m_pairs in [10, 50]:
        T_m5 = []
        for i in range(m_pairs):
            T_m5.extend([2 * i + 1, 2 * i, 2 * i + 1, 2 * i, 2 * i] * 20)
        L, p = db.bwt_forward(T_m5)
        r0 = len(db.runs_from_seq(L))
        got, st = db.decode_recomp(L, p, variant="B", collect_metrics=True)
        assert got == T_m5
        print(f"  Multi-5cyc({m_pairs} pairs, n={len(T_m5)}): r0={r0:4d}, max_r={st['max_r']:4d} ({st['max_r']/r0:.3f}x), "
              f"sum_r/r0={st['total_run_work']/r0:.2f}x, rounds={st['rounds']}")

    # Also test: BDMR on Symmetric-Middle-Run K=64 (the exact string that killed Alpha!)
    rng = random.Random(31415)
    M_cnt = 32
    low_tags = [[(i // 8), (i % 8)] for i in range(M_cnt)]
    high_tags = [[10 + (i // 8), 10 + (i % 8)] for i in range(M_cnt)]
    rng.shuffle(low_tags)
    rng.shuffle(high_tags)
    T_sym = []
    for i in range(M_cnt):
        T_sym.extend([8] * 64 + low_tags[i] + [8] * 64 + high_tags[i])
    L, p = db.bwt_forward(T_sym)
    r0 = len(db.runs_from_seq(L))
    got, st = db.decode_recomp(L, p, variant="B", collect_metrics=True)
    assert got == T_sym
    print(f"  BDMR on Sym-Middle-Run K=64 (n={len(T_sym)}): r0={r0:4d}, max_r={st['max_r']:4d} ({st['max_r']/r0:.3f}x), "
          f"sum_r/r0={st['total_run_work']/r0:.2f}x, rounds={st['rounds']} (self={st['self_rounds']}), rules={st['total_rules']} ({st['total_rules']/r0:.2f}r), h={st['slp_height']}")


def verify_beta_greedy_wikidata():
    print("\n=== 3. VERIFYING BETA VARIANT B (GREEDY) ON ALL 6 WIKIDATA BLOCKS ===")
    with open("/tmp/mibz2-blocks/manifest.json") as f:
        manifest = json.load(f)
    for entry in manifest["blocks"]:
        path = os.path.join("/tmp/mibz2-blocks", entry["file"])
        L = open(path, "rb").read()
        assert hashlib.sha256(L).hexdigest() == entry["sha256"]
        p = entry["primary_index"]
        got, st = db.decode_recomp(L, p, variant="B", collect_metrics=True)
        # Verify against reference
        ref = va.bwt_decode_reference(L, p)
        assert bytes(got) == ref, f"MISMATCH on {entry['file']}!"
        r0 = st["r0"]
        print(f"  {entry['file']:12s} EXACT | rounds={st['rounds']:2d} (self={st['self_rounds']}) | "
              f"sum_r/r0={st['total_run_work']/r0:.2f}x | max_r/r0={st['max_r']/r0:.2f}x | "
              f"max_sig={st['max_sigma']:5d} | rules={st['total_rules']:6d} | h={st['slp_height']:2d}")


if __name__ == "__main__":
    attack_alpha_all_modes()
    attack_beta_r_blowup()
    verify_beta_greedy_wikidata()
