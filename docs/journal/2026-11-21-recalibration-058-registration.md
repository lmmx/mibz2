# The worker-count recalibration for 0.5.8: the registration

*21 November 2026. Frozen before any data. RELEASING §3.4 and §5c.*

This is 0.5.7's recalibration (`bench-recalibrate-057`, the 2026-11-18 design re-pinned to
17405826) run again on master's head for 0.5.8. The design, the rule, the rounds, the settle,
the corpora and the diagnostics are the registered ones
(`docs/journal/2026-11-18-recalibration-registration.md`, Researcher REVIEW 6978182129, its
targeted correction applied at cee99296). This document says only what differs, and gives in
full the table each count is judged against.

- **Recipe:** `just bench-recalibrate-058`, on `bench/recalibrate-058`.
- **Evidence path:** `target/recalibrate-058/run-<UTC timestamp>/`, bundled by `bench/bundle.sh`.
  0.5.7's bundle is `recalibrate-057/run-20261119T185157Z` on the `evidence` branch; 2026-11-18's
  is `recalibrate/run-20261118T122744Z`.
- **The untimed DRY preflight** (`just bench-recalibrate-058 4 12 24 0.25 5 1`) is kept from
  057's recipe. It is not evidence.

## Why §5c applies

#668, merged as **70b18ca4411fa10c9f29b80821c835358dde2f9e**, changes the open kernel's per-block
work and its queue layout: a fixed 16-row queue fill, Vec16 queues kept at full length with the
retained-row count as the cursor, and a 16-byte emit. `src/open.rs` alone changes in the kernel;
routing is unchanged.
- **Natively:** the records-path Open kernel took 347–359 ms against 433–441 ms on master, one
  thread on a 4-vCPU container (#666's journal): about 18% less kernel time.
- **On the i9** (`bench-open-vec16`, open-vec16/run-20261121T004904Z): wall 0.947 [0.939, 0.955]
  at 10 threads unpinned, 0.974 at 8, 0.922 at 6; CPU 0.95; peak RSS within 3.4%.

That is §5c.1's first case: the release changes the kernel's work per block, so the knee may
move. Every lazy arm (`mibz2-occ-N-S`, `StreamDecoder::workers(.., true)`, `OpenCells`) runs the
new kernel on LazyCells' admitted blocks. The eager arms (`mibz2-oce-N-S`, `Cells`) do not
change.

## The decoder measured

- **The pin:** the recipe refuses to run unless `src`, `Cargo.toml` and `Cargo.lock` match
  master's **70b18ca4411fa10c9f29b80821c835358dde2f9e** byte for byte (`git diff --quiet $PIN
  HEAD -- src Cargo.toml Cargo.lock`). That was origin/master's head when this branch was made.
  0.5.7's pin was 17405826; 2026-11-18's was v0.5.5, 7214f8ca.
- **The branch:** origin/master at 70b18ca4, with `bench/recalibrate-057` (9ba4416a) merged in.
  Its harness (`examples/ab.rs`, `bench/two_arm.py`, `bench/counters.py`, their tests) is
  057's, unchanged. It adds nothing to `src`.
- **The build:** one, by rustc 1.99.1.

## What changed in the cells, and why

**The shipped trim at 6 threads is lazy from 0.5.7.** 0.5.7's recalibration adopted one cell
(#642, `trim_cell` and `unpinned_cell` in `src/parallel.rs`):
- *pinned at 6:* lazy 5/20 (`TRIMMED_AT_SIX`), at 0.9340 [0.9264, 0.9417] of eager 5/20;
- *unpinned at 6:* lazy 6/24 (`UNPINNED_AT_SIX`), the plan's six workers with the lazy kernel,
  at 0.8531 [0.8024, 0.9071] of eager 6/24.

The design judges each cell against the **shipped trim**, so the two 6-thread cells take the
new references. Their candidates follow the design's own rules for a lazy shipped trim: one
worker fewer and one more in its kernel (4 blocks a worker), and the untrimmed plan. The plan
at 6 threads is still eager (`Plan::for_threads(6)`, below `LAZY_FROM_THREADS`), so the
untrimmed arm is eager 6/24, and it is also the other kernel at unpinned 6's shipped count.
- **Pinned 6** was eager 5/20 against eager 4/16, eager 6/24 and lazy 5/20. It is now lazy 5/20
  against lazy 4/16, lazy 6/24 and eager 6/24.
- **Unpinned 6** was eager 6/24 against eager 5/20 and lazy 6/24. It is now lazy 6/24 against
  lazy 5/20 and eager 6/24.
- **Not run: eager 5/20 at pinned 6.** That is the trim 0.5.7 replaced. The design's
  "other kernel" arm belongs to the eager cells alone (2026-11-18, design choice 3), and #668
  only makes the lazy kernel cheaper. Leaving it out keeps the cell at 4 arms, as every
  pinned cell of 057's was, so its rounds rotate and balance as theirs did.

Every other cell and arm is 057's. The cells are still 12 pinned and 14 unpinned, the runs 111
a round.

## The cells and arms, and the trim each count is judged against

The reference at each cell is the decoder `StreamDecoder::new(T, 4T, k)` builds on the measured
i9 at master 70b18ca4. Changed rows are marked.

| placement | threads | shipped (reference) | −1 | +1 | untrimmed | other kernel |
|---|---|---|---|---|---|---|
| pinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| pinned | 8 | lazy 6/24 | lazy 5/20 | lazy 7/28 | lazy 8/32 | — |
| pinned | 6 (**changed**) | **lazy 5/20** | lazy 4/16 | lazy 6/24 | eager 6/24 | (= untrimmed's kernel) |
| pinned | 4 | eager 3/12 | eager 2/8 | eager 4/16 (= untrimmed) | eager 4/16 | lazy 3/12 |
| unpinned | 4 | eager 4/16 (untrimmed) | eager 3/12 | not run (above the threads) | — | lazy 4/16 |
| unpinned | 6 (**changed**) | **lazy 6/24** | lazy 5/20 | not run | eager 6/24 | (= untrimmed) |
| unpinned | 8 | lazy 8/32 (untrimmed) | lazy 7/28 | not run | — | — |
| unpinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| unpinned | 12 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/48 | — |
| unpinned | 16 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/64 | — |
| unpinned | 20 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/80 | — |

**The judged cells, as the recipe calls `two_arm.py`:**
- *Pinned* (`--ref-at mibz2-occ-7-28=10 mibz2-occ-6-24=8 mibz2-occ-5-20=6 mibz2-oce-3-12=4`),
  12 cells: at 10, lazy 6/24, 8/32, 10/40; at 8, lazy 5/20, 7/28, 8/32; at 6, lazy 4/16, lazy
  6/24, eager 6/24; at 4, eager 2/8, eager 4/16, lazy 3/12.
- *Unpinned* (`--ref-at mibz2-occ-7-28=10,12,16,20 mibz2-occ-8-32=8 mibz2-occ-6-24=6
  mibz2-oce-4-16=4`), 14 cells: at 10, lazy 6/24, 8/32, 10/40; at 12, 16 and 20, lazy 8/32 and
  lazy 10/S; at 8, lazy 7/28; at 6, lazy 5/20 and eager 6/24; at 4, eager 3/12 and lazy 4/16.

A reference at one count is a candidate at another, never at its own, as before (lazy 6/24:
the reference pinned at 8 and unpinned at 6, a candidate pinned at 10 and 6 and unpinned at 10;
lazy 5/20: the reference pinned at 6, a candidate pinned at 8 and unpinned at 6).

**Checked deterministically:** `tests/recalibrate_arms.rs` now holds this table. Under
`parallel::with_machine` with the measured i9's facts, each shipped arm is the default decoder's
worker count, window and kernel, and each untrimmed arm is the plan's. 057's table fails it on
master (`true 6: the default is mibz2-oce-5-20`); this one passes (3/3).

## Method

As registered: one `ab run` a cell with its own arms, the shipped trim first, rotated and
reversed round by round; **12 paired rounds, 24 pinned at 8 threads**; settle 0.25 s
(`--max-busy` 1.5); Wikidata ×4 (the gain family), text and random (the guards, generated afresh
by `ab gen`), every input's SHA-256 recorded; the spinner held at 4, 6 and 8 threads pinned.

**Time:** 111 runs a round, as 057. 057's run took 12 min 8 s to the end of the timed runs and
12 min 40 s in all (started 18:51:57, finished 19:04:37). #668 makes the lazy arms a little
faster, so **about 13 minutes** on the i9.

**Power:** as registered (2026-11-18 § "Power"). The rounds, families and cells per family are
unchanged, so the detectable gains are too. 057's run gave one more point on the pinned-10 cell's
noise: its lazy 6/24 and 10/40 intervals spanned about ±7%, far wider than the ±1.3% the power
table assumed. An INCONCLUSIVE at pinned 10 is the expected outcome at that noise, and no
evidence of no difference.

## The rule

Unchanged (2026-11-18 § "The rule"):
- each placement one Holm family at α 0.025 on Wikidata ×4, one-sided;
- the loss screen: no cell SLOWER on any of the three corpora;
- memory, rule 4 as amended (#558), with the encoded trade (`--primary wikidata-x4,text`) and no
  trade with two corpora above 1.10;
- the verdicts ADOPTS, ADOPTS BY TRADE, HELD ON MEMORY, A FINDING NOT ADOPTED, NOT ADOPTED;
- at each count, the choice is the ADOPTing candidate with the lowest main-corpus step; if none
  ADOPTs, **the shipped trim in the table above stays**;
- VOID on every check `two_arm.py` makes, and on a record whose worker threads are not its
  arm's N (`--check-workers`). A VOID placement is not re-run on this registration;
- one run decides: no repeat, no extension of the rounds.

**The integration decision:** each count's worker count and kernel in `trim_cell` and
`unpinned_cell`, before 0.5.8 is tagged. A count whose choice is not the shipped trim gets its
own PR to master, with the change, a deterministic test of the new cell and this run's figures,
routed to the Researcher before any tag. The verdict goes in 0.5.8's CHANGELOG section, changed
or not (§5c.4).

**What the rule most likely says at 10 threads and up, said before the data:** as 2026-11-18
said, 8 workers against 7 would cost about 12–14% more peak RSS for a gain of a few percent;
057 measured 8/32 at 0.973 and 1.120 of 7/28's memory pinned at 10. Under rule 4 that is HELD ON
MEMORY at best, not an adoption by rule.

## Model first (§5c.3): MONODROMY's prediction

**The model's recipe is in the repository, but not on master.** MONODROMY's predictions for
0.5.7's run are record 83,
`docs/journal/monodromy/2026-11-19-recalibration-057-predictions.md`, with its script
`2026-11-19-recalibration-057-predictions.recal057.py.txt`, on MONODROMY's branch
`claude/sharp-meitner-cx0v78` (head 32d32f74 when this was written). It is the records 45–56
model, wall ∝ c(W)/CPU(W) with c = 1 + β·W·CPU(W), its front-end term ν scaled by 1.06 for the
open kernel's faster workers.

**As 057 did, MONODROMY's prediction is frozen before the data,** as a new record for this
run, and cited in the queue entry before the ready tag is set. Its inputs differ from record
83's in two places, both for MONODROMY to decide:
- *the kernel:* #668's workers are faster again (about 5% wall at 10 threads), which record 83's
  form enters through ν;
- *the 6-thread cells:* record 83 predicted eager 5 and eager 6 to stay there; they are now
  lazy, and record 83 gives no prediction for lazy 4/16 or 6/24 pinned at 6, or for lazy 5/20
  unpinned at 6.

**Not found:** a scoring of record 83 against 057's run (`recalibrate-057/run-20261119T185157Z`).
Record 56's scoring exists (`2026-11-18-recalibration-scoring.md` on the same branch).

## Reported, and deciding nothing

**The memory controller**, as registered, at three cells, where `kernel.perf_event_paranoid` ≤ 0:
- unpinned at 10: lazy 7/28 against 6/24, 8/32 and 10/40;
- unpinned at 6: **lazy 6/24 (now the shipped trim) against eager 6/24**, the reference and
  candidate swapped from 057;
- pinned at 4: eager 3/12 against lazy 3/12.

**The diagnostics** (order, drift, load) are as declared on 2026-11-18, for every judged cell.

## What it does not decide

As 2026-11-18: no other machine, mask, quota or SMT placement; no arm with more workers than
the threads asked; no count other than these eleven cells; not `LAZY_FROM_THREADS` or the plan
off the measured i9.
