# N1's construction economics: what the Ψᵏ builder must cost, and where it spends

**Status:** read-only analysis. Nothing was built, benchmarked or run on CI. Gate B (N1's
builder cycles) is not authorised, and this note doesn't propose running it. It sets out
what Gate B would have to show and why.

**Prior art:** N1's record partition is not new. It is Kärkkäinen, Kempa and Puglisi's
*s*-runs ("Slashing the Time for BWT Inversion", DCC 2012), mirrored from LF to Ψ
(portfolio §3.1, after (k)). Two things were not found in the literature: the per-visit
k-byte walk, and its run-native doubling construction. This note is about the cost of
that construction, and makes no novelty claim.

**Boundaries:** every figure is one thread, at the decoder-stage boundary (c/B of output),
unless marked otherwise. Each figure is labelled:
- **measured:** from census data, from this note's sample simulation, or from recorded CI
  and local runs;
- **model:** an estimate. Its assumptions are listed in §8.

## 1. Summary

1. **No single level dominates the builder. The per-parent work does.**
   - **Model:** in a P⁸ build, the three doublings take 25%, 30% and 33% of the
     instructions (1→2, 2→4, 4→8). Level 1 takes 9–12%, and the rank directory and caps
     about 2%.
   - **By operation:** per-parent work is about half of all instructions. That is the
     counting pass (28–31%) and setting up each parent in the merge (20–24%). Writing
     children is 29–30%.
   - **Bytes:** the merges are 57% of the bytes moved.
   - **Cost:** 9.8–18.2 instructions per output byte on the median block. That is 1.5–2.7×
     the shipped build's 6.65 (i9 profile). The p90 block takes 17–31.
2. **The deciding quantity is cycles per record element, c_e, not bytes.**
   - An element is a run, a parent, a child or an upgrade.
   - **Measured (census):** the elements per output byte are, on the median block,
     - P²: 0.20;
     - P^{2|4}: 0.28;
     - P⁴: 0.37;
     - P^{4|8}: 0.46;
     - P⁸: 0.59.

     They roughly double on the p90 block.
   - **Model:** c_e ≈ 6–16 cycles on the EPYC 7763.
   - **Measured anchor:** a run-native builder of the same kind (#42's move-structure
     build, untuned) cost about 40 cycles per element locally. That is about 16–27
     scaled to CI.
3. **Each doubling is worth less than the one before, and the full 4→8 doubling is
   modelled never to pay on Wikidata.**
   - **Doublings (measured, census):** each one costs the elements below per lookup it
     saves (median):
     - 1→2: 0.28;
     - 2→4: 0.68;
     - 4→8: 1.73.
   - **A cheaper last step:** the s = 1 cut of the refinement tree, called P^{4|8} here.
     It upgrades each level-4 record whose interval passes to level 8 unsplit, into a
     separate array, and builds no level-8 children. (Corrected: first written "in place",
     which is wrong; no processing order makes in-place work,
     `2026-10-30-n1-upgrade-correctness.md` §6.)
     - **Measured** on a 512-segment walk over 15 blocks: 0.150 lookups per byte, against
       0.125 for P⁸ and 0.250 for P⁴.
     - It costs 0.94 upgrades per lookup it saves.
   - **P⁸ against P^{4|8}:** P⁸ is better only if an element costs less than about 0.2–0.35
     of a lookup. On the 7763 that is below about 1.3–3.3 cycles, under the shipped build's
     2.2 cycles a row.
   - **The optimum:** P^{4|8} or P^{2|4}, depending on the ratio c_e/c_L.
4. **The break-even (model, median block).** N1 can spend up to the following on its
   build before it stops beating the shipped build plus walk:

   | Runner | Shipped build + walk | N1 build budget | Multiple of the shipped build |
   |---|--:|--:|--:|
   | EPYC 7763 | 8.85 c/B | 7.4–7.9 c/B | 3.4–3.7× |
   | Zen 4 | 8.89 c/B | 7.3–7.9 c/B | 3.9–4.2× |
   | Zen 5 | 5.70 c/B | 4.8–5.1 c/B | 3.1–3.4× |

   The budget column is for P^{4|8}. The walk's 1.7× spread across runners (4.18 to
   7.01 c/B) sets the budget. Zen 5 is the tightest.
