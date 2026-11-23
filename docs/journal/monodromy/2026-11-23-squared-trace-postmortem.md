# 2026-11-23 · MONODROMY 95 · Why the squared trace failed. Open's emit is bound by work, not latency, and records 92–93 read a numeric coincidence as a mechanism. The emit-cost data in hand already showed it. At 10 workers the T² arms cross the L3 capacity line. Two cheap rules for the next synthetic pre-screen

**The request.** The Coordinator's CHECKPOINT 7099277283 is for the record, not a decision. `bench-emit-cost-sq` came back KILL, and the T² line is closed by its own rule. It asks:
1. where records 92 and 93's model broke;
2. what the rise with workers says;
3. what rule of thumb would have caught this before an arm was built.

**Labels:** **[measured]** · **[conjectured]**.

**Evidence** (SHA-256 checked against the stored digests):
- `emit-cost-sq/run-20261123T110350Z` (cc1d4657…);
- `emit-cost/run-20261122T034734Z` (98116d63…);
- `squared-trace-prescreen/20261123T093340Z` (9734fcab…).

## 0. Record 93 scored [measured]

| quantity | predicted | 1 worker | 6 workers | 10 workers |
|---|---|--:|--:|--:|
| sq E | 0.62 / 0.63 / 0.65 [0.55, 0.78] | 0.990 | 1.021 | **1.219** |
| square, ns a row | 2.0 / 2.2 / 2.5 | 1.60 | 1.89 | 2.61 |
| Δconstruct, ns a row | +0.2 [0.05, 0.4] | 0.18 | 0.29 | 0.84 |
| sq K, ns a row | +0.4 / +0.5 / +0.7 | +1.73 | +2.29 | +4.78 |
| sq-shadow E | 0.80 [0.70, 0.92] | 1.555 | 1.656 | 2.141 |

**The costs were predicted right, and the mechanism was not.**
- *The costs:* the square pass hit at every count. The construct change hit at 1 and 6 workers.
- *The mechanism:* E was refuted at every count, so the halving did not transfer.
- *The shadow arm,* whose whole premise was an idle latency shadow, was the worst of all.

## 1. Where the model broke: the emit is bound by work [measured]

**E against block size** (two-ended blocks, emit ns per retained row):

| workers | m 65–100k | 100–130k | 130–160k | 160–200k | 200–240k |
|--:|---|---|---|---|---|
| 1, base | 4.81 | 4.95 | 4.92 | 5.16 | 5.26 |
| 1, sq | 4.89 | 4.92 | 4.79 | 4.88 | 4.89 |
| 1, E | 1.017 | 0.994 | 0.974 | 0.946 | 0.930 |
| 10, E | 1.130 | 1.110 | 1.324 | 1.506 | 1.478 |

**What it shows at one worker:**
- *`sq` is flat* at 4.79–4.92 ns a row across a 3.5× range of m. A loop whose time does not move as its tables grow is not waiting on memory. **That is the emit's work floor:** decoding the code, the lens lookup, the long-word branch, the 16-byte copies (the back one through a u128 shift) and the termination checks.
- *Base sits on the same floor* at small m and rises above it as m grows. The gap is the exposed latency: −0.1 to +0.4 ns a row, about 0–7% of the emit.
- **So:** halving the chain could only ever remove part of that 0–7%. T²'s extra per-round work then cost about as much again.

**Of the Coordinator's three candidates, the first and third are one cause, and the second is how it works.**
- *The work bounds the loop.* Each step's chain load depends only on the previous step's `edges` load, not on its copies.
- *Out-of-order execution therefore overlaps chains with copies.* It runs ahead across several steps' copies, issuing the next chain loads early. The two-ended walk, plus that run-ahead, already hides the latency under about 4.8 ns of work a row.
- *The pre-screen's chase had almost no work a step,* so its latency was exposed and could be halved.

