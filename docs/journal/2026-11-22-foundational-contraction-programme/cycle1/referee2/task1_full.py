#!/usr/bin/env python3
"""Task 1: Exact MONODROMY Record 90 reconciliation on committed Lazard."""
import hashlib
import random
import sys

ALPHA_DIR = "/home/antigravity/lmmx/mibz2/docs/journal/2026-11-22-neon-teamwork-paradigm/alpha"
sys.path.insert(0, ALPHA_DIR)
import verify_full_and_falsify as va

# 0. Verify byte-for-byte identity of committed vs /tmp/teamwork-a
h1 = hashlib.sha256(open(f"{ALPHA_DIR}/verify_full_and_falsify.py", "rb").read()).hexdigest()
h2 = hashlib.sha256(open("/tmp/teamwork-a/verify_full_and_falsify.py", "rb").read()).hexdigest()
print(f"SHA256 committed verify_full_and_falsify.py: {h1}")
print(f"SHA256 /tmp/teamwork-a copy:                 {h2}")
assert h1 == h2

# 1. MONODROMY's test 1: audit_alpha_falsify.py (low + M^K + high, M_cnt=64, 2-byte tags)
print("\n=== 1. audit_alpha_falsify.py (low_2 + M^K + high_2, M_cnt=64) ===")
rng = random.Random(123)
for K in [4, 16, 64, 256]:
    M_cnt = 64
    low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
    high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
    rng.shuffle(low_tags)
    rng.shuffle(high_tags)
    T = b"".join(low_tags[i] + bytes([77] * K) + high_tags[i] for i in range(M_cnt))
    L, p = va.bwt_encode(T)
    got, st = va.lazard_decode(L, p, mode="both", measure=True)
    assert got == T
    r = st["r"]
    print(f"  K={K:3d} n={len(T):5d} r={r:3d} | both: tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:5.2f}r) rounds={st['rounds']:3d} steps={st['steps']:5d}")

# 2. MONODROMY's test 2: 1-char-tag variant of audit_alpha_falsify (low_1 + M^K + high_1, M_cnt=64 -> n=4224 at K=64)
print("\n=== 2. MONODROMY 1-char-tag variant (low_1 + M^K + high_1, M_cnt=64 -> n=4224 at K=64) ===")
rng = random.Random(123)
for K in [4, 16, 64, 256]:
    M_cnt = 64
    low_tags = [bytes([1 + i]) for i in range(M_cnt)]
    high_tags = [bytes([128 + i]) for i in range(M_cnt)]
    rng.shuffle(low_tags)
    rng.shuffle(high_tags)
    T = b"".join(low_tags[i] + bytes([77] * K) + high_tags[i] for i in range(M_cnt))
    L, p = va.bwt_encode(T)
    got, st = va.lazard_decode(L, p, mode="both", measure=True)
    assert got == T
    r = st["r"]
    print(f"  K={K:3d} n={len(T):5d} r={r:3d} | both: tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:5.2f}r) rounds={st['rounds']:3d} steps={st['steps']:5d}")

# 3. audit_surgical.py EXACT (M^K + low_2 + M^K + high_2, M_cnt=32 -> n=4224 at K=64)
print("\n=== 3. audit_surgical.py EXACT (M^K + low_2 + M^K + high_2, M_cnt=32 -> n=4224 at K=64) ===")
rng = random.Random(31415)
for K in [4, 16, 64, 128, 256]:
    M_cnt = 32
    low_tags = [bytes([65 + (i // 8), 65 + (i % 8)]) for i in range(M_cnt)]
    high_tags = [bytes([83 + (i // 8), 83 + (i % 8)]) for i in range(M_cnt)]
    rng.shuffle(low_tags)
    rng.shuffle(high_tags)
    T = b"".join(bytes([77] * K) + low_tags[i] + bytes([77] * K) + high_tags[i] for i in range(M_cnt))
    L, p = va.bwt_encode(T)
    print(f"  K={K:3d} n={len(T):5d}:")
    for mode in ("min", "max", "both"):
        got, st = va.lazard_decode(L, p, mode=mode, measure=True)
        assert got == T
        r = st["r"]
        print(f"    [{mode:4s}] r={r:3d} | rounds={st['rounds']:4d} steps={st['steps']:5d} tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:6.2f}r) ux={st['ux_runs']:5d} h={st['slp_height']:4d}")

# 4. Minimal 3-letter alphabet witness: T = M^K a M^K c (a < M < c), n = 2K + 2, r = 4!
print("\n=== 4. Minimal 3-letter witness: T = M^K + 'a' + M^K + 'c' (a < M < c, r = 4) ===")
for K in [4, 16, 64, 256, 1024]:
    T = b"M" * K + b"A" + b"M" * K + b"Z"
    L, p = va.bwt_encode(T)
    print(f"  K={K:4d} n={len(T):5d}:")
    for mode in ("min", "max", "both"):
        got, st = va.lazard_decode(L, p, mode=mode, measure=True)
        assert got == T
        r = st["r"]
        print(f"    [{mode:4s}] r={r:2d} | rounds={st['rounds']:4d} steps={st['steps']:5d} tot_syms={st['tot_syms']:5d} ({st['tot_syms']/r:7.2f}r) ux={st['ux_runs']:5d} h={st['slp_height']:4d}")
