# The worker-count recalibration after the Open kernel's construction stack: the registration

*22 November 2026. Frozen before any data. RELEASING §3.4 and §5c.*

This is the memory diet's recalibration (`bench-recalibrate-diet`, #695: the 2026-11-18 design
re-pinned to 23e40f77) run again on master's head, for the release after 0.5.11. The design, the
rule, the rounds, the settle, the corpora and the diagnostics are the registered ones
(`docs/journal/2026-11-18-recalibration-registration.md`, Researcher REVIEW 6978182129, its
targeted correction applied at cee99296). The cells and arms are the diet run's
(`docs/journal/2026-11-21-recalibration-diet-registration.md`), the unpinned 4-thread reference
it introduced included. This document says only what differs, and gives in full the table each
count is judged against.

- **Recipe:** `just bench-recalibrate-stack`, on `bench/recalibrate-stack`.
- **Evidence path:** `target/recalibrate-stack/run-<UTC timestamp>/`, bundled by
  `bench/bundle.sh`. The diet run's bundle is `recalibrate-diet/run-20261121T124719Z` on the
  `evidence` branch (9326d856, sha256 643e6a32…3caa); 0.5.8's is
  `recalibrate-058/run-20261121T020723Z` (d353b473); 0.5.7's is
  `recalibrate-057/run-20261119T185157Z`; 2026-11-18's is `recalibrate/run-20261118T122744Z`.
- **The untimed DRY preflight** (`just bench-recalibrate-stack 4 12 24 0.25 5 1`) is kept from
  the diet's recipe. It is not evidence.

## Why §5c applies

#719, merged as **53f7ecf935d62ac7896eafaf55c5d249789614ca**, is the Open kernel's construction
stack (with #714, GradedHandle/Prefix15), in `src/open/` and `src/open.rs` only. It changes how
a lazy worker builds each block, and what it keeps between blocks.

What it changed, measured:
- **On the i9** (`bench-open-stack`, open-stack/run-20261122T022924Z, evidence 91f58375, sha256
  50c832fd…1ba), against master 8ff1187a, unpinned: wall 0.952 [0.944, 0.961] at 6 threads,
  0.987 [0.960, 1.015] at 8 (inconclusive) and 0.975 [0.967, 0.984] at 10; CPU 0.949–0.962;
  median peak RSS 0.97–0.99. Adopted.
- **Natively** (EPYC, in-process, six blocks): 0.872 at 1 thread and 0.880 at 4 workers, with
  the kernel's retained scratch about **−19%**.

That is §5c.1's first case: the release changes what a worker costs, in time and in scratch, so
the knee may move. Every lazy arm (`mibz2-occ-N-S`, `StreamDecoder::workers(.., true)`,
`OpenCells`) runs the construction-stack kernel. The eager arms (`mibz2-oce-N-S`, `Cells`) do
not change.

Between the diet's pin and this one, master also took #703 (dense admission, width
certificate) and #705 (Relay8), both in `src/open/`, both shipped in 0.5.11. They too are in
every lazy arm here. `src/parallel.rs`, where the trims live, is unchanged since 23e40f77.

## The decoder measured