5. **The proposed hypothesis** (§6): a tuned run-native builder reaches a median c_e of at
   most 10 cycles on the 7763 and Zen 4, and at most 6 on Zen 5.
   - It is falsified, and N1 at one thread closes, if c_e ≥ 22 (7763) or ≥ 14 (Zen 5).
     Then no plan gives even 10% on build plus walk, with the optimistic walk.
   - The smallest kernel that decides it is the P^{2|4} builder: level 1, one doubling and
     the upgrade.
6. **Negative findings** (§7):
   - P⁸ builds 78% of its records for nothing (measured);
   - the unsplit fast path is worth about 5% of the build (model);
   - building from the T-vector is dominated;
   - cooperative construction cuts wall, not CPU;
   - the portfolio's peak comparison leaves L out on N1's side.

## 2. What was measured here

`sample_sim.py` measures 15 of the 233 full blocks:
- census indices 0, 20, …, 220;
- the three highest-r blocks: 145 (r = 179,972), 90 and 33.

It is read-only Python, about 68 s on one core. Each block's m₁, m₂, m₄ and m₈ match the
census, up to the last-row label the census counts (§3.1(a)).

**The walk simulation:**
- **Segment starts:** step 0 plus 511 seeded pseudo-random traversal steps, the shipped
  walk's shape.
- **Hops:** k at a time, capped at each segment's end, so the caps of §3.1(g) are
  implied.
- **P^{a|2a}:** it hops 2a from a row whose level-a interval passes to level 2a unsplit,
  and a otherwise.

**Per doubling** (measured; median, with min–max over the 15 blocks):

| | 1→2 | 2→4 | 4→8 |
|---|--:|--:|--:|
| Split parents | 9.4% (6.4–10.1) | 6.1% (1.7–7.1) | 5.9% (1.3–6.6) |
| Rows lying in split parents | 40% (37–44) | 37% (35–40) | 34% (32–38) |
| Children from parents with ≥ 3 children | 15% | 18% | 24% |
| Children from the top 1% of parents | 11% | 14% | 18% |
| Most children, one parent | 409 (max 3,161) | 525 (max 3,439) | 265 (max 2,093) |

**Interval lengths** (measured):
- Singletons are 74% of level-4 intervals and 75% of level-8 intervals. On the high-r
  blocks they reach 98%.
- About 68% of rows lie in level-8 intervals of 64 rows or more.

**The walk** (measured; 512 segments; median, with min–max):

| Table | Lookups per byte | Its records the walk ever executes | Long hops |
|---|--:|--:|--:|
| P² | 0.500 | 60% (51–62) | — |
| P⁴ | 0.250 | 37% (26–39) | — |
| P⁸ | 0.1252 | 22% (13–24) | — |
| P^{2\|4} | 0.296 (0.293–0.312) | 38% (28–40) | 69% |
| P^{4\|8} | 0.150 (0.148–0.156) | 25% (14–27) | 67% |

**The reading:**
- **Unsplit parents are not a row majority.** They are 94% of parents but cover only
  about 66% of rows. The split ones are the long intervals that cross a burst.
- **Most of a static table is never read.** A walk lands on 1/k of the rows, and three
  quarters of records are single rows inside bursts. So most of the records the deeper
  levels add are never looked up.

## 3. The builder's cost, per level and per operation (model over measured counts)

### 3.1 Elements per output byte (measured, census, 233 blocks)

| Plan | min / p10 / median / p90 / max |
|---|---|
| P² (level 1, then 1→2) | 0.171 / 0.182 / 0.201 / 0.357 / 0.627 |
| P^{2\|4} (P², then upgrade the unsplit level-2 records) | 0.235 / 0.251 / 0.277 / 0.492 / 0.853 |
| P⁴ | 0.313 / 0.336 / 0.369 / 0.658 / 1.105 |
| P^{4\|8} | 0.391 / 0.422 / 0.462 / 0.822 / 1.358 |
| P⁸ | 0.492 / 0.533 / 0.586 / 1.035 / 1.651 |

