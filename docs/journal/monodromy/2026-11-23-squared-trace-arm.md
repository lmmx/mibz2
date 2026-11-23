# 2026-11-23 · MONODROMY 93 · The squared-trace arm for bench-emit-cost: spec, frozen prediction and decision rule. The i9 pre-screen halves the dependent chase almost exactly (0.53–0.55) and refutes record 92's 0.85–0.95. T² composes with the two-ended emit. Built serially, it should lose slightly, about 5% of the open kernel. The prediction is SHADOW

**The request.** The Coordinator's CHECKPOINT 7096857998 says the pre-screen is ALIVE on the i9 (LOG 7096857555, bundle `5218e05b`). It asks, before anything is built, for:
1. the T² arm's spec on OpenCells' real tables, T and T⁻¹;
2. a frozen prediction and decision rule at 1, 6 and 10 workers;
3. whether T² composes with the two-ended emit or replaces it.

**Labels:** **[proved]** · **[measured]** · **[conjectured]**, including **[modelled]** arithmetic.

**Evidence read** (SHA-256 checked against the stored digests):
- `squared-trace-prescreen/20261123T093340Z` (9734fcab…);
- `emit-cost/run-20261122T034734Z` (98116d63…).

**Code read:** `src/open.rs` and `src/open/outq.rs` at master.

## 1. What the i9 says, and how my model changes [measured]

**The pre-screen split into its parts** (medians of 11 invocations, ns per row):

| m | chase T | gather | chase T² | T² chase / chase | gather / chase | ratio |
|--:|--:|--:|--:|--:|--:|--:|
| 89,874 | 4.60 | 0.94 | 2.55 | 0.554 | 0.204 | 0.760 |
| 114,000 | 4.92 | 0.99 | 2.67 | 0.543 | 0.201 | 0.745 |
| 156,404 | 5.24 | 1.06 | 2.79 | 0.532 | 0.202 | 0.733 |
| 204,281 | 5.46 | 1.10 | 2.87 | 0.526 | 0.201 | 0.726 |

**What it shows:**
- **The dependent chase halves almost exactly.** T² runs at 0.53–0.55 of the plain chase.
- **The gather is a fixed fifth of the chase** at every m.
- **The ratio falls with m** because the halving gets closer to perfect as each miss lengthens.
- **All four m are in one regime on the i9:** past 256 KiB of L2, inside 20 MiB of L3. My container has 2 MiB of L2 a core, so it held the small m in L2 and switched regime at the large ones. That produced the rising shape there. The i9's shape is the one that counts.

**Record 92's model is refuted on the walk.**
- *What it said:* "a walk ratio of 0.85–0.95 on the i9, a net 1–4% at best" [conjectured].
- *The miss:* it read the container's large-m regime onto the i9. The measured walk ratio is 0.73–0.76 with the gather included.

**Open's emit is this chase [measured].**
- *The baseline:* `bench-emit-cost` measured b = 4.82, 5.13 and 5.79 ns per retained row at 1, 6 and 10 workers. That is with mean m 114,024 and 928 of 932 blocks two-ended.
- *The match:* the pre-screen's plain chase at m = 114,000 was 4.92 ns.
- *So:* the 16-byte copies are hidden under the miss latency, and the emit is latency-bound. That is the regime where halving the chain pays.

## 2. The arm's spec

