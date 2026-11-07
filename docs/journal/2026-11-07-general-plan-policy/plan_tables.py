"""The plan tables of docs/journal/2026-11-07-general-plan-policy.md.

Pure arithmetic: no decoding, no timing. Each rule is a function of a machine's topology
and the threads asked for, written out as the journal specifies it, so its consequences can
be read for machines nobody has measured. `today` mirrors `plan_with` in src/parallel.rs at
4aa67e79 (0.2.6); `rule_a` and `rule_b` are the journal's candidates.

    python3 plan_tables.py            # every table, as Markdown
"""

MIB = 1 << 20
FULL_TABLE = 3.6e6  # a full block's T-vector: 900,000 rows x 4 bytes
COOP_MIN_CORES = 6


def show(plan):
    kind, *rest = plan
    if kind == "workers":
        return f"{rest[0]} workers"
    t, k, piped = rest
    return f"{t}×{k}" + (" pipelined" if piped else "")


def today(n, phys, l3):
    """plan_with at 0.2.6, with StreamDecoder::defaulted's pipelining (3 x 3 only)"""
    if phys is None:
        return ("workers", n)
    c = min(n, phys)
    if c < COOP_MIN_CORES:
        return ("workers", n)
    if phys == 10 and l3 == 20 * MIB and n >= 10:
        return ("teams", 3, 3, True)
    cap = 4 if l3 is None else max(1, round(0.7 * l3 / FULL_TABLE))
    return ("teams", max(1, min(c // 2, cap)), 2, False)


# Rule A: the two-budget rule
M_FAST = 6e6  # L3 bytes per fast team (F / 0.6)


def rule_a(n, phys, l3, m_fast=M_FAST, shared=True):
    if phys is None:
        return ("workers", n)
    c = min(n, phys)
    if c < COOP_MIN_CORES:
        return ("workers", n)
    t_wide = 4 if l3 is None else max(1, round(0.7 * l3 / FULL_TABLE))
    t_b = max(1, min(c // 2, t_wide))
    if l3 is not None and shared:
        t_cap = int(l3 // m_fast)
        t_f = min(t_cap, (c - 1) // 3)
        if t_b > t_cap and 3 * t_f >= 2 * t_b:
            return ("teams", t_f, 3, True)
    return ("teams", t_b, 2, False)


# Rule B: the fitted throughput model (per-table MB/s against the tables' share of the L3,
# from the i9's team-size grid, 2026-11-06)
I9_L3 = 20 * MIB
POINTS = [(2, 571), (3, 463), (4, 320), (5, 230), (6, 175), (8, 110)]
POINTS = [(t * FULL_TABLE / I9_L3, r) for t, r in POINTS]
PIPE_GAIN = {2: 1.0, 3: 1.072, 4: 1.072}


def per_table(x):
    if x <= POINTS[0][0]:
        return POINTS[0][1]
    for (x0, r0), (x1, r1) in zip(POINTS, POINTS[1:]):
        if x <= x1:
            return r0 + (r1 - r0) * (x - x0) / (x1 - x0)
    x1, r1 = POINTS[-1]
    return r1 * x1 / x


def rule_b(n, phys, l3, tie=0.02):
    if phys is None or l3 is None:
        return rule_a(n, phys, l3)  # its fallback
    c = min(n, phys)
    if c < COOP_MIN_CORES:
        return ("workers", n)
    best = []
    for k in (2, 3, 4):
        for t in range(1, c // k + 1):
            q = t * per_table(t * FULL_TABLE / l3) * PIPE_GAIN[k]
            best.append((q, t * k, t, k))
    top = max(q for q, *_ in best)
    q, threads, t, k = min((b for b in best if b[0] >= top * (1 - tie)), key=lambda b: (b[1], -b[0]))
    return ("teams", t, k, k >= 3)


def table(rows, header):
    out = ["| " + " | ".join(header) + " |", "|" + "---|" * len(header)]
    out += ["| " + " | ".join(map(str, r)) + " |" for r in rows]
    return "\n".join(out)


def main():
    print("## The i9-10900K (10 cores, 20 MiB L3)\n")
    rows = []
    for n in range(4, 21):
        p = (n, 10, 20 * MIB)
        rows.append((n, show(today(*p)), show(rule_a(*p)), show(rule_b(*p))))
    print(table(rows, ["threads", "today (0.2.6)", "rule A", "rule B"]))

    print("\n## CPU masks on the i9 (one CPU of each of c cores; the L3 stays 20 MiB)\n")
    rows = []
    for c in (4, 6, 8, 9):
        for n in sorted({c, 2 * c, 20}):
            p = (n, c, 20 * MIB)
            rows.append((c, n, show(today(*p)), show(rule_a(*p)), show(rule_b(*p))))
    print(table(rows, ["cores in mask", "threads", "today", "rule A", "rule B"]))

    print("\n## CI's runners (4 CPUs on 2 cores; macOS and Windows: topology unread)\n")
    rows = []
    for n in (1, 2, 4):
        rows.append((n, show(today(n, 2, 32 * MIB)), show(rule_a(n, 2, 32 * MIB)), show(rule_b(n, 2, 32 * MIB))))
    print(table(rows, ["threads", "today", "rule A", "rule B"]))

    print("\n## Hypothetical parts, all their cores asked for (threads = 2 x cores)\n")
    rows = []
    for cores in (6, 8, 10, 12, 16, 32):
        for l3 in (8, 12, 16, 20, 32, 64):
            p = (2 * cores, cores, l3 * MIB)
            a, t, b = show(rule_a(*p)), show(today(*p)), show(rule_b(*p))
            rows.append((cores, f"{l3} MiB", t, a + ("" if a == t else " ←"), b + ("" if b == t else " ←")))
    print(table(rows, ["cores", "L3", "today", "rule A", "rule B"]))

    print("\n## Rule A's one fitted constant: L3 per fast team\n")
    rows = []
    for m in (5.3e6, 6e6, 6.99e6):
        r = []
        for cores, l3 in ((10, 20), (12, 20), (8, 16), (16, 32), (32, 64)):
            r.append(show(rule_a(2 * cores, cores, l3 * MIB, m_fast=m)))
        rows.append((f"{m / 1e6:.2f} MB", *r))
    print(table(rows, ["L3 per fast team", "10 c, 20 MiB", "12 c, 20 MiB", "8 c, 16 MiB", "16 c, 32 MiB", "32 c, 64 MiB"]))


if __name__ == "__main__":
    main()