Elements are r plus Σ(m_a + m_2a) over the doublings, plus the upgrades. The shipped
build, for comparison, writes 1 entry per output byte, scattered.

### 3.2 Traffic per level (model; layout C′, P⁸)

**Layout C′ is this note's reconstruction of layout C,** with every access enumerated:
- **Records:** 12 B construction records, and an index image view.
- **Counting:** strict-interior popcounts, with E as a running prefix.
- **The F prefix:** taken in image order.
- **The merge:** one pass, with groups scattered to their children.
- **The final level** is written straight in walk format: 12 B, {D, 8-byte word}.
- **Caps** are fused into the 4→8 merge.

**Median bytes per output byte:**

| Level | Count | F prefix | Merge | Share of all bytes |
|---|--:|--:|--:|--:|
| Level 1 (L scan 1.00, run emit 1.25) | — | — | — | 9% |
| 1→2 | 1.50 | 0.75 | 3.71 | 26% |
| 2→4 | 1.76 | 0.91 | 4.47 | 32% |
| 4→8 | 2.12 | — | 4.71 | 30% |
| Directory | — | — | — | 2% |

**Totals, per block (min / p10 / median / p90 / max):**
- **Layout C′:** 19.5 / 20.8 / **22.5** / 37.5 / 59.6 B/B.
- **With caps as a separate post-split pass:** 25.5 B/B at the median.
- **The portfolio's layout C** gives 17.0, and its dup/dup/idx mix 17.5. The difference is
  this reconstruction's explicit gathers (S_a[i] and E[i] through the index view) and its
  F prefix pass.

**Either way:**
- every array is at most about 2.6 MB at the median;
- the accesses are sequential, or are group-scattered within L2/L3.

So bytes are not what decides it. The instruction count per element is.

### 3.3 Instructions per level and per operation (model, P⁸, median block)

**The per-element instruction counts assumed** (low / high, for a tuned scalar
implementation):

| Element | Low | High |
|---|--:|--:|
| Level-1 run | 10 | 20 |
| L scan, per byte | 0.3 | 1.0 |
| Counting, per parent | 12 | 24 |
| F prefix, per parent | 5 | 8 |
| Merge, per parent | 10 | 16 |
| Merge, per child | 10 | 18 |
| Upgrade, per parent | 16 | 26 |
| Directory, per 64 rows | 6 | 10 |
| Caps, per segment start | 150 | 300 |

**Instructions per output byte, by level:**

| | Level 1 | 1→2 | 2→4 | 4→8 | Final | Total (p90; max) |
|---|--:|--:|--:|--:|--:|---|
| Low | 0.92 | 2.44 | 2.98 | 3.28 | 0.18 | **9.79** (16.96; 27.27) |
| High | 2.25 | 4.36 | 5.32 | 5.94 | 0.33 | **18.17** (31.13; 49.79) |

**By operation:**
- counting, per parent: 28–31%;
- merge, per child: 29–30%;
- merge, per parent: 20–24%;
- level 1: 9–13%;
- F prefix: 6–7%;
- directory and caps: 2%.

**Mispredicts:** at 1–2 per split parent, 0.016–0.032 per output byte. That is about
0.3–0.5 c/B at 16 cycles each.

**The other plans,** in instructions per output byte (low–high, median block):
- P²: 3.2–6.4;
- P^{2|4}: 4.5–8.4;
- P⁴: 6.1–11.6;
- P^{4|8}: 7.6–14.0.

The portfolio's §3.1(e) estimate for P⁴, about 7–15, agrees.

**So, which level dominates?** 4→8 is the largest single level, at about a third of the
operations and 30% of the bytes. It is also the level with the worst return (§4.1), and
most of what it adds is never read (§2). The largest operation class is the per-parent
work, counting plus merge set-up, at about half the instructions. Unsplit parents pay it
too, which is why a fast path for them saves little (§4, (c)).

### 3.4 The measured anchor: the move-structure build (#42; local VM; untuned)