**Where it runs.** It runs only where OpenCells already runs two-ended: `QUEUE && !wide && m ≥ 65,536`. That is 928 of 932 open blocks a decode on Wikidata ×4.
- *Other blocks:* the forward emit is unchanged.
- *Why not below 65,536:* there, `edges` (4 bytes × m ≤ 256 KiB) is L2-resident on the i9 and the walk is bound by instructions. That is where the ψ² port lost (#34).

**The tables.** T is Open's retained return map, and `edges[j] = code(j) << bits | (T(j) ⊕ T⁻¹(j))` as wired today.

**(a) Recording the successor**, in construction:
- *The change:* in `outq::Reader::wire_xor` and `wire_xor_slow`, beside `edges[u] ^= pack | v; edges[v] ^= u`, the arm also writes `succ[u] = v`. This is behind a feature gate.
- *Its cost:* the writes are sequential within each band, since u runs over `dst_start..dst_start + span`. They are charged to `construct_ns`.
- *Memory:* `succ` is a u32 scratch of m entries, reused between blocks as `edges` is.

**(b) The square pass** (new, charged to its own stamp `square_ns`):
- *What it does:* zero `x2[..m]`, then for each j:
  - v₂ = succ[succ[j]];
  - x2[j] ^= v₂;
  - x2[v₂] ^= j.
- *The result:* x2[j] = T²(j) ⊕ T⁻²(j), the XOR-linked T² cycle, built exactly as `wire_xor` builds T's.
- *Its traffic:* one random read and one random read-modify-write a row.
- *Memory:* `x2` is a second u32 scratch of m entries. `succ` is dead after the pass, so peak scratch grows by 8m bytes. That is 0.9 MB at mean m. Report it.
- *The recorder:* this adds one clock read a block (four instead of three).

**(c) The walk, `emit_bidirectional_sq`.** It writes into the same `out`, with the same lo/hi layout, as `emit_bidirectional`.
- **Front** (moves by T², emits at lo):
  - *state:* (f, f₋₂ = T⁻²f, h = T⁻¹f);
  - *start:* f = mark, h = before, f₋₂ = (edges[before] & mask) ⊕ mark;
  - *a step:*
    - g = T(f) = (edges[f] & mask) ⊕ h;
    - emit code(f) = edges[f] ≫ bits, then code(g) = edges[g] ≫ bits;
    - then (f₋₂, f, h) ← (f, x2[f] ⊕ f₋₂, g).
- **Back** (moves by T⁻², emits at hi): the mirror.
  - *state:* (b, b₊₂ = T²b, k = T b);
  - *start:* b = before, k = mark, b₊₂ = (edges[mark] & mask) ⊕ before;
  - *a step:*
    - g′ = T⁻¹(b) = (edges[b] & mask) ⊕ k;
    - emit code(b) ending at hi, then code(g′) before it;
    - then (b₊₂, b, k) ← (b, x2[b] ⊕ b₊₂, g′).
- **The critical path** is one load a step for each end, `x2[f]` or `x2[b]`. The reads of `edges[f]` and `edges[g]` are off that path, exactly as in the pre-screen.
- **The word pair is not a new table.** Each step emits two of Open's existing codes, read from `edges`. Each is copied from `inline16` or `spelling` exactly as now. A pair dictionary (distinct 2-words are 3–5% of m, record 92) would save copies, not dependent loads. It is out of scope.
- **Termination is exact.** Before each round, check the four rows {f, g} and {b, g′} for any coincidence or crossing. On one, or when `hi − lo` is less than two words, hand over to `emit_bidirectional`'s single-step loop from the current state. Its (prev, front) is (h, f) and its (back, next) is (b, k). That covers odd cycle lengths, periodic blocks' short cycles and the meeting point. The existing `copy_within` and `repeat` then finish the block unchanged.
- **The hand-back to emit:** `walk()` calls `emit_bidirectional_sq` in place of `emit_bidirectional` when the arm is on, and nothing downstream changes. The block CRC checks every byte, as it does now.

**The bench.**
- *One binary:* `emit-cost` plus a `squared-trace` feature. The arm is switched at run time between decodes by a process-global flag the harness sets, so both arms decode the same blocks in the same build.
- *The grid:* {base, T²} × {1, 6, 10} workers × 20 rounds. The six cells rotate within each round, with 2 s of sleep before each decode, as now. That is about 120 decodes, roughly 9 minutes.
- *Per block:* `construct_ns`, `square_ns` (0 for base), `emit_ns`, m, and which emitter ran.
- **VOID**, adding to emit-cost's own conditions:
  - the two arms' (n, m, refused) multisets differ;
  - any decode's bytes differ from 838,800,784;
  - the T² arm's two-ended block count differs from the base's.
- **A reported check, not a VOID:** the base arm's b at 10 workers should land within 10% of 5.786. If not, the machine's state has moved since the emit-cost run, and the LOG should say so.

## 3. Composition with the two-ended emit [proved, then measured]

**T² composes with the two-ended emit. It does not replace it.**
- x2 is XOR-linked, as `edges` is, so it can be walked from both ends: T² forward from the mark, T⁻² backward from its predecessor.
- Each round covers four rows. The dependent rounds fall from m/2 (Law 4) to m/4.
- Bijectivity and wire_xor are kept, as Law 4 requires.
- The pre-screen measured exactly this composition: two chains, each on the squared map.

**Below m = 65,536 it does not run.** That leaves those few blocks as they are.

## 4. The frozen prediction (per retained row, Wikidata ×4, each arm paired with base in the same round)

**The judged quantities.**
- **E**, the mechanism: emit_T² / emit_base.
- **K**, the net in ns per retained row: (construct + square + emit)_T² − (construct + emit)_base.

**The model [modelled]:**
- *emit_T²:* the pre-screen's 0.53–0.55, plus the share of copies that no longer hides under a halved latency.
- *square:* about twice the pre-screen's one-read gather, because it adds a random read-modify-write.
- *construct:* rises by the `succ` stores.
- *For scale:* base construct is 5.68, 6.21 and 7.10 ns per retained row (647.5, 708.0 and 809.7 µs a block over mean m 114,024).

| workers | base emit (measured) | E | square, ns | Δconstruct, ns | K, ns per retained row | K / base kernel |
|--:|--:|--:|--:|--:|--:|--:|
| 1 | 4.82 | 0.62 [0.55, 0.75] | 2.0 [1.4, 2.8] | +0.2 [0.05, 0.4] | **+0.4 [−0.5, +1.3]** | +4% [−5%, +12%] |
| 6 | 5.13 | 0.63 [0.55, 0.77] | 2.2 [1.5, 3.1] | +0.2 [0.05, 0.4] | +0.5 [−0.5, +1.5] | +4% [−4%, +13%] |
| 10 (judged) | 5.79 | 0.65 [0.55, 0.78] | 2.5 [1.7, 3.5] | +0.2 [0.05, 0.4] | **+0.7 [−0.4, +1.8]** | +5% [−3%, +14%] |

**How K is formed:** K = E × base emit + square + Δconstruct − base emit. Its interval combines the three inputs' half-widths in quadrature. The base kernel is 10.50, 11.34 and 12.89 ns per retained row at 1, 6 and 10 workers.

**What I expect:**
- **The mechanism transfers:** E is about 0.62–0.65.
- **The serial square pass costs about what it saves.** Two random line touches a row, against a halved chase of about 2 ns a row.
- **The net is a small loss,** about 4–5% of the open kernel at the point estimate. A gain is inside the interval.

## 5. The decision rule (frozen before any data; diagnostic, C1a)

Judged at 10 workers. 1 and 6 workers are reported. E and K are means over the 20 paired rounds, with 95% t intervals.

| outcome | condition at 10 workers | what follows |
|---|---|---|
| **KILL** | E > 0.80 | The halving does not survive Open's copies and dictionary. The T² line ends. |
| **PORT-CANDIDATE** | E ≤ 0.80, K ≤ −0.3 ns per retained row, and K's interval wholly below 0 | Serial T² is worth a production port. It goes through PERFORMANCE-POLICY adoption, memory gate included: +8m bytes of scratch. |
| **SHADOW** | E ≤ 0.80, and not PORT-CANDIDATE | The mechanism transfers, but the serial build costs too much. One further arm, and the line's last: the shadow build (below), judged by the same rule on K. If it is not PORT-CANDIDATE, the line ends. |

**My frozen call: SHADOW.** In rough odds:
- SHADOW about 80%;
- PORT-CANDIDATE about 10%;
- KILL about 10%.

**The shadow build (specified now, built only on SHADOW).**
- *What it does:* start the block with today's two-ended T walk. Interleave the square pass into that walk's latency shadow, a fixed batch of x2 entries per step, ordered by j. Switch to `emit_bidirectional_sq` when x2 is complete.
- *Where its cost lands:* all of it in `emit_ns`.
- *Correctness:* x2 is only read after it is complete, so the result is exact.
- *Its prediction if built [conjectured]:* E_shadow 0.80 [0.70, 0.92] and K −0.9 [−1.7, 0.0] ns per retained row at 10 workers. The limit is the i9's line-fill capacity: a little over one random line a nanosecond, shared with the chase.

**This rule does not decide adoption.** A PORT-CANDIDATE still faces the ordinary policy and the memory gate, with its own registration.

## 6. Against the earlier ceilings

**Record 92's "net 1–4% at best" was a ceiling on a walk ratio of 0.85–0.95.**
- *What the i9 changed:* the halving is real, so a fully hidden build could save up to about (1 − 0.65) × 5.79 − 0.2 ≈ 1.8 ns per retained row at 10 workers. That is about 14% of the open kernel (12.89 ns per retained row).
- *What the serial build changes:* it spends most of that saving. That is the whole reason for the SHADOW branch.

**Law 2's pool (2.65 ms on Neon's six blocks) bounds the same thing from above.** Only the latency share of the emit can be saved, and T² saves about half of it.
