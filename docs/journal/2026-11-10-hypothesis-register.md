# 2026-11-10: The hypothesis register, redone: 420 IDs read closely, and what the reading forces

**Status: a close reading of the record, a law fitted and tested on archived workstation data,
and corrections to this author's entries of 9 and 10 November. No new run. Nothing asks for a
build slot; each avenue carries its cheapest falsifier for the Optimiser's gate.**

- **The question** (Louis): redo the 30 October register at its scale, and mine it: which
  verdicts rest on premises that no longer hold.
- **The register:** 420 rows in `2026-11-10-hypothesis-register/`: the old 189 P and 63 K IDs,
  76 new ones from 31 October to 6 November (Q), 115 from 7 to 10 November (R). Each row has
  the premise its verdict rests on and whether that premise still holds.
- **Forced by it, 1:** on the i9 the slow mode follows an idle gap before a run. Measured on
  28 October, not pursued, and contradicted by this author on 10 November.
- **Forced by it, 2:** on the i9 at one thread, instructions do not buy time. Three releases of
  kernel gains moved one-thread wall by 0.991, 0.985 and 1.011.
- **Forced by it, 3:** a law. A worker's inversion time a row is its own cost plus a penalty
  that depends only on the table bytes all workers hold live. Fitted on production's grid, it
  predicts the 24-bit table's grid within 13%.
- **Forced by it, 4:** the k-step table is not this author's design D. It is the register's N1,
  counted on 233 blocks at 0.90 MB a block, with its lane rule checked and its build settled.
  Every verdict on it was priced in instructions on CI. On the law it is worth 3.3 to 5.3 GB/s
  of inversion at ten workers against the cooperative plan's 2.4 to 2.6.

Labels: **verified** (this author re-read the primary source or recomputed from the archived
bundle), **reader's** (a section reader's finding, not re-checked), **derived**, **model**.

## 0. What is inherent, and what is the representation's (claim, proved)

- **Inherent:** n bytes are stored. That is n ÷ 64 lines, in any order, by any number of
  threads.
- **Not inherent: n dependent loads.** The walk is list ranking on one cycle. Its chain is a
  property of following Ψ a row at a time.
- **The equivalence that removes it.** Let K[q] be the number of steps rows q − 1 and q stay
  adjacent. Rows with K ≥ k strictly between them form a cell: one k-byte word, and Ψ^k a
  translation on it. The cells number r_k = 1 + #{q : K[q] < k}, and a walk over them takes
  n ÷ k dependent steps.
- **The cells cost no chain to make.** A_2k = A_k ∪ Ψ^(−k) A_k, where A_k is the set of level-k
  cell starts: the overlay of two sorted partitions of the rows, a merge. The companion to the
  old register holds this as "mathematics settled" (§3.1 k, c′), with 0.90 to 0.94 of parents
  passing a doubling unsplit on 40 sampled blocks (§3.1 h). **Verified** there.
- **What bounds k is the words:** a table is r_k (k + c) bytes. Counted on Wikidata: r_k ÷ n is
  0.088, 0.109, 0.142, 0.196 at k = 2, 4, 8, 16. Under 1.5 MB a block that is k ≤ 8: at least
  one dependent load an eight bytes.
- **And the tall tunnels lower that:** 48% of the bytes sit in blocks 33 rows high or more
  (`2026-11-10-tunnels.md`), copied with no table access: about 0.074 loads a row at k = 8.
- **So the chain a row is born of the T-vector.** What is left of it under a table of 1 to
  1.5 MB is a twelfth to an eighth of a load a row.

## 1. The law of the knee (fitted, then tested out of sample; verified)

`coop-20261029T215136Z` on `origin/research/compact-psi`: the i9, plain workers, one core a
block, the inversion alone. Two tables in one run: `p`, production's (3.6 MB), and `u`, a 24-bit
link (2.7 MB) that costs 1.466 times `p` at one block.

**The law:** a worker's time a row = g + a · X(L).
- g: the table's own cost at one block, in units of production's.
- L: the table bytes live across all workers, W × s.
- a: its dependent loads a row, in units of production's one.
- X: the machine's penalty, read from `p` alone:

| L, MB | 3.6 | 7.2 | 14.4 | 18.0 | 21.6 | 28.8 | 36.0 |
|---|--:|--:|--:|--:|--:|--:|--:|
| X | 0 | 0.015 | 0.489 | 1.092 | 1.962 | 5.244 | 8.628 |

**Tested on `u`, which the fit never saw:**

| workers | 1 | 2 | 4 | 5 | 6 | 8 | 10 |
|---|--:|--:|--:|--:|--:|--:|--:|
| measured, MB/s | 371 | 742 | 1,313 | 1,479 | 1,615 | 1,213 | 805 |
| predicted | 371 | 738 | 1,266 | 1,434 | 1,446 | 1,269 | 924 |
| measured ÷ predicted | 1.00 | 1.01 | 1.04 | 1.03 | 1.12 | 0.96 | 0.87 |

- **The penalty is additive and depends on live table bytes alone.** A dearer step does not
  shrink it or hide it.