- **The pin:** the recipe refuses to run unless `src`, `Cargo.toml` and `Cargo.lock` match
  master's **53f7ecf935d62ac7896eafaf55c5d249789614ca** byte for byte (`git diff --quiet $PIN
  HEAD -- src Cargo.toml Cargo.lock`). That was origin/master's head when this branch was made.
  The diet's pin was 23e40f77; 0.5.8's 70b18ca4; 0.5.7's 17405826; 2026-11-18's v0.5.5,
  7214f8ca.
- **The branch:** origin/master at 53f7ecf9, with `bench/recalibrate-diet` (d62890fb) merged in
  as a merge commit. The merge was clean. Its harness (`examples/ab.rs`, `bench/two_arm.py`,
  `bench/counters.py`, their tests) is the diet's, unchanged. It adds nothing to `src`.
- **The build:** one, by rustc 1.99.1.

## The cells: unchanged from the diet run

The diet run moved no trim (CHANGELOG, 0.5.10), and `src/parallel.rs` is unchanged since, so the
shipped trim at every cell is the one the diet run judged against. Every cell and arm is the diet
run's, the unpinned 4-thread cell's reference (lazy 4/16, from 0.5.8's #678) included. The cells
are 12 pinned and 14 unpinned, the runs 111 a round.

## The cells and arms, and the trim each count is judged against

The reference at each cell is the decoder `StreamDecoder::new(T, 4T, k)` builds on the measured
i9 at master 53f7ecf9.

| placement | threads | shipped (reference) | −1 | +1 | untrimmed | other kernel |
|---|---|---|---|---|---|---|
| pinned | 10 | lazy 7/28 | lazy 6/24 | lazy 8/32 | lazy 10/40 | — |
| pinned | 8 | lazy 6/24 | lazy 5/20 | lazy 7/28 | lazy 8/32 | — |
| pinned | 6 | lazy 5/20 | lazy 4/16 | lazy 6/24 | eager 6/24 | (= untrimmed's kernel) |
| pinned | 4 | eager 3/12 | eager 2/8 | eager 4/16 (= untrimmed) | eager 4/16 | lazy 3/12 |
| unpinned | 4 | lazy 4/16 | lazy 3/12 | not run (above the threads) | eager 4/16 | (= untrimmed) |
| unpinned | 6 | lazy 6/24 | lazy 5/20 | not run | eager 6/24 | (= untrimmed) |
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
  mibz2-occ-4-16=4`), 14 cells: at 10, lazy 6/24, 8/32, 10/40; at 12, 16 and 20, lazy 8/32 and
  lazy 10/S; at 8, lazy 7/28; at 6, lazy 5/20 and eager 6/24; at 4, lazy 3/12 and eager 4/16.

A reference at one count is a candidate at another, never at its own, as before.

**Checked deterministically:** `tests/recalibrate_arms.rs` holds this table, unchanged from the
diet's but for its header. Under `parallel::with_machine` with the measured i9's facts, each
shipped arm is the default decoder's worker count, window and kernel, and each untrimmed arm is
the plan's. It passes on master 53f7ecf9 (3/3).

## Method

As registered: one `ab run` a cell with its own arms, the shipped trim first, rotated and
reversed round by round; **12 paired rounds, 24 pinned at 8 threads**; settle 0.25 s
(`--max-busy` 1.5); Wikidata ×4 (the gain family), text and random (the guards, generated afresh
by `ab gen`), every input's SHA-256 recorded; the spinner held at 4, 6 and 8 threads pinned.

**Time:** 111 runs a round, as before. The diet run took 12 min 38 s in all (started 12:47:19,
finished 12:59:57). The construction stack makes the lazy arms 1–5% faster, so **about 12–13
minutes** on the i9.

**Power:** as registered (2026-11-18 § "Power"). The rounds, families and cells per family are
unchanged, so the detectable gains are too. The diet run's intervals at the cells where this
run's question is sharpest were wide: lazy 8/32 over 7/28 spanned [0.738, 1.129] pinned at 10
and [0.737, 1.073] to [0.818, 1.296] unpinned at 10–20. An INCONCLUSIVE there is the expected
outcome for a gain of a few percent, and no evidence of no difference.

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

**What the construction stack changes for a worker.** A lazy worker builds each block faster
(0.872–0.880 natively, 0.952–0.987 of wall on the i9) and keeps about a fifth less kernel scratch
between blocks. Both lower what one more worker costs: less time on the shared front end's
budget per block, and less of the i9's 20 MiB L3 (`TEN_CORES`) held per worker. The scratch cut
is smaller than the diet's (−19% of what was already 1,200 KiB natively, against the diet's
0.283×), so the direction is the same as the diet's but the step is smaller. This is a
direction, not a forecast of the knee.