#42's run-native build is close kin to N1's level 1 plus one merge: a run scan, then three
O(r) passes.
- **The passes:** F positions and piece counts; a scatter into 256 per-byte streams; a
  merge of images against interval starts.
- **The cost:** 6.2–6.8 c/B, against 2.7 for the shipped build, at r/n = 0.072, with
  0.079–0.102 intervals per row.
- **Per element:** that is about 39–41 cycles. The journal itself gives about 30–40 cycles
  per run per pass.
- **Scaled to CI:** that journal puts its local figures at 1.5–2.4× CI's, which makes this
  about 16–27 cycles per element.
- **What it shows:** if N1's passes cost what #42's untuned passes did, c_e lands in §5's
  inconclusive-to-kill band.
- **Why a tuned build might do better:** the #42 journal says its loops branched on run
  lengths and divided by the cap, and N1's needn't. That is an expectation, not a
  measurement.

## 4. Mechanisms compared

### 4.1 The marginal value of each step (measured counts)

A doubling a→2a pays only if one lookup costs more than this many elements
(c_L/c_e > ρ), where ρ is the elements it adds per lookup it saves:

| Step | ρ (min / p10 / median / p90 / max) |
|---|---|
| 1→2 | 0.23 / 0.25 / 0.28 / 0.49 / 0.85 |
| 2→4 | 0.57 / 0.62 / 0.68 / 1.20 / 1.92 |
| 4→8 | 1.42 / 1.58 / 1.73 / 3.08 / 4.36 |
| Upgrade P⁴ → P^{4\|8} (upgrades per lookup saved) | 0.78 / 0.85 / 0.94 / 1.66 / 2.52 |
| P^{4\|8} → P⁸ (the full 4→8 doubling in place of the upgrade) | 3.97 / 4.47 / 4.89 / 8.73 / 11.95 |

**The lower envelope** (model, median block) is the cheapest plan as a function of
x = c_e/c_L. An upgrade element is weighted u elements:
- **u = 1.0:** P⁸ for x < 0.25, P^{4|8} to 0.80, P^{2|4} to 2.70, then P².
- **u = 1.5:** P⁸ for x < 0.35, P^{4|8} to 0.75, P⁴ to 0.85, P^{2|4} to 1.80, then P².
- **u = 2.0:** P⁸ for x < 0.60, P⁴ to 1.50, then P².

**The model's x:**
- c_L = 16–24 instructions per lookup at the shipped walk's IPC. That is 6.3–9.5 cycles
  on the 7763.
- c_e ≈ 6–16 cycles.
- So x ≈ 0.6–2.5, which puts the optimum at P^{4|8}, P⁴ or P^{2|4}, and P⁸ off the
  envelope.

### 4.2 The mechanisms

All rows are model, for the median block on the 7763. Shipped build + walk = 8.85 c/B;
total 12.93.

| Mechanism | Build c/B | Walk c/B | Build + walk | One-core decoder | Peak MB (excl. L) / resident | What would falsify it |
|---|--:|--:|--:|--:|---|---|
| (a) Layout-C doubling to P⁸ | 3.5–9.6 | 0.79–1.19 | 4.3–10.8 (0.49–1.22×) | 0.65–1.15× | 3.79 (p90 6.40, max 9.07) / 1.54 | c_e > 13 cycles (break-even), or measured c_L < 5 c_e (P^{4\|8} better) |
| (b) Stop at P⁴ | 2.2–6.2 | 1.58–2.38 | 3.8–8.5 (0.43–0.96×) | 0.61–0.98× | 2.76 (4.55, 7.16) / 0.90 | dominated on the envelope unless u ≥ 1.5 and x ≈ 0.75–0.85 |
| (b) Stop at P² | 1.2–3.4 | 3.17–4.75 | 4.3–8.2 (0.49–0.92×) | 0.65–0.95× | 2.34 (3.81, 6.40) / 0.77 | best only if c_e > 1.8–2.7 c_L; then its walk saving is small |
| (f1) **P^{4\|8}**: P⁴, then the s = 1 upgrade | 2.8–7.5 | 0.94–1.48 | 3.7–9.0 (0.42–1.02×) | 0.60–1.01× | 2.76 (4.55, 7.16) / 1.24 | upgrade cost u > 2 elements, or c_e > 16 |
| (f2) **P^{2\|4}**: P², then the upgrade | 1.7–4.6 | 1.85–2.96 | 3.5–7.5 (0.40–0.85×) | 0.59–0.90× | 2.34 (3.81, 6.40) / 1.04 | c_e > 21 cycles (break-even) |
| (c) Unsplit fast path at 4→8 | −0.2 to −0.5 on (a) | 0 | about −5% of (a)'s build | — | unchanged | a profile showing the child-loop control above about 15% of the build |
| (d) Records from the T-vector | ≥ 2.16 + C₈ marking (about 1) + a chase of 8·m₈/n = 0.98 loads per byte (p90 1.74) | 0.8–1.2 | ≥ about 8–10 | ≈ 1.0× or worse | T-vector + P⁸ ≈ 5.1 | dominated by counting: the chase alone is about a walk's worth of dependent loads |
| (e) Cooperative build across a team | CPU unchanged, plus 2–5% of barriers (about 10 passes per block) | — | wall per block / K | none at `Plan::Workers` (4-core target) | shared | team build speed-up below 1.6× at K = 2 |
| (f3) Lazy or hot-row-only records | — | — | — | — | — | no hot rows exist: each row is visited once, and which rows the walk lands on is known only by walking |