- **`u` was the proof of share-nothing, eleven days early.** It lost at one block (0.681
  against a gate of 0.85) and was closed for the decoder. Its best, at six workers, beat
  production's best at four by 1.100 (1.093–1.108). The entry's own words: "does a smaller
  table alone move the knee? Yes."

**What the law predicts, same harness (model):**

| table | g | a | best, MB/s | at |
|---|--:|--:|--:|--:|
| production, plain workers (measured) | 1 | 1 | 1,461 | 4 |
| production, three teams of three (measured, the archive check) | | | 2,430 to 2,584 | 9 |
| compact30, 1.95 MB (g is the reader's) | 1.18 | 1 | 2,350 | 7 |
| N1 at k = 4, 0.90 MB | 1.0 to 1.5 | ¼ to ½ | 3,470 to 5,260 | 10 |
| N1 at k = 8, 1.54 MB | 1.0 to 1.5 | ⅛ to ¼ | 3,270 to 5,030 | 10 |

- **Limits:** one run of 29 October, the kernels of that day, the inversion only. The k-step
  rows assume the penalty scales with loads a row; no k-step table has run. Other per-worker
  buffers are inside X as they were in that harness (**reader's** caution: at the decoder's
  boundary 62% of ten plain workers' cold lines are not the table's).

## 2. The i9 does not pay for instructions at one thread (recomputed from the archived bundles; verified)

`bench/release-scaling/`, the i9, 10 paired rounds, new release over old, wall:

| threads | 0.2.4 over 0.2.1 | 0.2.5 over 0.2.4 | 0.2.6 over 0.2.5 |
|---|--:|--:|--:|
| 1 | 0.991 (0.979–1.003) | 0.985 (0.968–1.003) | 1.011 (0.996–1.026) |
| 4 | **0.846** (0.832–0.860) | 1.003 | 0.995 |
| 8 | 0.966 | 0.992 | 1.007 |
| 10 | 0.994, CPU 1.110 | **0.893**, CPU 0.853 | 0.964 (0.918–1.012), CPU 1.094 |

One thread: 403, 403, 400 MB/s. Ten: 1,274, 1,415, 1,503.

- **Nine decoder merges that CI's one-thread row accepted bought nothing at one thread here**
  (the reader gives CI's figure for them as −15.4% on an EPYC 7763; not re-checked).
- **They bought 15% at four plain workers,** where the tables press on the cache.
- **They bought nothing at ten,** where the default plan runs kernels they never reached (§3.2).
- **Ten threads moved twice, both times by a change of plan:** three teams (0.893), then the
  pipeline (0.964, not resolved, with 9% more CPU).
- **So on this machine the currency is dependent loads and live bytes.** A design's instruction
  count ranks it on CI's runners and says little here. E's 3.67× is a CI figure; its own
  registration marked other machines "not shown".

## 3. What the reading found, each checked against its source

### 3.1 The slow mode follows an idle gap (verified)
`2026-10-28-workstation-scaling.md` §4, `just bench-regimes`, 6 threads: 5 of 12 runs slow with
a 0.15 s gap before the run, 0 of 12 with none; the slow runs at about 1.5× the CPU a block.
"Recorded, not pursued." The gap returned with one fresh process a run. This author's "Not the
gap" (`2026-11-10-two-modes.md`) read a bundle whose gap never varied. The par-scan bundle's 180
back-to-back cells show one mode.
- **Falsifier, free:** `just bench-regimes --threads 10`, the default plan.

### 3.2 The team path runs the kernels of October (verified in the source)
| stage | one worker | the team plans | recorded? |
|---|---|---|---|
| MTF | #170, #172, #207 | none of them | C29, inconclusive |
| build | #312 | the 16-row path only | no |
| walk | staged (#171) | unstaged | 4 November: "A port waits" |
| output | over L's buffer (#164) | off | its 6-to-10-worker bar never run |

- **Bound, from §2:** what plain workers got at four threads, 15%. Not the 30% the instruction
  counts suggest.
- **Falsifier:** a callgrind count of the default plan at this head; then the 9-thread sweep.

### 3.3 N1 is design D, already counted (verified in the companion, §3.1)
- **Size, 233 blocks, displacement records:** 0.79 / 0.84 / 0.90 / 1.42 / 2.04 MB at k = 4
  (min, p10, median, p90, max); 1.29 / 1.43 / 1.54 / 2.58 / 3.44 at k = 8.
- **Lanes:** capped records that store their own length; 10,188 segments by brute force, no
  failure.
- **Fallback:** records, random and the crate's own sources fail the storage gate (11.0, 11.0,
  4.45 MB at k = 8); the rule is by r, known after MTF.
- **Why it stopped:** its gate was 0.67 of production's inversion at one thread, on a model in
  instructions. Under §1 and §2 that gate asks the wrong machine the wrong question.
- **This author's `2026-11-10-backward-table.md` §4a did not cite it.** The spec's first rule
  is to start from the record.

### 3.4 The gate for a small table has had four values (reader's; the four are in the record)
0.67× (the companion), parity (8 November), 1.44× (this author, §3a of share-nothing), about
2.5× (the same entry, §3b), and nearer 2.1× on one harness (its correction). §1 replaces the
gate with a prediction a table: its g, its a, its bytes.

### 3.5 Reader's findings worth a falsifier, not re-checked
- **compact30 is built, 1.95 MB, and was never run with plain workers on the i9.** The law says
  2,350 MB/s at seven: a tie with three teams of three, with no cooperation.
- **At 1.2 MB the old ladder has ten workers behind eight** (1,604 against 1,729, level 3).
- **The emitter has three prices that do not reconcile:** 17% dearer than MTF on one CI run, 6%
  cheaper on another, 2.4× faster locally.
- **The application runs four plain workers** (wikidata-pq's cap). At four the table is no
  constraint and CPU a byte is the cost.
- **#183 (the fused post-walk pass) and #180's window** were judged on CI at four threads and
  never varied alone on the i9.

## 4. This author's errors, found by the reading
| what was written | where | what the record held |
|---|---|---|
| "Not the gap" | two-modes §1 | the gap experiment of 28 October |
| design D as new | backward-table §4a | N1, the companion's §3.1 |
| D's lanes "open" | share-nothing §3a | capped records, checked |
| D's build "E's shape" | share-nothing §3a | a merge; 94% unsplit |
| C and E judged at 1.44× | share-nothing §3a | replaced by §3b the same day; the table not redone |
| "cooperation costs 42% more" | share-nothing §1 | mostly kernels not ported |
| break-even 2.5× | share-nothing §3b | two harnesses mixed |
| the walk's port "not in the record" | REVIEW 6758209736 | the old register's §6 |

Each is corrected in its entry or here.

## 5. The register
`2026-11-10-hypothesis-register/`, one file a section, as the readers wrote them:

| file | covers | rows |
|---|---|--:|
| `section-a.md` | (a) dependent transitions a byte: P01 to P16, N1 | 17 |
| `section-b.md` | (b) transition cost: P20 to P68 | 48 |
| `section-c.md` | (c) work that is not the walk: P70 to P89 | 20 |
| `section-def.md` | (d) scheduling and the knee, (e) measurement, (f) the application; K1 to K63 | 144 |
| `section-q1.md` | (g) entered 31 October to 6 November: Q01 to Q76 | 76 |
| `section-r.md` | (h) entered 7 to 10 November: R01 to R115 | 115 |

- **Each row:** the idea, the advantage as a quantity that could be false, its status, whether
  a construction or the advantage was rejected, the whole-loop price where the record has one,
  the cheapest falsifier now, its bearing on share-nothing, the premise the verdict rests on,
  and whether that premise still holds.
- **Each file ends with** "Mined" (at most twelve findings, each a sentence in the record set
  against a later one), "Changed since 30 October" and "Reopen candidates".
- **Statuses over all rows:** open 156, validated 73, rejected 76, conditional 35, superseded
  33, shipped 31.
- **How far to trust it:** the readers were told to copy figures only from files they read and
  to mark inferences. §1 to §3.3 here were re-checked; the rest was not. A row is a pointer to
  its sources, not a finding.

## 6. The avenues the reading leaves open, cheapest falsifier first

| # | avenue | what it could be worth | cheapest falsifier | gate |
|--:|---|---|---|---|
| 1 | the gap | whether every gapped i9 comparison read an idle artefact | `bench-regimes --threads 10`: an existing recipe | the Coordinator's discriminator |
| 2 | the law on a built table | share-nothing at level 9 with no new kernel | compact30 with 6 to 10 plain workers in the grid harness: 2,350 MB/s at seven, ±13% | the Optimiser's price of the arm; then the queue |
| 3 | N1 at k = 4 | 3.5 to 5.3 GB/s of inversion at ten workers (model) | the counting probe now running, read in loads a row as well as instructions | 3a |
| 4 | the team path's ports | up to 15% at ten threads | a callgrind count of the default plan at this head | 3a, then the 9-thread sweep |
| 5 | tall tunnels in a k-step lane | 0.125 to 0.074 loads a row at k = 8 | one counter in `phrase_census` | 3a |
| 6 | the reader's `BufRead` | the next serial stage past about 5 GB/s | none needed | the Coordinator's schedule |

## Reproduce

```text
git show origin/research/compact-psi:docs/journal/2026-10-29-compact-psi/coop-20261029T215136Z.md   # §1's grid
bench/release-scaling/2026-11-06-…i9…/runs.jsonl, …-0.2.5/, 2026-11-07-…-0.2.6/                      # §2: paired by round, new over old
git show origin/docs/research-portfolio:docs/journal/2026-10-30-research-portfolio.md               # the companion, §3.1
git show origin/docs/research-portfolio:docs/journal/2026-10-30-research-portfolio-register.md      # the old register
```

§1's fit: X(W × 3.6) = W × p₁ ÷ p_W − 1 from `p`'s medians, linear between its points; `u`
predicted as W ÷ (1.466 + X(W × 2.7)) × p₁.
