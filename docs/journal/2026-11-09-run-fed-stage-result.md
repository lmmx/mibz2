# The run-fed stage diagnostic: the registered CI run

9 November 2026. This is the result of the one run registered in
`docs/journal/2026-11-09-run-fed-stage.md`, frozen at `fd60fffada25d475bbcc2b54165b96cf5fd7a92f`.
The Researcher said PROCEED in 6729947501, and Guardian D048 lifted D044 for this run.

**The run:** `sweep.yml` run 42549324025, `tool: run_fed_stage`, 15 rounds,
`registered_head` fd60fffa. Its artifact is `sweep-fd60fffa…`. The judged record,
`run-fed-1.json`, is the pinned copy. The four contended copies are reported only.

**Validity:** `bench/run_fed_stage.py verify --head fd60fffa…` on `run-fed-1.json` gives
**VALID; readings JUDGED**.

## The judged reading

- **Q1:** R_stage, B over A at the entry point on Wikidata, is **1.1592 (1.1387, 1.1801)**.
  - **STOP:** no saving of 5% or more on this runner. This is an operational pause for this
    implementation on this runner, not proof of no effect (PROTOCOL §11).
  - **Screens:** the worst synthetic, `runs`, is at 1.546. The worst fallback, `wikidata`'s fallback
    blocks, is at R_fb 1.110. Both fail; both are point-estimate screens.
- **Q2 (exploratory):** R_lead is 1.1185 (1.0935, 1.1441), so there is no leader lead.

## Where the cycles go

These are Wikidata, the pinned copy, nominal c/B: wall time × the sampled clock, not
counted cycles.

| phase | A: column (today) | B: records | C: column + checkpoints |
|---|---|---|---|
| front: MTF or emitter | 1.290 | 1.508 | 1.640 |
| sort | — | 0.317 | — |
| build or fill | 1.432 | 1.332 | 1.429 |
| fallback | 0.971 | 1.071 | 1.154 |
| entry point | 3.676 | 4.291 | 4.244 |

There were 384 blocks by records (79.8% of rows, with r/n 0.0621 there) and 47 fallback blocks.

- **The fill is cheaper than production's build.** 1.332 against 1.432, about 7% less, even
  though it writes 1.48 slots a row.
- **The losses:**
  - The **emitter is 17% dearer than plain MTF** (R_front 1.185), because it writes a 4-byte
    record per literal. It is 8% cheaper than the checkpointed MTF (R_emit 0.925), which agrees
    with #270's C29 finding that the earlier 25% was mostly the list kernel.
  - The **sort** costs 0.317, about 22% of the build.
  - **Fallback blocks cost 10% more** (R_fb 1.110), although their bodies are meant to be the
    same. The selector or the dispatch is charged on blocks that gain nothing.
- **The build is not store-bound.** R_floor is 0.321: the table's stores alone are about a third
  of the build. R_warm is 1.010: a pre-touched table changes nothing. Mechanism M, that the
  build's cost is set by writing its table, is not supported here. What binds is the build's
  latency or instructions, not its stores.
- **Run-heavy blocks win.**
  - `one byte` gave 0.641 (0.587, 0.699) and `pattern` gave 0.638 (0.595, 0.683).
  - `runs` (r/n 0.18) gave 1.546, and `text` (R_acc 1.458) lost on its run blocks.
  - So the gain depends strongly on run structure, beyond what the r/n fallback rule captures.

## What this means

As built, the run-fed stage loses 16% on Wikidata, on one worker. Its table fill is not where
it loses. It loses on the emitter's literal records, the sort, and the selector's cost on
fallback blocks. Each of those is a construction target (PROTOCOL §11):
1. **Direct placement with no sort (C31):** it removes the 0.317 of sort and its F array.
2. **A cheaper emitter:** cut the per-literal record store, for example by keeping L for
   literals and emitting records for runs only.
3. **A zero-cost fallback:** a decision made without wasted work on blocks that won't take runs.
4. **Per-block admission by measured benefit:** the run-heavy blocks gain 36%. Find the block
   statistic that predicts the win better than r/n.

The line stays open. These go to the Optimiser and the Researcher. Nothing here changes
production.