**Notes:**
- **How the ranges are built.** The build range puts the low instruction counts with
  IPC 3.0, and the high ones with IPC 2.0 plus mispredicts. The walk range is 16–24
  instructions per lookup at each runner's implied walk IPC. The ranges are wide by
  construction.
- **Zen 4 and Zen 5** follow the same pattern (`model.py` prints them). P^{2|4} and
  P^{4|8} stay the best two. On Zen 5, P⁸ reaches 0.52–1.32× and P^{2|4} 0.41–0.89×.
- **(f1) and (f2) are the refinement-tree cut with s = 1,** from the portfolio's §3.1(g)1,
  costed as a construction.
  - An unsplit parent's image lies in one level-a interval J. So
    Ψ^{2a} = Ψ^a ∘ Ψ^a translates the whole parent by D_I + D_J, with the word
    w_I ‖ w_J. The upgraded record is exact over the whole parent.
  - The upgrade reads the record, finds J by a rank query on the level-a directory,
    gathers S_a[J] and writes a 12 B record with a length of 4 or 8.
  - It writes to a separate array, because S_a[J] may itself be upgraded.
  - It needs no counting pass, no E or F, and no child placement.
  - The walk already handles records of variable length (caps), so a lane stores 8 bytes
    and advances by len.
  - Correctness is argued here, not checked: no brute-force test of the upgrade was run.
- **(c), the fast path.** Unsplit parents are 94% of parents but 66% of rows. They still
  pay for counting, the merge set-up and placement. The saving is the per-child loop
  control: about 3–5 instructions on each of about 195,000 parents, which is 0.65–1.2
  instructions per byte. P^{4|8} is the structural form of the same observation.
- **(d), building from the T-vector:**
  - Finding C₈ is cheap. Breakpoints come from one sequential pass over Ψ. The preimages
    come from blitting bitvector ranges through L's runs, about (n/64 + 2r) word
    operations a step.
  - The records' D and words are what cost: each needs eight dependent T-vector loads.
  - Its peak also exceeds the T-vector's.
- **(e), cooperation.**
  - Every pass partitions: the counting pass by source order, the merge by image order,
    and the prefixes in two phases. Split by element counts, the largest parent (≤ 3,439
    children) doesn't unbalance K = 2.
  - It helps the teams plan (≥ 6 cores), where a block's wall matters. A smaller shared
    table also eases L3 pressure there, which is B1's effect.
  - It does nothing for CPU-seconds per GB.

## 5. Break-even

Break-even is build_N1 + walk_N1 = build_T + walk_T. The T-side figures are measured CI
stages; the N1 side is model.

### 5.1 The measured stage figures

