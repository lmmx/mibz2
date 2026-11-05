"""Route A cost model: master's room walk vs the radix-partitioned walk, EPYC 9V74 (Zen 4).

Inputs marked O are observed (pinned: callgrind 3.22 cache sim on block 1 of
wikidata-independent.bz2, or the archived CI walk sweep 41989596336); A are assumed
(vendor figures, not measured here); I are inferred (derived below).
Usage: python3 -I model.py
"""

F = 3.6947  # GHz, O: sweep 41989596336 header ("3.6947 GHz reported")
N = 900_000  # rows of block 1, O (out/rows = 1.0001 over the file, O)

# --- calibration from the CI sweep (walk + assembly alone, c/B at the reported clock), O
alone32 = 4.27 / F  # ns/B, room/32/L1/512
lane12, lane16 = 6.44 / F, 5.29 / F
four32 = [x / F for x in (8.65, 9.35, 9.67, 8.64)]  # four copies, cpu0..3

# I: at 12-16 lanes the walk is latency-bound (one prefetch a lane a round): L = lanes * t
L_eff = (12 * lane12 + 16 * lane16) / 2  # ns per T-vector access, averaged over L2 hits
# I: at 32 lanes it is not (24-64 lanes flat): the concurrency it sustains is M = L / t
M = L_eff / alone32

L2_HIT = 14 / F  # ns, A (Zen 4 L2 load-to-use ~14 cycles)
BW = 32 * F  # GB/s = B/ns, A (L2<-L3 fill bandwidth ~32 B/cycle a core)
IPC_ALONE, IPC_CORE = 4.0, 6.0  # A: sustained IPC alone; the core's dispatch width, shared under SMT

# master, O (callgrind, LL = 1 MB 8-way as the L2): per byte
m = dict(
    ir=(9_407_174 + 178_852) / N,
    t_miss=661_690 / N,  # room_segments' read misses (the entry load is 618,803 of them)
    other_miss=(20_067 + 14_161 + 13_962) / N,
)
L_MISS = (L_eff - (1 - m["t_miss"]) * L2_HIT) / m["t_miss"]  # I: an L2 miss's mean service


def regimes(ir, occ_ns, lines, smt):
    """ns/B per walk: issue, miss-capacity, bandwidth; smt: two walks share the core"""
    issue = ir / ((IPC_CORE / 2 if smt else IPC_ALONE) * F)
    cap = occ_ns / M * (2 if smt else 1)
    bw = lines * 64 / BW * (2 if smt else 1)
    return issue, cap, bw


def show(name, ir, occ, lines):
    for smt in (False, True):
        i, c, b = regimes(ir, occ, lines, smt)
        print(
            f"  {name:34s} {'SMT ' if smt else 'alone'}  issue {i:5.2f}  miss-capacity {c:5.2f}  "
            f"bandwidth {b:5.2f}  -> max {max(i, c, b):5.2f} ns/B"
        )


print(f"I: L_eff = {L_eff:.1f} ns, M = {M:.1f} misses in flight a core, L2 miss service = {L_MISS:.1f} ns")
print(f"O: alone {alone32:.3f} ns/B; four copies {min(four32):.2f}-{max(four32):.2f} ns/B a walk "
      f"({min(four32) / alone32:.2f}-{max(four32) / alone32:.2f}x)")
print(f"I: latency-bound floor at 32 lanes: {L_eff / 32:.2f} ns/B")

occ_m = 1.0 * L_eff  # every step one L1-missing access, mean service L_eff
show("master (room/32/L1/512)", m["ir"], occ_m, m["t_miss"] + m["other_miss"])

import json, sys

for path in sys.argv[1:]:
    d = json.load(open(path))
    # d: ir, t_access (T loads a byte), t_miss, q_l1 (queue L1 misses a byte), other_miss, lines
    occ = (d["t_access"] - d["t_miss"]) * L2_HIT + d["t_miss"] * L_MISS
    occ += d["q_l1"] * L2_HIT  # queue streams, best case: prefetched to L2, never to L3 latency
    occ += d["other_miss"] * L_MISS
    show(d["name"], d["ir"], occ, d["lines"])
