# 2026-11-17 · Pathfinder · The fused post-walk on Cells: its i9 adoption registration, corrected, frozen before data

Direction: the Coordinator's QUEUED 6946051925 on #145, which ports #183's fusion to Cells, counts it, then measures it on the i9. The registration was frozen before any data:
- first posted as REVIEW 6946436207;
- corrected by the Researcher's REVIEW 6947023492: three targeted corrections, then PROCEED with the experiment unchanged.

This file is that registration with the three corrections made. The code and recipe it registers are this branch's at `2bcd82721d637c7e21326006b8475c84a57e9fd1`, unchanged here: `just bench-fused`, its arms and `bench/two_arm.py`.

## What it decides

Whether production's decoder undoes RLE1 with the block's CRC folded in as it goes, wherever it runs lazy workers. Nothing else changes: the plan, the window, the threads and the kernel are master's (0.5.3, `77fa4cd3`).

## The port, and the count

**What fuses:** Cells' output is contiguous, so #183's segment chain has nothing to fuse here. What's left is the CRC. Master undoes RLE1 into the output, then reads the whole output back for `crc32`. That is about 0.5 MB a Wikidata block, past a 256 KiB L2 by then.

**The port** (#183's `crc::Running` and `update_with`, with a new `rle::unrle_crc`) folds the CRC in every 16 KiB as RLE1 appends, while those bytes are still in L1.

**The arms:**
- **`bwt::FusedFinish<LazyCells>`** is `LazyCells` with `decode_block` taking `finish_fused`.
- **Every other kernel's finish is master's.** `finish` and `decode_block::<LazyCells>` are instruction for instruction master's.

**The counts,** per output byte of `corpora/wikidata.bz2`, on `ab child … 1` under rustc 1.99.1. Cachegrind models the i9's cache: a 32 KiB 8-way L1d, with the 256 KiB 4-way L2 as the last level.

| | master | fused | change |
|---|--:|--:|--:|
| the CRC pass's L2 read misses | 0.0157 | 0.0003 | |
| the CRC pass's L1 read misses | 0.0157 | 0.0036 | |
| the process's L2 read misses | 0.3077 | 0.2920 | −5.1% |
| the process's L2 misses, reads and writes | 0.4343 | 0.4182 | −3.7% |
| the process's instructions | 27.47 | 27.56 | +0.33% |

The bytes and the CRC are the same on every block.

## The run (`just bench-fused`)

**The arms:** production's decoder from two builds, both by rustc 1.99.1.
- *`mibz2@build-master`:* master at `77fa4cd34e6ceab28d172e5dfed1ef3aafa7be03` (0.5.3), which this branch has merged.
- *`mibz2-fused`:* this commit's `BzReader::parallel_lazy_as::<FusedFinish<LazyCells>>`, production's plan, window and threads with only the finish fused.

**The cells:** 10 paired rounds.
- *Corpora:* 4 copies of the Wikidata sample (`wikidata-x4`, the gain family), and text (`ab gen` at scale 16) as a loss and memory guard.
- *Judged:* 10 and 12 threads asked. No controls, and no spinner.

**What each judged cell runs (correction 1):**
- **10 threads:** both arms are **lazy 10 workers with 40 in flight**. `ab run --spread` pins a run of 10 threads to CPUs 0–9. `target_topology` reads `Cpus_allowed_list` and sees 10 logical CPUs, so production's trim to seven declines.
- **12 threads:** unpinned, both arms are #524's **7 workers with 28 in flight**.

So the two cells cannot separate worker count from affinity. Each compares the two finishes like for like.

## The rule (`bench/two_arm.py`, as #506's), unchanged

- **A cell's step:** the rounds' paired wall ratios, `mibz2-fused` over `mibz2@build-master`: geometric mean and 95% interval (Student's t on the logs). FASTER below 1, SLOWER above, else INCONCLUSIVE.
- **Gain:** Holm's one-sided tests over the two Wikidata cells, family α = 0.05; established if Holm rejects at one or more.
- **Loss screen:** no judged cell, Wikidata or text, SLOWER.
- **Memory gate:** at every judged cell, the median over the rounds of each run's peak RSS, fused over master, at most 1.10.
- **Verdicts:**
  - *ADOPTS:* gain, no SLOWER cell, memory gate passed.
  - *HELD ON MEMORY:* gain and no SLOWER cell, memory gate failed; the Coordinator's under rule 4.
  - *A FINDING, NOT ADOPTED:* gain, but a cell SLOWER.
  - *NOT ADOPTED:* otherwise.
- **VOID** (exit 1, no verdict), before any verdict is computed:
  - *the records:* the file missing or a line not JSON; a key duplicated, outside the roster, or missing (80 at R = 10); a run not `success` or with an `err`; wall, CPU or peak RSS not finite and positive; an output's size or hash missing, or a corpus's outputs differing;
  - *the machine* (`ab env`): not 10 cores and 20 CPUs with each core's CPUs listed and no quota;
  - *a record's pinning:* not one CPU on each of t cores up to 10 threads, or not unpinned past them;
  - *a spinner* recorded anywhere.

## What an i9 ADOPTS means (correction 3)

An ADOPTS here is evidence on the i9 only. By itself it does not qualify an all-host default.

A head that makes the fused finish production's default (`LazyCells` fusing its finish) goes through one of two routes afterwards:
- the CI performance policy (`docs/team/PERFORMANCE-POLICY.md`) and B5 review;
- a forward C6 registration.

Eager Cells, the teams and the service core keep master's finish in any case.

## Deciding nothing

**The context pair:** the same two finishes through `mibz2-workers-lazy@build-master` against `mibz2-workers-fused`, at 10 threads, on both corpora, for 10 rounds.
- *What it measures:* it repeats the judged 10-thread cell's lazy 10/40 through another constructor.
- *Its standing:* it stays context, not a control, and voids nothing.

**The L2 diagnostic (correction 2):**
- *What:* `l2_rqsts.miss` and `l2_rqsts.references` a byte, with cycles and instructions, at 10 threads (lazy 10/40, as above). Fused against master's build, 10 reps (`bench/counters.py --events l2`), skipped and said so if perf can't count.
- *Its standing:* diagnostic only.
- *What a non-fall would not show:* if aggregate L2 misses a byte don't fall, that does not prove the CRC's reads were already hitting L2 on the i9. Other misses in the process could move with it.

**The expectation (M, not part of the rule):**
- *Wall at each judged cell:* 0.985 [0.96, 1.00].
- *L2 misses a byte:* down about 3–6%.
- *Text:* about flat.

## Checks before the run

- **End to end on a 4-core VM** at `2bcd8272`:
  - *the records:* 16 judged and 8 context, each its full roster;
  - *the verdict and the context:* both VOID on the machine record, as they must be off the i9;
  - *the L2 diagnostic:* skipped and said so;
  - *the outputs:* every one byte-identical.
- **CI at `2bcd8272`:** every check run green, and pre-commit.ci too.