| Runner | Scanner | MTF+RLE2 | Build | Walk | Total | Walk share | Walk IPC (16.9 instructions/B) |
|---|--:|--:|--:|--:|--:|--:|--:|
| EPYC 7763 (Zen 3) | 1.23 | 2.29 | 2.16 | 6.69 | 12.93 | 52% | 2.53 |
| EPYC 9V74 (Zen 4) | 1.18 | 2.23 | 1.88 | 7.01 | 12.80 | 55% | 2.41 |
| EPYC 9V45 (Zen 5) | 1.08 | 1.97 | 1.52 | 4.18 | 9.17 | 46% | 4.04 |

**How much each stage moves across runners:**
- the walk: 1.68× (4.18–7.01 c/B);
- the build: 1.42×.

**The two IPC columns are inferences.** They use the i9's instruction counts (16.9 for the
walk, 6.65 for the build). The build's would be 3.1 on the 7763, 3.5 on Zen 4 and 4.4 on
Zen 5. N1 trades walk for build, so it is worth least where the walk is cheapest, on Zen 5.

### 5.2 Builder budget = build_T + walk_T − walk_N1 (model walk, median block)

| Runner | P² | P^{2\|4} | P⁴ | P^{4\|8} | P⁸ |
|---|--:|--:|--:|--:|--:|
| EPYC 7763 | 4.1–5.7 | 5.9–7.0 | 6.5–7.3 | 7.4–7.9 | 7.7–8.1 |
| Zen 4 | 3.9–5.6 | 5.8–6.9 | 6.4–7.2 | 7.3–7.9 | 7.6–8.1 |
| Zen 5 | 2.7–3.7 | 3.8–4.5 | 4.2–4.7 | 4.8–5.1 | 5.0–5.2 |

As a multiple of the shipped build:
- P⁸ may cost 3.5–3.7× on the 7763, 4.1–4.3× on Zen 4 and 3.3–3.4× on Zen 5;
- P^{4|8} may cost 3.4–3.7×, 3.9–4.2× and 3.1–3.4×.

**The walk N1 must reach, given a build.** Inverting the budget: with a P^{4|8} build at
the model's midpoint (about 5 c/B on the 7763), the walk must stay under
8.85 − 5 = 3.85 c/B, which is 0.58× today's walk. A 1.25× one-core decoder gain needs
build + walk ≤ 6.26 c/B on the 7763.

### 5.3 Affordable cycles per element at break-even (model; conservative walk)

| Runner | P² | P^{2\|4} | P⁴ | P^{4\|8} | P⁸ |
|---|---|---|---|---|---|
| EPYC 7763 | 20.4 (11.5) | 21.3 (12.0) | 17.5 (9.8) | 16.0 (9.0) | 13.1 (7.4) |
| Zen 4 | 19.5 (11.0) | 20.9 (11.8) | 17.3 (9.7) | 15.9 (8.9) | 13.1 (7.4) |
| Zen 5 | 13.6 (7.7) | 13.9 (7.8) | 11.4 (6.4) | 10.3 (5.8) | 8.5 (4.8) |

Each cell is the median block, with the 10th-percentile block (high r) in brackets. The
highest-r block allows 3.0–6.9 cycles. The graded per-block selector exists for those
blocks.

### 5.4 Gate thresholds on c_e (model; best plan at each c_e; u = 1.5)

**The two thresholds:**
- **Pass:** the best plan reaches 0.75× of build_T + walk_T with the conservative walk
  (24 instructions per lookup) and an L scan of 0.6 c/B.
- **Kill:** even the optimistic walk (16 instructions per lookup, scan 0.3 c/B) can't
  reach 0.9×.

| Runner | c_L (cycles per lookup) | Pass if c_e ≤ | Kill if c_e ≥ | Four-copy model: pass if c_e ≤ |
|---|--:|--:|--:|--:|
| EPYC 7763 | 6.3–9.5 | 10.2 (P^{2\|4}) | 22.4 (P²) | 31 |
| Zen 4 | 6.6–10.0 | 9.9 (P^{2\|4}) | 21.8 (P²) | 31 |
| Zen 5 | 4.0–5.9 | 6.1 (P^{2\|4}) | 14.2 (P²) | 19 |

