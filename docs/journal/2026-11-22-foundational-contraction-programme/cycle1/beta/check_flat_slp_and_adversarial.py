#!/usr/bin/env python3
"""
Check:
1. Flat-leaf (inline <= 8B / 16B) + output-backed memoized SLP expansion on all 6 authenticated blocks
   under IET-BSE (rarest_first_deg & most_freq_first).
2. Explicit adversarial valid-BWT witness for non-monotone T_self^* hidden-Theta(rows) risk!
3. Also measure exact bucket-size distribution (how many steps in L1d <= 1.5k pcs, L2 <= 10k pcs, > 10k pcs).
"""

import json
import sys
sys.path.insert(0, "/tmp/fcp-b")
import measure_iet_bse as mb


def measure_flat_memo_expansion(rules, root, inline_limit=16, copy_thresh=17):
    """
    If a nonterminal u has wlen(u) <= inline_limit, treat it as a SINGLE flat-leaf write
    (e.g., 16-byte SIMD store from a flat inline pool, built during contraction without tree descent!).
    If wlen(u) >= copy_thresh and u was already emitted once, emit ONE memcpy from output buffer!
    Otherwise descend into binary cat children!
    """
    wlen = {}
    stack = [(root, False)]
    while stack:
        u, done = stack.pop()
        if u in wlen:
            continue
        r = rules[u]
        if r[0] == 0:
            wlen[u] = 1
        elif r[0] == 1:
            if done:
                wlen[u] = wlen[r[1]] + wlen[r[2]]
            else:
                stack.append((u, True))
                stack.append((r[1], False))
                stack.append((r[2], False))
        else:
            if done:
                wlen[u] = wlen[r[1]] * r[2]
            else:
                stack.append((u, True))
                stack.append((r[1], False))

    memo = set()
    node_exp = 0
    memcpys = 0
    memcpy_bytes = 0
    flat_writes = 0
    flat_bytes = 0
    unique_flat_nodes = set()

    stack = [root]
    while stack:
        u = stack.pop()
        if u < 0:
            memo.add(-u - 1)
            continue
        wl = wlen[u]
        if wl <= inline_limit:
            flat_writes += 1
            flat_bytes += wl
            unique_flat_nodes.add(u)
            continue
        if wl >= copy_thresh and u in memo:
            memcpys += 1
            memcpy_bytes += wl
            continue
        r = rules[u]
        if r[0] == 1:
            node_exp += 1
            if wl >= copy_thresh:
                stack.append(-u - 1)
            stack.append(r[2])
            stack.append(r[1])
        else:
            node_exp += 1
            if wl >= copy_thresh:
                stack.append(-u - 1)
            for _ in range(r[2]):
                stack.append(r[1])

    assert flat_bytes + memcpy_bytes == wlen[root]
    return {
        "node_exp": node_exp,
        "memcpys": memcpys,
        "memcpy_bytes": memcpy_bytes,
        "flat_writes": flat_writes,
        "flat_bytes": flat_bytes,
        "unique_flat_nodes": len(unique_flat_nodes),
        "unique_flat_bytes": sum(wlen[u] for u in unique_flat_nodes),
        "total_emit_leaves": flat_writes + memcpys,
        "total_ops": node_exp + flat_writes + memcpys,
        "avg_emit_bytes": wlen[root] / (flat_writes + memcpys),
    }


def extract_slp_and_measure_flat():
    print("=== 1. FLAT-LEAF (<=8B / <=16B) + OUTPUT-BACKED MEMOIZED SLP EXPANSION (ALL 6 BLOCKS) ===")
    blocks = mb.load_authenticated_blocks()
    # We'll re-run rarest_first_deg quickly while keeping rules & root via a thin wrapper
    # Actually let's inspect directly by monkey-patching measure_slp_expansion to also compute flat!
    orig_meas = mb.measure_slp_expansion
    captured = {}

    def hook_meas(rules, root, thresholds=(4, 8, 16)):
        captured["f8"] = measure_flat_memo_expansion(rules, root, inline_limit=8, copy_thresh=9)
        captured["f16"] = measure_flat_memo_expansion(rules, root, inline_limit=16, copy_thresh=17)
        return orig_meas(rules, root, thresholds)

    mb.measure_slp_expansion = hook_meas

    tot_m16 = 0
    tot_f8_leaves = 0
    tot_f16_leaves = 0
    tot_f16_ops = 0

    for binfo, raw in blocks:
        fname = binfo["file"]
        runs = mb.rle_from_bytes(raw)
        _, m16, _ = mb.compute_open_m16(runs, binfo["primary_index"], limit=16)
        mb.run_iet_bse_instrumented(runs, binfo["primary_index"], order_mode="rarest_first_deg", verify_bytes=None)
        f8 = captured["f8"]
        f16 = captured["f16"]
        tot_m16 += m16
        tot_f8_leaves += f8["total_emit_leaves"]
        tot_f16_leaves += f16["total_emit_leaves"]
        tot_f16_ops += f16["total_ops"]
        print(f"  {fname:12s} (m16={m16:6d}): "
              f"Flat<=8: leaves={f8['total_emit_leaves']:6d} ({f8['total_emit_leaves']/m16:.2f}*m16, avg={f8['avg_emit_bytes']:4.1f}B) "
              f"[flat={f8['flat_writes']:5d}, cpy={f8['memcpys']:5d}, ops={f8['total_ops']:6d}={f8['total_ops']/m16:.2f}*m16] | "
              f"Flat<=16: leaves={f16['total_emit_leaves']:6d} ({f16['total_emit_leaves']/m16:.2f}*m16, avg={f16['avg_emit_bytes']:4.1f}B) "
              f"[flat={f16['flat_writes']:5d}, cpy={f16['memcpys']:5d}, ops={f16['total_ops']:6d}={f16['total_ops']/m16:.2f}*m16, pool={f16['unique_flat_bytes']//1024}KiB]")
    print(f"  SUM (6 blks) (m16={tot_m16:6d}): Flat<=8 leaves={tot_f8_leaves:6d} ({tot_f8_leaves/tot_m16:.2f}*m16) | "
          f"Flat<=16 leaves={tot_f16_leaves:6d} ({tot_f16_leaves/tot_m16:.2f}*m16), total_ops={tot_f16_ops:6d} ({tot_f16_ops/tot_m16:.2f}*m16)")
    mb.measure_slp_expansion = orig_meas


