# The open kernel's port for 0.5.7: the C6 adoption registration

*19 November 2026. The Pathfinder's C6 adoption registration, frozen before any data, for the
Researcher's review through the Coordinator.*

- **Assigned:** #145 CHECKPOINT 7004759994 on #314. It dispatches QUEUED 6992388066 (B5),
  re-scoped to open.
- **The Coordinator's decisions folded in:**
  - *the admission,* accepted after REVIEW 7005084600;
  - *held-out guards with explicit bounds,* from the correction of 19 November;
  - *the gate's cost reported at each W,* with ungated open as context;
  - *Louis's rule:* adaptive dispatch is fair, changing the measurement is not
    (PERFORMANCE-POLICY §2, #626).
- **The change:** the port, #630 (`perf/open-kernel`), at
  9743bbe0d51ba451b8b8ab9e6fd731c238940052. Its B5 evidence is `docs/journal/2026-11-19-open-port-b5.md`; the
  Guardian's D115 gave G1 PROCEED on it (7007260113). The pin moved from 3f09500e, the head D115
  reviewed, only for D115's two scoped corrections (Miri's path filter and the journal's memory
  claims); no source file differs between them.
- **The recipe:** `just bench-open-port`, on `bench/open-port`, with the port's head pinned in the
  recipe as `PORT`.
- **What it decides:** whether 0.5.7 ships `OpenCells` as the workers' kernel wherever 0.5.6 ran
  `LazyCells`. If not, 0.5.7 ships without the change.

## The change measured

**`OpenCells`:**
- *Where it goes:* the open kernel on a run-route block that passes `LazyCells`' two shipped
  record-count gates, density r/n < 0.15 and sparse r/n ≥ 1/64. `LazyCells` as shipped decodes
  every other block. There is no new constant.
- *Where it runs:* wherever 0.5.6's default ran lazy workers. On the i9 unpinned, that is 8
  workers at 8 threads and 7 from 10 to 20.
- *What it leaves:* the plan, the windows and the eager cells below 8 workers are unchanged.

**Why the gates** (`docs/journal/2026-11-19-open-admission.md`, the development set):
- *The census:* one thread, every block, production's route against the open kernel's.
- *What it found:* the open kernel is faster only on the blocks `LazyCells` admits (Wikidata
  0.972), and slower on every block those two gates refuse: text 5.1×, pattern 1.55×, runs 1.6×,
  onebyte 1.9×.
- *The statistic and its mechanism:*
  - *the statistic:* r/n, the block's run records per row, read before any allocation;
  - *the mechanism (interpreted):* the open kernel's construction costs per run record (its strips and its
    elimination), so its advantage over `LazyCells`' induction holds where runs are long, and is
    lost where runs are few rows long (text) or where whole blocks are a few runs (pattern,
    onebyte), the case `LazyCells` itself leaves to `Segmented`.

## The decision run

**Two builds, each in its own worktree, both with the same toolchain (rustc 1.99.1):**
- *the port's head (`PORT`);*
- *v0.5.6 (530ed404).*

**Both run through the same public entry point:** each build's own `ab child mibz2`,
`BzReader::parallel`, the default a user gets. The two differ in the change alone; the reference
is the release, not a kernel-only arm. That avoids the construction constant record 71 found.

**The harness only schedules.** This branch's `ab run --unpinned` sets the CPUs, the 2 s gate and
the order, and runs every arm as a child of one build or the other (`mibz2@<port's ab>`,
`mibz2@<v0.5.6's ab>`). No code of this branch's is timed. (Master's `ab` has no `--unpinned`,
which the dry run found.)

**Reference and candidate admission:** both must observe the shipped workers, 8 at 8 threads and 7
at 10 and 20, on the registered machine (`lscpu` names an i9-10900K; `ab env` 10 cores, 20 CPUs).
Anything else VOIDs the run.
- *Where the workers are read:* on the Wikidata corpora (the primary, the held-out pair and the
  sustained cell).
- *Why there alone:* a guard's decode of a few milliseconds can end before the thread sampler
  sees every worker. The dry run showed it on `runs`. An arm builds the same decoder whatever the
  corpus, so the long decodes settle it.

**The cells:**
- *the decision:* unpinned at 8, 10 and 20 threads, 12 paired rounds, the 2 s settle gate;
- *sustained:* 10 threads, back to back (no gap), on Wikidata ×16, 8 rounds.

