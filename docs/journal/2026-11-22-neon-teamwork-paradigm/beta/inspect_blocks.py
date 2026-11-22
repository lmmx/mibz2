#!/usr/bin/env python3
import hashlib
import json
import os
import sys
from collections import Counter

BLOCKS_DIR = "/tmp/mibz2-blocks"

def verify_and_inspect():
    with open(os.path.join(BLOCKS_DIR, "manifest.json")) as f:
        manifest = json.load(f)

    for entry in manifest["blocks"]:
        path = os.path.join(BLOCKS_DIR, entry["file"])
        with open(path, "rb") as f:
            L = f.read()
        sha = hashlib.sha256(L).hexdigest()
        assert sha == entry["sha256"], f"SHA mismatch for {entry['file']}"
        n = len(L)
        assert n == entry["n"]
        p = entry["primary_index"]

        # Counts and C
        counts = [0] * 256
        for b in L:
            counts[b] += 1
        sigma = sum(1 for c in counts if c > 0)
        C = [0] * 256
        tot = 0
        for c in range(256):
            C[c] = tot
            tot += counts[c]

        # Build maximal runs
        runs = []  # (start, len, byte)
        i = 0
        while i < n:
            b = L[i]
            j = i + 1
            while j < n and L[j] == b:
                j += 1
            runs.append((i, j - i, b))
            i = j
        r = len(runs)
        assert r == entry["r"], f"r mismatch: {r} vs {entry['r']}"

        # Run width stats
        w1 = sum(1 for _, w, _ in runs if w == 1)
        w2 = sum(1 for _, w, _ in runs if w == 2)
        w_le4 = sum(1 for _, w, _ in runs if w <= 4)
        w_le16 = sum(1 for _, w, _ in runs if w <= 16)
        rows_le16 = sum(w for _, w, _ in runs if w <= 16)
        rows_gt16 = n - rows_le16

        # Build LF array
        pos = C[:]
        LF = [0] * n
        for idx, b in enumerate(L):
            LF[idx] = pos[b]
            pos[b] += 1

        # Width-1 row membership
        is_w1 = bytearray(n)
        for st, w, _ in runs:
            if w == 1:
                is_w1[st] = 1

        # Transitions w1 -> w1
        w1_to_w1 = 0
        w1_from_w1 = 0
        w1_entries = 0  # w1 row whose FL (predecessor in LF) is not w1, i.e., start of LF-chain in w1
        for st, w, _ in runs:
            if w == 1:
                nxt = LF[st]
                if is_w1[nxt]:
                    w1_to_w1 += 1

        # Breakpoints of LF^k: count i in [0, n) where LF^k[(i+1)%n] != (LF^k[i] + 1) % n
        # Let's compute for k = 1, 2, 4, 8, 16, 32
        cur = LF[:]
        bp_counts = {}
        for step_pow in range(6):  # k = 1, 2, 4, 8, 16, 32
            k = 1 << step_pow
            bp = 0
            for idx in range(n - 1):
                if cur[idx + 1] != cur[idx] + 1:
                    bp += 1
            bp_counts[k] = bp + 1
            if step_pow < 5:
                cur = [cur[x] for x in cur]

        # Also check chain lengths inside w1 under LF
        # Note: in a permutation, any induced subgraph on a subset S is a union of directed paths and cycles.
        in_deg_w1 = bytearray(n)
        for st, w, _ in runs:
            if w == 1:
                nxt = LF[st]
                if is_w1[nxt]:
                    in_deg_w1[nxt] = 1
        chain_lens = []
        visited = bytearray(n)
        for st, w, _ in runs:
            if w == 1 and not in_deg_w1[st]:
                length = 0
                x = st
                while is_w1[x]:
                    visited[x] = 1
                    length += 1
                    x = LF[x]
                chain_lens.append(length)
        # Any remaining are pure w1 cycles
        w1_cycle_nodes = sum(1 for st, w, _ in runs if w == 1 and not visited[st])
        avg_chain = sum(chain_lens) / len(chain_lens) if chain_lens else 0
        max_chain = max(chain_lens) if chain_lens else 0

        print(f"=== {entry['file']} (SHA OK) ===")
        print(f"  n={n}, p={p}, sigma={sigma}, r={r} ({r/n:.4f})")
        print(f"  runs: w=1: {w1} ({w1/r:.3f}), w=2: {w2}, w<=4: {w_le4}, w<=16: {w_le16}")
        print(f"  rows: w<=16: {rows_le16} ({rows_le16/n:.3f}), w>16: {rows_gt16} ({rows_gt16/n:.3f})")
        print(f"  w1->w1 under LF: {w1_to_w1}/{w1} ({w1_to_w1/w1:.3f}), chains={len(chain_lens)}, avg_len={avg_chain:.2f}, max_len={max_chain}, w1_cycle_nodes={w1_cycle_nodes}")
        print(f"  pieces of LF^k: " + ", ".join(f"k={k}:{v} ({v/n:.3f})" for k, v in bp_counts.items()))

if __name__ == "__main__":
    verify_and_inspect()