**Correction (`2026-10-30-n1-four-copy-sensitivity.md`):** the four-copy column is wrong.
`model.py` divided c_e by the N1 build factor instead of multiplying (31.2 corrected alone
is 24.0); the shipped build's four-copy factor was measured at ×1.26, not 1.14; the 5.0 →
10.4 c/B walk figure was probably from a 9V74, not the 7763; and α_N = 1.3 is below every
measured analogue. With measured factors, the four-copy pass needs c_e ≤ 10.3–16.0 on the
measured 7763 base. The column is kept below as first written.

**The four-copy column:**
- **What it rests on (measured):** CI's stage walk at four concurrent copies went from 5.0
  to 10.4 c/B (P111; a different run from the one in §5.1).
- **What it assumes (model):** the shipped build is ×1.14 at four copies, the N1 build
  ×1.14, and the N1 walk ×1.3. N1's table is 1.0–2.0 MB against 3.6, and it makes 1/7 to
  1/4 of the dependent accesses.
- **Why it matters:** this is N1's strongest case at the 4-core release target, where
  `Plan::Workers` runs four blocks at once.
- **Its weakness:** it is also the most assumption-laden figure here.

## 6. The proposed hypothesis

**H-B′ (the builder's cost per element).**
- **The scope:** CI's runners, one thread, the 233 full Wikidata bench blocks.
- **The claim:** a tuned run-native record builder processes its elements at a median
  c_e ≤ 10 cycles on the EPYC 7763 and Zen 4, and ≤ 6 on Zen 5.
- **The elements:** runs, parents, children and upgrades, counted exactly from each block's
  census.
- **The builder:** level 1 from L's runs, then layout-C doublings with strict-interior
  popcount counts.
- **The two predictions:**
  - the instruction model predicts 6–16 cycles on the 7763;
  - the #42 build, untuned, implies 16–27.

  The hypothesis can fail either way, which is what makes it worth testing.

**Why this hypothesis, and not "the P⁸ builder against the T-vector build":**
- **c_e is the one unmeasured quantity** that decides the plan (§4.1), the break-even
  (§5.3) and the kill (§5.4) together. The record counts are already measured exactly.
- **It needs the least code.** The P^{2|4} builder is level 1, one doubling and one
  upgrade pass. Deeper levels reuse the same passes on larger arrays.
- **It doesn't depend on the walk.** It can kill N1 at one thread without a walk being
  written, using the optimistic walk model. Only a pass needs Gate C's measured c_L.

**A pre-registered Gate B, sketched** (for the maintainer to decide; not proposed to run):
- **What it builds:** a standalone kernel bench over stored L columns of the bench blocks,
  as P74's gate does with `Symbols`. The production decoder is untouched.
  - **Arm N:** the P^{2|4} builder, with per-pass cycle counters for the L run scan,
    level-1 emit, counting, merge and upgrade. Optionally a second arm adds 2→4 and the
    4→8 upgrade (P^{4|8}), to see whether c_e rises once arrays leave L2.
  - **Control:** the shipped `bwt::build`, in the same process, in alternating order.
- **Exactness, on every block, untimed:**
  - every record's D and word equal the directly computed Ψ² and Ψ⁴ values;
  - a scalar walker over the records, with caps from the same 512 starts, reproduces
    `bwt::walk`'s output byte for byte.
- **What it reports:**
  - c_e per block, its median and p90, and c_e per pass;
  - build_N1 / build_T, paired;
  - the same run's measured build_T and walk_T on that runner;
  - u, the upgrade's cost relative to a merge element.
- **The decision rule.** It is recomputed from the same run's build_T and walk_T, so the
  runner's own walk sets the bar:
  - **Pass to Gate C:** the median c_e is at most the pass threshold of §5.4 for that
    runner, on every runner measured, with its 95% interval below it.
  - **Kill:** the median c_e is at least the kill threshold on every runner measured.
    N1 at one thread then closes as class II, the budget measured. The four-copy route
    stays open only through its own pre-registered four-copy run.
  - **Otherwise inconclusive:** recorded, with no further one-thread N1 work. The plan
    choice follows the envelope (§4.1) with the measured c_e and u.
  - **In every case:** P⁸ goes forward only if Gate C measures c_L > 5·c_e.

## 7. Negative findings

1. **P⁸'s full 4→8 doubling is modelled never to be the optimum** on the median Wikidata
   block.
   - It saves 0.025 lookups per byte over P^{4|8}, for m₈ ≈ 0.12 extra elements per byte.
   - So it needs c_e < 0.20–0.35·c_L.
   - That depends only on measured counts and the ratio c_e/c_L.
2. **78% of P⁸'s records, and 63% of P⁴'s, are never executed** (measured, 15 blocks,
   512-segment walk).
   - It is inherent to a static table: 75% of the intervals are single rows inside bursts,
     and a k-step walk lands on 1/k of the rows.
   - Lazy construction can't avoid it, because the landing rows are known only by walking.
3. **The unsplit fast path is worth about 5% of a P⁸ build** (model). The 94% unsplit
   parents cover only 66% of the rows, and they pay the per-parent work, about half the
   builder's instructions, regardless.
4. **Building from the T-vector is dominated.** It adds 8·m₈/n = 0.98 dependent loads per
   output byte at the median, and 1.74 at p90, on top of the T-vector build. Its peak is
   about 5.1 MB.
5. **Cooperative construction cuts a block's wall time, not its CPU.** It does nothing for
   the 4-core `Plan::Workers` target.
6. **The portfolio's peak figure for N1** (4.23 MB against the T-vector's 4.5, §3.1(j))
   leaves out L's 0.9 MB, which the T-vector side includes.
   - With MTF still writing L (N1 without N3), add 0.9 MB, unless L's buffer is recycled
     after level 1.
   - Writing the final level in walk format (12 B, not 16) brings P⁸'s median peak to
     3.79 MB, excluding L.
