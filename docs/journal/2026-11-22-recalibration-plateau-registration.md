# The worker-count recalibration after Wormhole's plateau stack and the block reader: the registration

*22 November 2026. Frozen before any data. RELEASING §3.4 and §5c.*

This is the construction stack's recalibration (`bench-recalibrate-stack`,
`docs/journal/2026-11-22-recalibration-stack-registration.md`) run again on the decoder with
#739 (Wormhole's plateau stack) and #745 (the block reader), and widened at the cells where
Wormhole's handoff predicts the trims move. The design, the rule, the settle, the corpora and
the diagnostics are the registered ones (`docs/journal/2026-11-18-recalibration-registration.md`,
Researcher REVIEW 6978182129, its targeted correction applied at cee99296). Every cell and arm of
the stack run is kept. This document says what differs and gives in full the table each count
is judged against.

- **Recipe:** `just bench-recalibrate-plateau`, on `bench/recalibrate-plateau`.
- **Evidence path:** `target/recalibrate-plateau/run-<UTC timestamp>/`, bundled by
  `bench/bundle.sh`. The stack run's bundle is `recalibrate-stack/run-20261122T030603Z` on the
  `evidence` branch (9f203342, sha256 88d062e0…30b2); the diet run's is
  `recalibrate-diet/run-20261121T124719Z` (9326d856).
- **The untimed DRY preflight** (`just bench-recalibrate-plateau 4 12 24 24 0.25 5 1`) is kept.
  It is not evidence.

## Why §5c applies

Two changes to what a worker costs, both since the stack run's pin (53f7ecf9):

- **#739, Wormhole's plateau stack, merged at master 3296195e3104967d800e105d44c4c43d7fc01f14.**
  Owned run-route output, in-place RLE1, tiled CRC, a 12 MiB ready cap (`READY_BYTES`, with a
  floor of the workers and 2 more) and a ready ring. It cuts DRAM traffic and per-worker scratch.
  On the i9 at 10 threads (`bench-wormhole-plateau`, wormhole-plateau/run-20261122T085156Z,
  evidence a811364d, sha256 9faac67b…c92f): 0.963× wall, 0.937 CPU, peak RSS 0.63 (87 → 55 MiB).
- **#745, the block reader** (Carbon's cycle 37: `src/huff.rs`, `src/block.rs`, `src/bits.rs`,
  `src/stream.rs`, `src/runs.rs`), head 7b901b6789d98158d8f006e061599214365dc44b. That head is
  master 3296195e with the block reader merged in. On the i9 against the old base
  (`bench-parscan-c37`, parscan-c37/run-20261122T082138Z): 1 thread 0.977, 8 threads 0.884,
  10 threads 0.932. It is being re-qualified on top of #739 (#746,
  `bench-block-reader-on-wormhole`).

That is §5c.1's first case. The ready cap and the owned output change the memory a worker holds
(the term the memory gate reads). The run-route, RLE1 and CRC changes cut a worker's DRAM bytes
per output byte (the contention term). The block reader speeds the scanner, the serial front
end that caps the workers at 8 threads pinned. Every arm, lazy and eager, runs all of it.
`src/parallel.rs`, where the trims live, changed in #739 (the cap and the ring). Its trims did
not: `trim_cell`, `unpinned_cell` and the constants are 0.5.12's, and
`tests/recalibrate_arms.rs` passes at 7b901b67 (4/4).

## The decoder measured, and the re-pin

- **The pin:** the recipe refuses to run unless `src`, `Cargo.toml` and `Cargo.lock` match the
  pin byte for byte (`git diff --quiet $PIN HEAD -- src Cargo.toml Cargo.lock`). The pin is now
  **7b901b6789d98158d8f006e061599214365dc44b**, #745's head.
- **Re-pinned when #745 merges.** The coordinator re-pins to master's merge commit of #745, whose
  `src`, `Cargo.toml` and `Cargo.lock` are expected to be 7b901b67's: one line, `PIN=` in the
  recipe, and this paragraph. The pin check proves the trees match. If they do not, the recipe
  refuses to run.