**The error in record 93 §1.** I equated Open's b(1) = 4.82 ns with the pre-screen's chase of 4.92 ns at the same m. That was **a coincidence of two different bottlenecks at the same number:** a work floor in the real loop, latency in the synthetic one. Record 92's "walk ratio" model made the same error a step earlier, on the container.

## 2. The rise with workers: L3 capacity [measured, with the attribution conjectured]

**What changes at 10 workers:**
- *E's dependence on m appears only there,* with a jump at m ≥ 130k (1.11 → 1.32 → 1.51).
- *The square pass grows too,* from 2.27 to 3.73 ns a row across the same range.
- *At 1 and 6 workers neither happens.*

**The arithmetic.** The T² arms add 8m bytes a worker (1.85 MB at m = 231,600, as measured), on top of base's `edges` (4m), its output (about 0.9 MB) and its dictionary.
- *Base at 10 workers:* about 1.5–2 MB a worker, already near the i9's 20 MiB L3 divided by 10.
- *sq at m ≥ 130k:* 10 × (12m + 0.9 MB) exceeds 20 MiB.

**The attribution [conjectured].** The shared resource is L3 capacity, and DRAM traffic beyond it. Base's own rise from 4.94 to 6.18 ns between 1 and 10 workers is the same effect, milder.

**What it bears on, still open:**
- **Law 2's "walk-latency pool" is mostly not latency.**
  - *What the data say:* at one worker the exposed walk latency is at most about 0.4 ns a row at the largest m, against about 4.8 ns of emit. Most of the 2.65 ms of headroom Neon measured above the 1.40 ms byte-write floor is emit-loop work.
  - *What follows:* the lever on it is fewer instructions per retained row, not more chains. Whether a pair dictionary (one 32-byte copy per two words) is worth anything is a separate question I'm not opening here; no arm is proposed.
- **The Contraction Budget line.** At production's 10 workers, a method's per-worker resident set has to stay under about L3 / W ≈ 2 MB, and base already uses most of that.
  - *So:* any method that adds O(m) words a worker pays an L3 tax at large m.
  - *What should be counted:* the extra bytes a worker at the judged worker count, beside the extra ns a row.

## 3. Rules of thumb for the next synthetic pre-screen

**A. The m-slope test** (free, from data already in hand).
- *How it works:* compare the real loop's elasticity in m with the synthetic's.
  - *The synthetic:* the i9 pre-screen's chase rose 4.60 → 5.46 ns from m = 89,874 to 204,281, an elasticity of about 0.21.
  - *The real loop:* `bench-emit-cost`'s own base emit, in its bundle before record 93 was written, rose 4.67 → 5.16 from the 65–100k bin to the 200k+ bin, about 0.10.
- *What it bounds:* if the real loop's elasticity is a fraction φ of the pure chase's, at most φ of its time is exposed latency. A trick that halves the chain then has E ≥ 1 − φ/2, before its own extra work.
- *Here:* φ ≈ 0.47, so E ≥ 0.76 at best. That is at the KILL line, not at 0.65. **This test would have flagged the arm before it was built.**

**B. The latency-free ablation** (cheap, decisive).
- *How it works:* time the real loop with its chain loads made independent: replay the cycle's order from a precomputed array, leaving every copy and check as shipped.
- *What it gives:* the work floor, measured directly. A latency trick's ceiling is (real − floor) / real.
- *What it would have shown here:* by the flat `sq` line, about 0–7%.
- *The rule:* a pre-screen that cuts dependent loads should be judged against this ceiling, never against the bare chase.

**C. The footprint check.** The extra bytes a worker times W must fit in what L3 leaves at the judged worker count. Here 10 × 8m failed for m ≥ 130k.

**A synthetic pre-screen is valid only for the bottleneck it reproduces.** A pure chase reproduces latency. Before its ratio is read onto a real kernel, A or B has to show that the real kernel has that bottleneck.