def test_adversarial_nonmono_self_iet():
    print("\n=== 2. ADVERSARIAL HIDDEN-Theta(rows) WITNESS FOR NON-MONOTONE T_self^* ===")
    # Construct a valid single-cycle BWT with CONSTANT r0 = O(1), where eliminating a bucket
    # creates a non-monotone 2-piece self-IET inside another bucket that forces Theta(n) steps!
    # Idea: Take two alternating symbols (a, b) with a 2-piece rotation / shifted cycle:
    # Actually: consider T = (0 2)^K (1 2)^K 3? Wait: in (0 2)^K (1 2)^K 3, 2 has 2 self-pieces after eliminating 0 and 1!
    # Even simpler: T = (0 2)^K (1 2)^K (0 2 1 2)... Let's test K = 16, 64, 256, 1024 on several clean families!
    def bwt_encode_bytes(T):
        n = len(T)
        # Fast O(n^2) only for small n, or suffix doubling for n<=10000
        # Use doubling suffix array!
        s = list(T)
        sa = list(range(n))
        rk = list(s)
        k = 1
        while k < n:
            sa.sort(key=lambda i: (rk[i], rk[(i + k) % n]))
            tmp = [0] * n
            for j in range(1, n):
                a, b = sa[j - 1], sa[j]
                tmp[b] = tmp[a] + (1 if (rk[a], rk[(a + k) % n]) != (rk[b], rk[(b + k) % n]) else 0)
            rk = tmp
            if rk[sa[-1]] == n - 1:
                break
            k <<= 1
        L = bytes(s[(i - 1) % n] for i in sa)
        p = sa.index(0)
        return L, p

    # Family A: Interleaved period-2 runs with a unique sentinel:
    # Wait: in T = (1 2)^K (1 3)^K 0, if 2 & 3 are eliminated first, 1 gets TWO monotone self-translations
    # shifted by 1 — wait: do they invert (non-monotone) or stay monotone?
    # To make T_self on bucket '1' NON-MONOTONE with Theta(K) bounces:
    # Make bucket '1' map top-half to bottom-half and bottom-half to top-half (a 2-piece rotation x -> x + s mod M)!
    # A Sturmian / rotation word on {1, 2} with shift K/3 has BWT with r0 = 2..4!
    # Wait: what about a 3-letter word where eliminating 2 leaves a 2-IET rotation on 1 with step ~1 (so Zorich DOESN'T fire because it alternates)?
    # Test Family 1: T = b"ab" * K + b"ac" * K + b"d" (a=1, b=2, c=3, d=0)
    # Test Family 2: T = (b"a" + b"b"*2) * K + (b"a" + b"c"*2) * K + b"d"
    # Test Family 3: Interleaved multi-block with M=8 unique tags + (b"M" + b"X")^K!
    import random
    for name, gen in [
        ("Alt_ab_ac", lambda K: b"ab" * K + b"ac" * K + b"!"),
        ("MultiTag_MX", lambda K: b"".join(
            bytes([65 + i]) + b"MX" * K + bytes([80 + i]) for i in range(8)
        )),
        ("MultiTag_M_runs", lambda K: b"".join(
            bytes([65 + i]) + b"M" * K + bytes([80 + i]) for i in range(8)
        )),
    ]:
        print(f"  Family [{name}]:")
        for K in [16, 64, 256]:
            T = gen(K)
            L, p = bwt_encode_bytes(T)
            runs = mb.rle_from_bytes(L)
            res_str = []
            for om in ("most_freq_first", "rarest_first_deg", "fvs_acyclic_order", "max_sym"):
                st = mb.run_iet_bse_instrumented(runs, p, order_mode=om, verify_bytes=T)
                nm_s = st["steps_nonmono_self"]
                m_s = st["steps_mono_self"]
                fin_s = st["final_rv_steps"]
                res_str.append(f"{om[:6]}:tot={st['total_all_steps']}(nm={nm_s},m={m_s},fin={fin_s},z={st['zorich_skips']})")
            print(f"    K={K:3d} n={len(T):4d} r0={len(runs):2d} | " + " | ".join(res_str))


if __name__ == "__main__":
    extract_slp_and_measure_flat()
    test_adversarial_nonmono_self_iet()