- **If #745 fails to re-qualify,** the base is master 3296195e. Then `PIN=` is set to
  3296195e3104967d800e105d44c4c43d7fc01f14, and the branch's `src`, `Cargo.toml` and
  `Cargo.lock` are reset to it in one commit (`git checkout 3296195e -- src Cargo.toml
  Cargo.lock`). Then §"Why §5c applies" loses the block reader, and nothing else here changes.
- **The branch:** 7b901b67, with `bench/recalibrate-stack` (4792f7fc) merged in as a merge
  commit. The merge was clean, and it adds nothing to `src`. The harness (`examples/ab.rs`,
  `bench/two_arm.py`, `bench/counters.py`, their tests) is the stack run's, unchanged. What is
  new: this recipe, this registration, and three more checks in `tests/recalibrate_arms.rs`.
- **The build:** one, by rustc 1.99.1.

## What the arms can vary

`ab`'s occupancy arms are `mibz2-occ-N-S` (`StreamDecoder::workers(N, S, k, true)`, lazy) and
`mibz2-oce-N-S` (eager). Each sets the **workers** N and the **window** S, the blocks in flight.
The **ready cap** is not an arm's to set. `READY_BYTES` (12 MiB) is a constant, and every arm
and the default build with it (`with_inverse` → `with_inverse_capped(.., READY_BYTES)`, which
is private). So Wormhole's "10 workers with a 12 MiB cap" is the arm `mibz2-occ-10-S`, and S is
the one free knob. Two windows are natural for ten workers:
- **4 blocks a worker** (10/40): the trims' convention (`window(Plan::Workers(n), threads, slots,
  true)` scales the window to the workers);
- **4 blocks a thread asked** (10/48, 10/64, 10/80 at 12, 16 and 20): the plan's, the untrimmed
  decoder.

At 10 threads the two coincide (10/40). Under the cap, a wider window lets the scanner run ahead
only until 12 MiB of decoded blocks wait, so the two may now differ little. The run reports
them as a pair at 12, 16 and 20.

## The cells and arms, and the trim each count is judged against

The reference at each cell is the decoder `StreamDecoder::new(T, 4T, k)` builds on the measured
i9 at the pin. Added since the stack run: **bold**.

| placement | threads | shipped (reference) | −1 | +1 | the sweep | untrimmed | other kernel |
|---|---|---|---|---|---|---|---|
| pinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | **lazy 9/36** | lazy 10/40 | — |
| pinned | 8 | lazy 6/24 | lazy 5/20 | lazy 7/28 | — | lazy 8/32 | — |
| pinned | 6 | lazy 5/20 | lazy 4/16 | lazy 6/24 | — | eager 6/24 | (= untrimmed's kernel) |
| pinned | 4 | eager 3/12 | eager 2/8 | eager 4/16 (= untrimmed) | — | eager 4/16 | lazy 3/12 |
| unpinned | 4 | lazy 4/16 | lazy 3/12 | not run (above the threads) | — | eager 4/16 | (= untrimmed) |
| unpinned | 6 | lazy 6/24 | lazy 5/20 | not run | — | eager 6/24 | (= untrimmed) |
| unpinned | 8 | lazy 8/32 (untrimmed) | lazy 7/28 | not run | — | — | — |
| unpinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | **lazy 9/36** | lazy 10/40 | — |
| unpinned | 12 | lazy 7/28 | (at 10) | lazy 8/32 | **lazy 9/36, lazy 10/40** | lazy 10/48 | — |
| unpinned | 16 | lazy 7/28 | (at 10) | lazy 8/32 | **lazy 9/36, lazy 10/40** | lazy 10/64 | — |
| unpinned | 20 | lazy 7/28 | (at 10) | lazy 8/32 | **lazy 9/36, lazy 10/40** | lazy 10/80 | — |

So at unpinned 10, 12, 16 and 20 the sweep is 7, 8, 9 and 10 workers, 4 blocks a worker, with the
plan's window for ten beside it from 12. Pinned at 10 it is 6 to 10.

**The judged cells, as the recipe calls `two_arm.py`:**
- *Pinned* (`--ref-at mibz2-occ-7-28=10 mibz2-occ-6-24=8 mibz2-occ-5-20=6 mibz2-oce-3-12=4`),
  **13 cells**: at 10, lazy 6/24, 8/32, **9/36**, 10/40; at 8, lazy 5/20, 7/28, 8/32; at 6, lazy
  4/16, lazy 6/24, eager 6/24; at 4, eager 2/8, eager 4/16, lazy 3/12.
- *Unpinned* (`--ref-at mibz2-occ-7-28=10,12,16,20 mibz2-occ-8-32=8 mibz2-occ-6-24=6
  mibz2-occ-4-16=4`), **21 cells**: at 10, lazy 6/24, 8/32, **9/36**, 10/40; at 12, 16 and 20,
  lazy 8/32, **9/36**, **10/40** and lazy 10/4T; at 8, lazy 7/28; at 6, lazy 5/20 and eager
  6/24; at 4, lazy 3/12 and eager 4/16.

A reference at one count is a candidate at another, never at its own, as before.

**Reported, judging nothing** (`--pair`): at every count where both run, 9/36 over 8/32 and
10/40 over 9/36, so each step of the sweep has its own ratio. At 12, 16 and 20, 10/4T over 10/40:
the window, for ten workers under the cap.

**Checked deterministically:** `tests/recalibrate_arms.rs` holds this table. Under
`parallel::with_machine` with the measured i9's facts, each shipped arm is the default decoder's
worker count, window and kernel, and each untrimmed arm is the plan's. It adds three checks. 9/36
sits one worker from 8/32 and from 10/40. The ten-worker arms 10/40 and 10/4T differ in the window
alone. `READY_BYTES` is 12 MiB. It passes at 7b901b67 (4/4).

**Checked against the scorer:** both `two_arm.py` calls, with exactly the recipe's arguments,
were run on records synthesised from the stack run's bundle. The synthesis copied 8/32's records
as 9/36, copied 10/4T's as 10/40, and repeated the unpinned 10–20 rounds to 24. Both calls
parsed without a VOID, giving 13 and 21 verdicts and the pairs above. That checks the
arguments, and nothing else.

## Method

As registered: one `ab run` a cell with its own arms, the shipped trim first, rotated and
reversed round by round. Settle 0.25 s (`--max-busy` 1.5). Wikidata ×4 is the gain family. Text
and random are the guards, generated afresh by `ab gen`, and every input's SHA-256 is recorded.
The spinner is held at 4, 6 and 8 threads pinned.

**Rounds:** 12 paired rounds, 24 pinned at 8 as before. New: **24 (`ROUNDS_HI`) unpinned at 10,
12, 16 and 20** (`--rounds-at`), the cells of this run's question. There the stack run's 95%
intervals on Wikidata spanned about ±20% (lazy 8/32 over 7/28: [0.757, 1.157] at 10, [0.792,
1.209] at 12, [0.754, 1.128] at 16, [0.751, 1.140] at 20). Twice the rounds narrow them by about
1/√2, to about ±14% at the same spread. Holm's family is unchanged by the rounds.

**Time:** the stack run took 12 min 31 s in all (started 03:06:03, finished 03:18:34), for 492
arm-rounds (37 arms a round over 12 rounds, and 4 more for 12 more rounds pinned at 8), each of
three runs. That is 1.53 s an arm-round, build, data and scoring included. This run has
**828 arm-rounds**:
- pinned: 5×12 at 10, 4×24 at 8, 4×12 at 6 and 4×12 at 4, so 252;
- unpinned: 5×24 at each of 10, 12, 16 and 20, then 2×12 at 8, 3×12 at 6 and 3×12 at 4, so 576.

At the stack run's rate, that is **about 21 minutes**, and about 22 with the DRAM diagnostic's
two added arms and its cell at 20. The plateau stack makes each decode a few percent faster. The
settle and process start take most of a run's time, so the estimate stays.

**Dropped to fit about 25 minutes**, each with what it would add at 1.53 s an arm-round:
- *11 and 12 workers at unpinned 12, 16 and 20*, the other counts in Wormhole's PR 5 list
  (W ∈ {7, 8, 9, 10, 11, 12, 16, 20}). These are SMT workers, beyond a worker a core. They would
  add 144 arm-rounds, about 3.7 min, to about 25.5 min, and six more cells to the unpinned family.
- *Other windows:* ten workers at 2 a worker (10/20, a memory lever under the cap), and 7–9
  workers at the plan's 4T window. Four cells at 24 rounds come to 96 arm-rounds, about 2.5 min.
  Both questions are outside the hypothesis, and they would dilute Holm.
- *Unpinned 14 and 18 threads:* as in every recalibration, 12, 16 and 20 stand for 10–20.
- *24 rounds pinned at 10:* not this run's question. Pinned 10 stays at 12 rounds.

**Power:** the families grow from 12 to 13 cells pinned and from 14 to 21 unpinned. Holm's first
threshold unpinned goes from 0.025/14 to 0.025/21. The doubled rounds at the cells that grew
more than pay for it, for a gain of the same size. An INCONCLUSIVE at a gain of a few percent
remains likely, and it is no evidence of no difference.

## The rule

Unchanged (2026-11-18 § "The rule"):
- each placement is one Holm family at α 0.025 on Wikidata ×4, one-sided;
- the loss screen: no cell SLOWER on any of the three corpora;
- memory follows rule 4 as amended (#558), with the encoded trade (`--primary wikidata-x4,text`).
  No trade is allowed with two corpora above 1.10;
- the verdicts are ADOPTS, ADOPTS BY TRADE, HELD ON MEMORY, A FINDING NOT ADOPTED and NOT ADOPTED;
- at each count, the choice is the ADOPTing candidate with the lowest main-corpus step. If none
  ADOPTs, **the shipped trim in the table above stays**;
- VOID on every check `two_arm.py` makes, and on a record whose worker threads are not its
  arm's N (`--check-workers`). A VOID placement is not re-run on this registration;
- one run decides: no repeat, and no extension of the rounds.

**A change of trim is an adoption registration (C6).** This document is frozen before the data,
and is reviewed by the Researcher before the ready tag.

**The integration decision:** each count's worker count, window and kernel in `trim_cell` and
`unpinned_cell`, before the next release is tagged. A count whose choice is not the shipped trim
gets its own PR to master, with the change, a deterministic test of the new cell and this run's
figures. That PR is routed to the Researcher before any tag. The verdict goes in the release's
CHANGELOG section, changed or not (§5c.4). Frozen here for the counts this run does not measure,
since `unpinned_cell` covers every count from 10:
- *Between measured counts* (unpinned 11, 13–15 and 17–19): such a count takes a new choice only
  where the measured counts on both sides chose the same arm shape. That means the same workers,
  and the same window convention: 4 a worker, or the plan's 4 a thread asked. Otherwise it keeps
  lazy 7/28.
- *Above 20 threads unpinned:* such a count follows 20's choice.
- *A choice of 10/4T at a count* is the plan's own decoder there, an end to the trim at that
  count. A choice of 10/40 is a trim to ten workers at 4 blocks a worker.

**Memory, said before the data:** in the stack run, the ten-worker arms read 1.23–1.32 of 7/28's
peak memory on all three corpora. Under rule 4 as amended, two corpora above the gate never
trade. Ten workers can ADOPT only where #739's cap brings at least two corpora within 1.10 of
7/28's peak, and the third within the trade. Otherwise the cell is at best HELD ON MEMORY, the
Coordinator's to weigh, not adopted by this rule.

## The hypothesis, said before the data (deciding nothing)

**Wormhole's prediction, quoted as a hypothesis.** It is a prediction and not evidence: modelled
on a 2-vCPU Xeon and a Comet Lake roofline, not measured on the i9. From the final handoff
(`origin/research/carbon-wormhole` 8ec6f94c2,
`docs/journal/2026-11-21-wormhole-cycle50-round5-final-handoff.md`), wall W0:

> `trimmed_here` (`src/parallel.rs:2969-3035`) hard-caps **all** `T ∈ {10, 12, 16, 20}`
> unpinned to the exact same `7W/28` decoder (`workers(7, 28, k, true)`). […] **Recalibrate in
> PR 6 (C48)** after W1–W5 land: `10–20T` → **`10W / 12 MiB cap`** (1/phys core).

And its PR 5:

> After PRs 1–4 land, benchmark `W ∈ {7, 8, 9, 10, 11, 12, 16, 20}` on the i9-10900K […].
> Expected optimum: shift `unpinned_cell` for `T >= 10` from `7W/28` → **`10W / 12 MiB cap`**
> (1 worker per physical core, clamping SMT `12–20T` to `10W`).

**What that prediction assumes, which this pin does not have.** Wormhole's cycle-48 table
(`docs/journal/2026-11-21-wormhole-cycle48-multiregime-and-trim-recalibration.md`, §3) gives two
regimes:
- **Regime 1** is the serial scanner, with Wormhole's memory and sync work. It predicts **8
  workers** from 10 to 20 threads ("`8W` (`12 MiB` cap, 1 core left for serial scanner +
  reader)"), at about 61–64 ms.
- **Regime 2** adds Mechanisms' parallel scan (`C38b StreamParScan`). It predicts **10
  workers**, at about 39–43 ms.

The parallel scan entry point was rejected on the i9 (`bench-parscan-c37`, #742, Decision 1;
#741 open as not adopted). The block reader that #745 keeps speeds the serial scanner, and does
not remove it. This pin is therefore Regime 1 with a faster scanner. Read literally, Wormhole's
own model predicts 8 workers here, and 10 only once the scan is parallel. The brief's
hypothesis, 10 workers with the 12 MiB cap, is Regime 2's. The run judges both: 8/32 and 10/40
(and 10/4T) at every unpinned count from 10, with 9/36 between them.

**What this run can say about it:**
- *If ten workers win:* 10/40 or 10/4T ADOPTS at 12–20 (and at 10), and the memory gate holds
  under the cap. Regime 2's prediction would then hold without the parallel scan.
- *If eight or nine win:* the knee has moved by one or two workers, Regime 1's prediction.
- *If nothing moves:* the trim stays at 7/28, as in the diet and stack runs. The pairs say whether
  the sweep's direction moved, even where no cell is adopted.

**The trims less likely to move, as in the stack registration:**
- *Pinned 8 (lazy 6/24):* set by the front end's budget. The block reader eases that budget
  (0.884 at 8 threads against the old base), so 7/28 there is the cell this pin makes likelier
  to move than the stack's did.
- *Below 8 threads:* set by scheduling and the choice of kernel. The plateau stack speeds both
  kernels' common path (RLE1, CRC, output), so the kernel-switch arms should move little.

## Model first (§5c.3): MONODROMY's prediction

*Left for MONODROMY's frozen prediction, which the coordinator requests. It is to be filled in,
with its record number, branch and head, before the ready tag is set. It is cited in the queue
entry, and it decides nothing.*

Its inputs differ from its stack-run record's in three places, for MONODROMY to decide:
- *the working set:* the 12 MiB ready cap with its W+2 floor, and the owned run-route output
  with in-place RLE1 (no separate `ll` scratch). Peak RSS is 0.63 at 10 threads on the i9;
- *the traffic:* tiled CRC and in-place RLE1 cut DRAM bytes per output byte;
- *the front end:* the block reader's faster serial scan.

**The record:**

> *(MONODROMY's frozen prediction goes here: each judged count's predicted best arm, and the
> predicted ratio of 8/32, 9/36 and 10/40 over 7/28 at unpinned 10, 12, 16 and 20.)*

## Reported, and deciding nothing

**The memory controller**, as registered, where `kernel.perf_event_paranoid` ≤ 0:
- unpinned at 10: lazy 7/28 against 6/24, 8/32, **9/36** and 10/40;
- **unpinned at 20: lazy 7/28 against 8/32, 10/40 and 10/80** (ten workers on twenty CPUs, the
  clamp);
- unpinned at 6: lazy 6/24 against eager 6/24;
- pinned at 4: eager 3/12 against lazy 3/12.

**The diagnostics** (order, drift, load) are as declared on 2026-11-18, for every judged cell.

## What it does not decide

As 2026-11-18, it decides nothing about:
- any other machine, mask, quota or SMT placement;
- any arm with more workers than the threads asked, or more than ten;
- any ready cap but 12 MiB;
- any count other than these eleven cells, except under the integration rule above;
- `LAZY_FROM_THREADS`, or the plan off the measured i9.
