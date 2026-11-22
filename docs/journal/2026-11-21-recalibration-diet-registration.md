# The worker-count recalibration after the Open kernel's memory diet: the registration

*21 November 2026. Frozen before any data. RELEASING §3.4 and §5c.*

This is 0.5.8's recalibration (`bench-recalibrate-058`, the 2026-11-18 design re-pinned to
70b18ca4) run again on master's head, for the release after 0.5.9. The design, the rule, the
rounds, the settle, the corpora and the diagnostics are the registered ones
(`docs/journal/2026-11-18-recalibration-registration.md`, Researcher REVIEW 6978182129, its
targeted correction applied at cee99296). This document says only what differs, and gives in
full the table each count is judged against.

- **Recipe:** `just bench-recalibrate-diet`, on `bench/recalibrate-diet`.
- **Evidence path:** `target/recalibrate-diet/run-<UTC timestamp>/`, bundled by
  `bench/bundle.sh`. 0.5.8's bundle is `recalibrate-058/run-20261121T020723Z` on the `evidence`
  branch (d353b473); 0.5.7's is `recalibrate-057/run-20261119T185157Z`; 2026-11-18's is
  `recalibrate/run-20261118T122744Z`.
- **The untimed DRY preflight** (`just bench-recalibrate-diet 4 12 24 0.25 5 1`) is kept from
  058's recipe. It is not evidence.

## Why §5c applies

#692, merged as **23e40f7708c86e4a421b1f0b9a6413ad3a6e70db**, is the Open kernel's memory diet.
It changes the kernel's working set and scratch layout, in `src/open/` only:
- *W1:* the endpoint elimination's nodes, rules and bottom coordinates live in `edges`' own
  buffer, which is sized exactly instead of doubled;
- *OutQ16:* the 256 `Vec<u32>` queues are gone; each byte's retained ranks are queued as `u16`
  in 1 KiB chunks of the block's own `out` buffer, which is dead until the walk.