7. **This note's traffic reconstruction is higher than the portfolio's:** 22.5 B/B against
   17.0–17.5, for P⁸ layout C. Neither figure decides the question, since the arrays fit in
   L2 or L3.
8. **Zen 5 is the hardest runner for N1.** Its walk is the cheapest (4.18 c/B, IPC about
   4), so its c_e thresholds are about 40% lower (6.1 and 14.2, against 10.2 and 22.4).
9. **High-r blocks** (p90 r ≈ 99,000) allow about half the median block's c_e. The graded
   selector (P⁸/P⁴/P²/T-vector, §3.1(i)) is needed for them, and so is a measured fallback
   cost.

## 8. Assumptions (every model figure)

- **The walk:**
  - c_L is 16–24 instructions per lookup: the shipped room walk's 16-instruction step,
    plus a rank and a word store.
  - It runs at the shipped walk's IPC on each runner. That IPC is derived from the i9's
    16.9 instructions per byte and the CI stage c/B.
  - The walk is instruction-bound, with two dependent L2/L3 accesses per lookup hidden by
    32 lanes.
- **The build:**
  - The per-element instruction counts are those in §3.3.
  - The IPC is 2.0–3.0 on the 7763, with 16 cycles per mispredict at 1–2 per split parent.
  - Zen 4 and Zen 5 builds are scaled by the shipped build's measured ratio across
    runners (0.87, 0.70).
- **The lookups and the split fractions** are sample medians (15 blocks) applied to all
  233. The lookups vary by less than ±5% across the sample.
- **The L run scan** costs 0.3–0.6 c/B. That is #42's measured 0.93 c/B locally, scaled
  to CI.
- **The four-copy factors** in §5.4 are as stated there.
- **The upgrade's correctness** is argued, not brute-force checked.

## 9. Files

Read-only scripts, kept in the scratchpad's `n1-analysis/` (not committed), as the portfolio's `theory/` scripts are:
- `sample_sim.py`: the 15-block measurements of §2;
- `sample.json`: its output;
- `model.py`: the census-wide model of §§3–5;
- `model.json`: its output.

Inputs: `data/census.json`, `data/blocks.npz` and the portfolio §3.1. The #42 build
figures are from `docs/journal/2026-10-27-move-structure-walk.md` (mibz2 `origin/master`).