**Corpora:**

| set | corpora | role |
|---|---|---|
| development (the census's) | `wikidata-x4` (the pinned sample ×4) | **primary** |
| development | text, random, records, runs, pattern, onebyte (`ab gen`, afresh) | guards |
| **held out** | `ho-wikidata-l9`: 12 new windows of the same dump (`bench/wikidata-heldout.json`, at (4i + 3)/48, between the sample's), Wikimedia's own blocks | **replication** |
| **held out** | `ho-wikidata-l1`: the same bytes at level 1 (100k blocks) | guard |
| **held out** | `ho-prose-l1`, `-l9`: the journal's prose, every `docs/**/*.md` and `*.txt` (7.0 MB) | guards |
| **held out** | `ho-code-l1`, `-l9`: other code, every `*.py`, `*.just`, `*.yml` and `docs/**/*.asm` (2.7 MB) | guards |
| **held out** | `ho-structured-l1`, `-l9`: structured data that isn't Wikidata, every `docs/**/*.json`, `*.jsonl`, `*.csv` and `bench/*.json` (8.9 MB) | guards |

- *Determinism:* the held-out prose, code and structured sets are built by `bench/heldout.py`
  from the frozen commit's own tree, so their bytes are fixed by it. The small ones run as 4
  copies, as Wikidata ×4 does.
- *Never timed before this freeze:*
  - the held-out Wikidata windows were fetched and checked against libbzip2 and lbzip2 only;
  - nothing in the held-out set has been timed under any arm.

**Time:** about 41 minutes for the decision cells, 1 for the sustained cell, 15 for the context,
3 for the census and 3 for the builds. About **65 minutes** in all.

## One confirmatory attempt (the Researcher's correction, REVIEW 7007213171)

**This registration allows one timed attempt on the i9.** Its bundle or failure record is kept,
whatever happens.
- *A VOID* gives no adoption evidence.
- *An inconclusive required gain* (P1 or H missing its cutoff) is NOT ADOPTED.
- *Neither opens a retry* under this registration. A different experiment would need its own
  prospective review, and would leave this attempt and its outcome standing.
- *Untimed setup* may be repaired before the attempt starts.
- *The recipe enforces it.* A non-dry run refuses to start if a timed attempt already started in
  this checkout, or if an `open-port/run-*` bundle is on the evidence branch.

## The preflights

- **The earlier dry runs were a *timed* preflight.**
  - *Which:* `bench-open-port … dry` at b6412e1e, and two before it at 70879f15 and fab84e57, in
    the cloud container.
  - *What they generated:* through the recipe's own `ab run` calls, timing records for every cell
    (the held-out corpora and the sustained cell included), the context runs, and the per-block
    census `census-timed`.
  - *What happened to them:* marked DRY, VOID by design, and excluded. Their timings were not
    read; only the census's route counts were (which blocks the gates admit).
  - *What they changed:* no timing changed the port, the corpus roster, the thresholds or the
    decision rule. The changes they caused were mechanical:
    - the harness, since master's `ab` lacks `--unpinned`;
    - where the workers are read;
    - the context reporter skipping failed records.
  - *Kept:* the last bundle, `run-20261119T103840Z` (sha256 1d44ee48…7f1b), is kept, VOID.
- **The preflight for this pin is untimed** (`just bench-open-port … dry`). It doesn't run
  `ab run`, the timed census or the context runs. It does:
  - both builds, the identities and every corpus with its digest;
  - each corpus decoded once by each build, keeping only the output's size and digest
    (`untimed-check.txt`);
  - the scorer's own checks.

## The rule (`bench/open_port.py judge`, frozen with this registration)

**The roster is fixed in the scorer, never read from the records:**
- the 15 corpora of the table, at 8, 10 and 20 threads, 12 pairs each: 1,080 records;
- `wikidata-x16` at 10 threads, 8 pairs: 16 records.

**The registered identities**, from the bundle's own files:
- `commits.txt` names the port's pinned commit and v0.5.6 (530ed404);
- `inputs.sha256` names the pinned Wikidata sample (82f65ace…) and held-out windows (7f5c9a00…),
  and gives a digest for every corpus of the roster.

**Each cell** (corpus, count): the 12 rounds' paired log ratios of wall time, the port over
v0.5.6.
- *Wall:* geometric mean, two-sided 95% t-interval, and a one-sided p for a gain.
- *Peak RSS:* the median of the port's peaks over the median of v0.5.6's.

**VOID**, with no verdict, where:
- a record of the roster is missing, duplicated, malformed or failed;
- a record lies outside the roster, or a file is empty or missing;
- a wall time, peak RSS or output size is not a finite positive number (a NaN included);
- a corpus's outputs differ;
- an identity doesn't hold;
- the machine isn't the registered one;
- either arm observes other than the shipped workers;
- the run is a dry run.

**ADOPTS only if every one of these holds:**

| | condition | the bound |
|---|---|---|
| **P1** (primary, wall) | a gain on `wikidata-x4` at **every** count | Holm at family α 0.05 over the 3 cells |
| **P2** (co-primary, memory) | `wikidata-x4` peak RSS at every count | **≤ 1.00** of v0.5.6's |
| **H** (held-out replication) | `ho-wikidata-l9` at every count | FASTER: the 95% interval wholly below 1 |
| **G** (non-regression, every other corpus, every count: 13 × 3 cells) | wall not SLOWER (the interval not wholly above 1), **and** geometric mean **≤ 1.02**, **and** peak RSS **≤ 1.10** | as stated |
| **S** (sustained) | `wikidata-x16`, 10 threads, back to back | not SLOWER, and geometric mean ≤ 1.02 |

**Otherwise NOT ADOPTED,** with every failing condition named, and 0.5.7 ships without the change.

**What these bounds don't relax:** they are the policy's own, applied to more corpora than any run
before.
- *The 1.02 bound* adds a point-estimate limit to the SLOWER screen, so that a guard regressing
  within noise is caught too.
- *There is no trade.* Rule 4 isn't used here: P2 requires Wikidata memory not to grow at all.

`bench/test_open_port.py` checks the scorer, 39 checks on bundles of the full roster:
- ADOPTS;
- each of P1, P2, H, G (SLOWER, the bound, and memory) and S failing, alone and together;
- **the Researcher's counterexamples, now regressions,** each VOID:
  - only the primary and the held-out replication, with no guard at all (which had scored
    ADOPTS);
  - a NaN peak RSS on the primary (which had scored ADOPTS);
  - an empty sustained file (which had crashed);
- the roster:
  - one corpus missing at one count, or everywhere;
  - a record or corpus outside the roster;
  - a missing sustained round;
  - a NaN or infinite wall, or a zero peak RSS;
  - a record with no digest;
- each identity: another port or reference commit, no `commits.txt`, another sample or held-out
  set, a corpus with no digest;
- the machine, the workers, the outputs, a third arm, two reference binaries, a dry run;
- a guard's short decode seeing fewer workers, without a VOID.

## Context, reported and not judged

- **The gate's cost** (the Coordinator's request): kernel-only arms of W workers, at W = 7, 8, 9 and
  10, on Wikidata ×4 and text, 8 rounds:
  - `mibz2-workers-lazy` (the shipped `OpenCells`);
  - `mibz2-workers-open` (ungated, `ForcedOpenCells`);
  - `mibz2-workers-lazycells` (0.5.6's `LazyCells`).

  Reported: OpenCells over ungated open, and OpenCells over `LazyCells`, at each W. Text is
  expected to show the gate's purpose.
- **The crossover:** eager (`mibz2-workers-eager`) against `OpenCells` at W = 4 to 8, on Wikidata ×4
  and text. It fixes where open would pay below the shipped switch (lazy from 8 workers). Any
  change to that switch is a later registration, not this one.
- **Coverage and cost on every corpus,** development and held out: the per-block census
  (`ab census-timed`, one thread): each block's route under the gates, and the open kernel's cost
  against production's.

## Predictions, before the data (the Pathfinder's; nothing is scored against them)

- **P1:** about 0.94–0.95 on Wikidata at each count. That is opend's kernel-only gain, which
  record 71 put at about 0.95 and bench-opend-sweep at 0.933–0.953. Ungated open measured 0.90–0.92
  in the moonshot runs, and the gate costs part of that.
- **P2:** about 0.85, opend's in bench-open-return.
- **The guards:** level on text, random, records, pattern and onebyte, where the gates send every
  block to `LazyCells` as shipped. The held-out prose and code are expected to be refused like
  text. The structured data is the least certain: some blocks may fall under the line.
- **The gate's cost:** a few percent on Wikidata at W ≥ 7, and large savings on text.