What it changed, measured:
- **Natively** (#692's harness, the six admitted blocks, one kernel reused across them): the
  kernel's persistent scratch, which is also its in-flight peak, fell from **4,236 KiB to
  1,200 KiB** (0.283×). The queues (1,556 KiB), `edges`' doubling (1,404 KiB) and Endpoint
  (877 KiB) were most of the old figure. `edges` (798 KiB, exact) is most of what is left.
- **On the i9** (`bench-open-diet`, open-diet/run-20261121T121714Z, evidence c56a1371): wall
  0.952 [0.950, 0.955] at 6 threads unpinned, 0.917 [0.894, 0.940] at 8 and 0.931 [0.916,
  0.946] at 10; median peak RSS 0.887–0.898 of base. Adopted.

That is §5c.1's first case: the release changes a worker's working set and scratch layout, so
the knee may move. Every lazy arm (`mibz2-occ-N-S`, `StreamDecoder::workers(.., true)`,
`OpenCells`) runs the dieted kernel. The eager arms (`mibz2-oce-N-S`, `Cells`) do not change.

## The decoder measured

- **The pin:** the recipe refuses to run unless `src`, `Cargo.toml` and `Cargo.lock` match
  master's **23e40f7708c86e4a421b1f0b9a6413ad3a6e70db** byte for byte (`git diff --quiet $PIN
  HEAD -- src Cargo.toml Cargo.lock`). That was origin/master's head when this branch was made.
  0.5.8's pin was 70b18ca4; 0.5.7's 17405826; 2026-11-18's v0.5.5, 7214f8ca.
- **The branch:** origin/master at 23e40f77, with `bench/recalibrate-058` (c695b930) merged in
  as a merge commit. The merge was clean. Its harness (`examples/ab.rs`, `bench/two_arm.py`,
  `bench/counters.py`, their tests) is 058's, unchanged. It adds nothing to `src`.
- **The build:** one, by rustc 1.99.1.

## What changed in the cells, and why

**The shipped trim unpinned at 4 threads is lazy from 0.5.8.** 0.5.8's recalibration adopted
one cell (#678, `UNPINNED_AT_FOUR` in `unpinned_cell`, `src/parallel.rs`): the plan's four
workers with the lazy kernel, lazy 4/16 at 0.9452 [0.9225, 0.9685] of eager 4/16's wall on
Wikidata ×4 (Holm p 1.71e-04 among 14 cells), at 0.927 of its peak memory. It shipped in 0.5.9.

The design judges each cell against the **shipped trim**, so the unpinned 4-thread cell takes
the new reference. Its candidates follow the design's rules for a lazy shipped trim at the
plan's own count, exactly as 0.5.8 applied them at unpinned 6 (lazy 6/24): one worker fewer in
its kernel (4 blocks a worker), and the untrimmed plan, which at 4 threads is eager
(`Plan::for_threads(4)`, below `LAZY_FROM_THREADS`) and so is also the other kernel at the
shipped count. One worker more is not run, as before: it would run more workers than the
threads asked.
- **Unpinned 4** was eager 4/16 against eager 3/12 and lazy 4/16. It is now **lazy 4/16
  against lazy 3/12 and eager 4/16**.
- **Not run: eager 3/12 unpinned at 4.** It is neither the shipped trim's neighbour in its
  kernel nor the plan. 0.5.8 measured it at 1.1845 [1.1421, 1.2285] of eager 4/16, and the diet
  does not touch the eager kernel. Leaving it out keeps the cell at 3 arms, as before, so its
  rounds rotate and balance as 058's did.

Every other cell and arm is 058's. The cells are still 12 pinned and 14 unpinned, the runs 111
a round.

## The cells and arms, and the trim each count is judged against

The reference at each cell is the decoder `StreamDecoder::new(T, 4T, k)` builds on the measured
i9 at master 23e40f77. The changed row is marked.

| placement | threads | shipped (reference) | −1 | +1 | untrimmed | other kernel |
|---|---|---|---|---|---|---|
| pinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| pinned | 8 | lazy 6/24 | lazy 5/20 | lazy 7/28 | lazy 8/32 | — |
| pinned | 6 | lazy 5/20 | lazy 4/16 | lazy 6/24 | eager 6/24 | (= untrimmed's kernel) |
| pinned | 4 | eager 3/12 | eager 2/8 | eager 4/16 (= untrimmed) | eager 4/16 | lazy 3/12 |
| unpinned | 4 (**changed**) | **lazy 4/16** | lazy 3/12 | not run (above the threads) | eager 4/16 | (= untrimmed) |
| unpinned | 6 | lazy 6/24 | lazy 5/20 | not run | eager 6/24 | (= untrimmed) |
| unpinned | 8 | lazy 8/32 (untrimmed) | lazy 7/28 | not run | — | — |
| unpinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| unpinned | 12 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/48 | — |
| unpinned | 16 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/64 | — |
| unpinned | 20 | lazy 7/28 | (at 10) | lazy 8/32 | lazy 10/80 | — |

**The judged cells, as the recipe calls `two_arm.py`:**
- *Pinned* (`--ref-at mibz2-occ-7-28=10 mibz2-occ-6-24=8 mibz2-occ-5-20=6 mibz2-oce-3-12=4`),
  12 cells, as 058: at 10, lazy 6/24, 8/32, 10/40; at 8, lazy 5/20, 7/28, 8/32; at 6, lazy
  4/16, lazy 6/24, eager 6/24; at 4, eager 2/8, eager 4/16, lazy 3/12.
- *Unpinned* (`--ref-at mibz2-occ-7-28=10,12,16,20 mibz2-occ-8-32=8 mibz2-occ-6-24=6
  mibz2-occ-4-16=4`), 14 cells: at 10, lazy 6/24, 8/32, 10/40; at 12, 16 and 20, lazy 8/32 and
  lazy 10/S; at 8, lazy 7/28; at 6, lazy 5/20 and eager 6/24; at 4, **lazy 3/12 and eager
  4/16**.

A reference at one count is a candidate at another, never at its own, as before. Within the
unpinned family lazy 4/16 is now the reference at 4 and a candidate nowhere; lazy 3/12 is a
candidate at 4 in each family.

**Checked deterministically:** `tests/recalibrate_arms.rs` now holds this table. Under
`parallel::with_machine` with the measured i9's facts, each shipped arm is the default decoder's
worker count, window and kernel, and each untrimmed arm is the plan's. 058's table fails it on
master 23e40f77 (`false 4: the default is mibz2-oce-4-16`); this one passes (3/3).

## Method

As registered: one `ab run` a cell with its own arms, the shipped trim first, rotated and
reversed round by round; **12 paired rounds, 24 pinned at 8 threads**; settle 0.25 s
(`--max-busy` 1.5); Wikidata ×4 (the gain family), text and random (the guards, generated afresh
by `ab gen`), every input's SHA-256 recorded; the spinner held at 4, 6 and 8 threads pinned.

**Time:** 111 runs a round, as 058. 058's run took 12 min 31 s in all (started 02:07:23,
finished 02:19:54). The diet makes the lazy arms 5–8% faster, so **about 12–13 minutes** on the
i9.

**Power:** as registered (2026-11-18 § "Power"). The rounds, families and cells per family are
unchanged, so the detectable gains are too. 058's run gave the noise at the cells where this
run's question is sharpest: at pinned 10, lazy 8/32's interval spanned [0.955, 1.097]; unpinned
at 10–20, lazy 8/32's spanned about ±4–7%. An INCONCLUSIVE there is the expected outcome for a
gain of a few percent, and no evidence of no difference.

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
`unpinned_cell`, before the next release is tagged. A count whose choice is not the shipped
trim gets its own PR to master, with the change, a deterministic test of the new cell and this
run's figures, routed to the Researcher before any tag. The verdict goes in the release's
CHANGELOG section, changed or not (§5c.4).

## The hypothesis, said before the data (deciding nothing)

**What the diet changes for a worker.** A lazy worker's own kernel scratch is about 3 MiB
smaller (4,236 to 1,200 KiB natively). Set against the i9's 20 MiB L3 (`TEN_CORES`), seven
workers' kernel scratch was about 29 MiB before and is about 8 MiB now; eight workers' is about
9 MiB. These are capacities, not bytes touched per block, and each worker's T-vector and output
sit beside them, so this is a direction, not a forecast of the knee. But it is the direction in
which the knee moves: a lazy worker now costs the shared cache less, so where a count was
trimmed because many workers contend for it, one more worker may now fit.

**The trims most likely to move:**
- **From 10 threads, lazy 7/28: the contention trims.** These are the cells the model's
  contention term (β·W·CPU(W), records 45–56, 83 and 87) decides, and where `LAZY_FROM_THREADS`'
  own account says "many workers contend for the cache". 058 read lazy 8/32 over 7/28 at 1.0236
  [0.9552, 1.0970] pinned at 10, and unpinned at 1.0362 (10), 1.0339 (12), 1.0372 [1.0054,
  1.0700] SLOWER (16) and 0.9733 [0.9090, 1.0421] (20). Its memory was 1.06–1.11 of 7/28's,
  above the gate on text alone at 10, 12 and 20. If the diet lowers contention more at eight
  workers than at seven, 8/32 moves below 1. If it lowers each worker's footprint more than the
  shared buffers', 8/32's memory ratio falls toward the gate. The rule needs both for an
  adoption.
- **The unpinned 10–20 thread cells especially,** where the shipped plan is lazy 7/28 on every
  online CPU and the untrimmed arms are the plan's ten workers (10/40 to 10/80, 1.07–1.14 of
  7/28 in 058, memory 1.27–1.45). The untrimmed arms carry the most workers, so they gain most
  from a smaller per-worker working set; they are also the arms furthest from adopting.

**Less likely to move, and why:**
- **Pinned 8 (lazy 6/24):** MONODROMY's scoring of record 83 put this trim on the CPU-budget
  term ν, the front end's share, not contention. 058 read 7/28 and 8/32 at 1.08 of 6/24 there.
  A faster kernel raises the front end's share, which favours fewer workers, not more.
- **Below 8 threads (pinned 6 and 4, unpinned 8, 6 and 4):** these trims were set by
  scheduling (a worker fewer frees a core for the front end) and by the choice of kernel, not by
  cache contention. The counts are unlikely to move. The kernel-switch arms may: the diet speeds
  the lazy kernel alone, so the eager arms at unpinned 4 and 6 and pinned 6 should lose by more
  than in 058. The one eager shipped trim left, **pinned 4 (eager 3/12)**, had lazy 3/12 at
  1.0329 [1.0242, 1.0416] in 058. A lazy gain of 5% at 4 threads would bring it to about 0.98,
  near the line.

## Model first (§5c.3): MONODROMY's prediction

**The model's recipe is in the repository, but not on master.** MONODROMY's predictions for
0.5.8's run are record 87, `docs/journal/monodromy/2026-11-21-recalibration-058-predictions.md`,
with its script `2026-11-21-recalibration-058-predictions.recal058.py.txt`, on MONODROMY's
branch `claude/sharp-meitner-cx0v78` (head e42fc32f when this was written). It is the records
45–56 model refitted on 0.5.7's run, with #668's change entering as a uniform per-worker
speed-up g on the front-end term ν.

**As 058 did, MONODROMY's prediction is frozen before the data,** as a new record for this run,
and cited in the queue entry before the ready tag is set. Its inputs differ from record 87's in
three places, all for MONODROMY to decide:
- *the kernel's speed:* #692's lazy workers are faster again (0.952 / 0.917 / 0.931 wall at
  6 / 8 / 10 threads), which record 87's form enters through ν·g;
- *the working set:* the diet's change is to memory, not instructions. Record 87's form has no
  per-worker footprint term; whether β moves with the scratch cut is the model's question;
- *the unpinned 4-thread cell:* record 87 predicted eager 4/16 to stay the shipped trim there,
  with lazy 4/16 a near-tie at 1.02 [0.97, 1.06]. 058 measured 0.9452, and lazy 4/16 is now the
  reference. Record 87 gives no prediction for lazy 3/12 against lazy 4/16 unpinned.

**Not found:** a scoring of record 87 against 058's run (`recalibrate-058/run-20261121T020723Z`).
Record 83's scoring is in record 87 itself.

## Reported, and deciding nothing

**The memory controller**, as registered and as 058, at three cells, where
`kernel.perf_event_paranoid` ≤ 0:
- unpinned at 10: lazy 7/28 against 6/24, 8/32 and 10/40;
- unpinned at 6: lazy 6/24 against eager 6/24;
- pinned at 4: eager 3/12 against lazy 3/12.

**The diagnostics** (order, drift, load) are as declared on 2026-11-18, for every judged cell.

## What it does not decide

As 2026-11-18: no other machine, mask, quota or SMT placement; no arm with more workers than
the threads asked; no count other than these eleven cells; not `LAZY_FROM_THREADS` or the plan
off the measured i9.
