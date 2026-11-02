# External research report 149: the researcher on report 148: four copies do the same instruction work per row in 37–43% more cycles (verbatim)

A research input, not a finding of this repository.

- **Relayed:** by the maintainer, 2026-11-02, with reports 148–150 together.
- **Source:** the independent researcher of reports 138–147.
- **In reply to:** report 148.

**Evidence cutoff:** 2 November 2026. A recalculation from the archived raw records; no new run.

It is kept exactly as received (rendered text, cards flattened, tracking parameters
included).

## The message, as received

`````text
I agree with the guardian’s assessment, and **the concurrency result should now have more weight in the research direction than I gave it in my previous reply**. I concentrated on whether the repaired measurement was trustworthy. The stronger architectural finding is that a substantial penalty survives after removing the production pipeline and decoded-output work.

I made one additional comparison using the existing archive that strengthens that conclusion.

## The copies perform almost the same instruction work, but need substantially more cycles

I restricted this comparison to the `exec` group in both the one-core and four-core measurements, so instructions, µops and cycles were counted together.

| Rung | One-core instructions/row | Four-copy instructions/row | Change in instructions | Change in cycles/row |
|---|---:|---:|---:|---:|
| **A** | 19.4381 | 19.4432 | **+0.026%** | **+38.3%** |
| **B** | 20.4347 | 20.4376 | **+0.014%** | **+36.6%** |
| **P** | 16.9915 | 16.9956 | **+0.024%** | **+43.3%** |

These are recalculated from the walker-ladder-20261102T004502Z.jsonl[archived raw measurements](sandbox:/workspace/scratch/eb4b5d8c4a8d/analysis/walker_20261102T004501Z/walker-ladder-20261102T004502Z.jsonl). The cycle increases differ slightly from the guardian’s table because this uses only the matched execution-counter group; the guardian’s figures pool the available cycle measurements across groups. Both show the same effect.

Retired-slot counts per row also change by less than **0.1%**, and issued µops by less than **0.51%**, across these three comparisons.

**The additional software work per row is tiny compared with the increase in cycles. The concurrent copies are making slower progress through essentially unchanged instruction work.**

That strengthens the argument against explaining this particular penalty through extra dispatch instructions or a large increase in walker bookkeeping.

## What this allows us to rule out as necessary causes

The reduced rungs give us a useful sequence of exclusions:

- **The production job queue is not required for this kind of slowdown to appear.** These are independent walker copies.
- **Writing decoded output and assembling segments are not required.** A and B omit that work and still suffer a large penalty.
- **Software prefetch is not required.** A has no software prefetch and still becomes substantially more expensive.

Those are meaningful findings. They do not imply that those mechanisms contribute nothing to P’s cost, or that the original full decoder’s slowdown has exactly the same cause.

One precision point is worth preserving: **A and B are output-free, but they still maintain lane state.** They are not kernels without bookkeeping or memory writes of any kind. The source updates each lane’s position and remaining work. [Pinned reduced-walker implementation](https://github.com/lmmx/mibz2/blob/29a0215e3d06cdb436bf212501b01e54c48c7bf0/examples/walker_ladder.rs#L153).

The defensible conclusion is therefore:

> A substantial concurrency penalty exists in the traversal itself, with almost unchanged work per row, even when several surrounding production mechanisms are absent.

That is considerably more informative than observing that the complete decoder scales imperfectly.

## Frequency explains part of the elapsed-time difference, but leaves the main question open

The same records show the ratio of counted cycles to elapsed nanoseconds falling from approximately **5.30 to 4.95–4.96 cycles/ns** between one and four copies. That is consistent with a lower effective core clock and helps explain why the elapsed-time penalty is larger than the cycle penalty. These ratios are also derived from the walker-ladder-20261102T004502Z.jsonl[raw records](sandbox:/workspace/scratch/eb4b5d8c4a8d/analysis/walker_20261102T004501Z/walker-ladder-20261102T004502Z.jsonl).

They are not a dedicated frequency measurement, so I would not label the effect thermal throttling or assign an exact frequency history.

The guardian’s distinction is correct: **a lower core clock alone, with unchanged cycles needed per row, cannot account for the large increase in cycles per row.** Something about progress through the workload also changes under concurrency.

That leaves questions about cache and memory service, request interactions, and package operating conditions. The current capture does not distinguish them reliably enough to choose one physical explanation.

## How I would now use this in the architectural assessment

I would carry forward three conclusions.

### 1. Ordinary scheduling and bookkeeping cleanup has a weaker case

The single-core P/M comparison is already inside the registered stopping band. The new instruction-count comparison adds a separate reason not to make ordinary cleanup the principal response to the concurrency penalty: the amount of executed instruction work barely changes when the penalty appears.

This does not make every local optimization worthless. It makes a substantial rewrite of the existing walker harder to justify.

### 2. Reducing the concurrent cost of traversal remains a relevant objective

A different representation could, in principle, alter the cost of servicing the traversal’s accesses when several copies run. The experiment leaves that possibility open.

But it has **not** shown that a smaller representation will deliver that benefit. In particular, it has not established cache capacity as the dominant constraint. The rare-event warnings prevent us from using the small L3 counts to settle that question.

The architectural question is now more focused: can a candidate reduce the cost of concurrent traversal enough to pay for its own extraction and state-management work?

### 3. FOR-14 needs to keep its two evidence streams separate

The frozen implementation’s measured loss remains decisive evidence about that implementation. The packed-token `k` and `j` results establish a reduction in compiled work, but their performance remains unknown.

The current capture supplies useful context for evaluating them:

- There is little demonstrated one-core headroom in the shipped walker’s existing bookkeeping.
- There is a substantial concurrent traversal penalty that survives simplification.
- A compact representation may change that penalty, while also adding work to every step.

Neither the **27-instruction listing** nor the **57% issued-µop share** resolves that trade-off. I would not derive a speed prediction for `k` or `j` from either number.

**I support closing this diagnostic with D1 preserved and no further workstation run now.** The perf setup has produced a useful result. The research emphasis should move towards explaining and accounting for the concurrent traversal cost, while retaining the distinction between a plausible memory-system benefit and one that a compact candidate has actually demonstrated.
`````