**The trims most likely to move:**
- **From 10 threads, lazy 7/28: the trims set by L3 contention.** These are the cells the
  model's contention term decides (β·W·CPU(W), records 45–56, 83, 87 and 88), and where
  `LAZY_FROM_THREADS`' own account says "many workers contend for the cache". If a smaller,
  faster worker contends less, one more worker may now fit: lazy 8/32 over 7/28 would move below
  1, and its memory ratio toward the gate. The rule needs both for an adoption.
- **The unpinned 10–20 thread cells especially.** In the diet run lazy 8/32 over 7/28 read
  0.8894 [0.7371, 1.0732] at 10, 0.9345 at 12, 0.9864 at 16 and 1.0292 at 20, all INCONCLUSIVE,
  with memory 1.06–1.10 on Wikidata and 1.13–1.14 on text; the plan's ten workers (10/40 to
  10/80) read 0.93–1.03, memory 1.21–1.36. Pinned at 10, 8/32 read 0.9126 [0.7377, 1.1288],
  memory 1.088 (1.134 on text). Lower per-worker scratch would lower those memory ratios
  most where the arms carry the most workers.

**Less likely to move, and why:**
- **The prior recalibration moved nothing.** The diet run (recalibrate-diet/run-20261121T124719Z)
  rejected no alternative in either family, and every reference stayed. MONODROMY's record 88,
  frozen before its data, predicted exactly that. The construction stack's step is smaller than
  the diet's in scratch, and of a like size in wall, so the prior for this run is the same: no
  trim moves.
- **Pinned 8 (lazy 6/24):** on the front end's budget, not contention (MONODROMY's scoring of
  record 83; record 88 §1 puts the front end as a serial cap that binds sooner as workers get
  faster). The diet run read 7/28 and 8/32 at 1.0656 and 1.0651 there, both SLOWER. A faster
  kernel favours fewer workers here, not more.
- **Below 8 threads (pinned 6 and 4, unpinned 8, 6 and 4):** set by scheduling and by the choice
  of kernel, not by cache contention. The counts are unlikely to move. The kernel-switch arms
  may: the stack speeds the lazy kernel alone, so the eager arms at unpinned 4 and 6 and pinned 6
  should lose by more. The one eager shipped trim, **pinned 4 (eager 3/12)**, had lazy 3/12 at
  1.0203 [1.0042, 1.0367] SLOWER in the diet run. A lazy gain of 3–5% at 4 threads would bring
  it to about 0.97–0.99, near the line.

## Model first (§5c.3): MONODROMY's prediction

**The model's records are in the repository, but not on master.** Record 88,
`docs/journal/monodromy/2026-11-21-recalibration-diet-predictions.md`, with its script
`2026-11-21-recalibration-diet-predictions.recaldiet.py.txt`, is on MONODROMY's branch
`claude/sharp-meitner-cx0v78` (head 52b3de87 when this was written). It scores record 87 against
0.5.8's run and predicts the diet run: the diet cuts contention to 0.73×, and no trim moves.

**As before, MONODROMY's prediction is frozen before the data,** as a new record for this run,
and cited in the queue entry before the ready tag is set. Its inputs differ from record 88's in
two places, both for MONODROMY to decide:
- *the kernel's speed:* #719's lazy workers are faster again (0.952 / 0.987 / 0.975 wall at
  6 / 8 / 10 threads unpinned; #703 and #705 are also in this pin);
- *the working set:* the retained scratch about −19% natively, a smaller step than the diet's.

**Not found:** a scoring of record 88 against the diet run (`recalibrate-diet/run-20261121T124719Z`).

## Reported, and deciding nothing

**The memory controller**, as registered, at three cells, where `kernel.perf_event_paranoid`
≤ 0:
- unpinned at 10: lazy 7/28 against 6/24, 8/32 and 10/40;
- unpinned at 6: lazy 6/24 against eager 6/24;
- pinned at 4: eager 3/12 against lazy 3/12.

**The diagnostics** (order, drift, load) are as declared on 2026-11-18, for every judged cell.

## What it does not decide

As 2026-11-18: no other machine, mask, quota or SMT placement; no arm with more workers than
the threads asked; no count other than these eleven cells; not `LAZY_FROM_THREADS` or the plan
off the measured i9.
