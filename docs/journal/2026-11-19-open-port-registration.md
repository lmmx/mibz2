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

## The result: NOT ADOPTED (the one confirmatory attempt, 19 November)

*Recorded after the data. It is a factual record and reinterprets nothing: the attempt and its
outcome stand, and this registration opens no retry. Everything from the next section on is the
registration as frozen at 4bb9e319, before the data.*

- **The bundle:** `open-port/run-20261119T125538Z.tar.gz` on the `evidence` branch, at
  5c028f5a8ec90effa9fdf04540c85f473c2e5375. Its SHA-256 is
  `fb4771f5252a0d03c97b85d8b3103c7db605134075b86d2b7a45f22aef36a523`, 180,102 bytes (#145 LOG
  7011043887).
- **The run:**
  - *started:* `just submit bench-open-port`, 2026-11-19T12:55:37Z (#145 LOG 7009734722);
  - *machine:* the i9-10900K;
  - *recipe:* at 4bb9e31963b20ab2c359bb89afaf81b8b537ba35, the frozen head;
  - *builds:* the port at 9743bbe0d51ba451b8b8ab9e6fd731c238940052, v0.5.6 at
    530ed404a014a414ef0b62adfd947fa14d4533e6;
  - *timing began* at 13:36:08Z (`timing-started.txt`).
- **The inputs:** `inputs-check.txt` reads "the builds and all 18 inputs are the registered ones".
- **Re-judged:** the Coordinator re-ran `open_port.py judge --port 9743bbe0…` at the frozen head,
  and its output is identical to the bundle's `verdict.txt` (#145 LOG 7011074912).
- **Consequences, as registered:**
  - #630 does not merge;
  - 0.5.7 does not ship `OpenCells`;
  - no retry follows under this registration.

  A successor is a new experiment under its own prospective review.

**The verdict, verbatim (`verdict.txt`):**

```
records 1080 of 1080 and 16 of 16 sustained; the registered roster: 15 corpora at [8, 10, 20] threads, 12 rounds (8 sustained); the machine: Intel(R) Core(TM) i9-10900K CPU @ 3.70GHz

the port over v0.5.6: wall, the rounds' paired ratios, geometric mean [95% interval]; one-sided p (gain); peak RSS, median over median
  8 threads
    wikidata-x4 (primary): 0.9465 [0.9344, 0.9587] FASTER; p 6.77e-07; memory 0.779
    text (guard): 0.9906 [0.9613, 1.0209] INCONCLUSIVE; p 2.53e-01; memory 0.990
    random (guard): 0.9997 [0.9899, 1.0095] INCONCLUSIVE; p 4.70e-01; memory 1.000
    records (guard): 0.9981 [0.9871, 1.0091] INCONCLUSIVE; p 3.52e-01; memory 0.998
    runs (guard): 0.9991 [0.9527, 1.0478] INCONCLUSIVE; p 4.84e-01; memory 1.000
    pattern (guard): 1.0067 [0.9815, 1.0325] INCONCLUSIVE; p 7.13e-01; memory 0.988
    onebyte (guard): 0.9906 [0.9680, 1.0137] INCONCLUSIVE; p 1.94e-01; memory 1.000
    ho-wikidata-l9 (held-out replication): 0.9419 [0.9237, 0.9605] FASTER; p 1.61e-05; memory 0.789
    ho-wikidata-l1 (guard): 0.9971 [0.9899, 1.0043] INCONCLUSIVE; p 1.97e-01; memory 0.783
    ho-prose-l1 (guard): 0.9808 [0.9509, 1.0117] INCONCLUSIVE; p 9.80e-02; memory 1.003
    ho-prose-l9 (guard): 1.0046 [0.9764, 1.0337] INCONCLUSIVE; p 6.36e-01; memory 1.001
    ho-code-l1 (guard): 1.0220 [0.9705, 1.0761] INCONCLUSIVE; p 8.13e-01; memory 0.996
    ho-code-l9 (guard): 1.0013 [0.9825, 1.0206] INCONCLUSIVE; p 5.61e-01; memory 1.006
    ho-structured-l1 (guard): 1.0153 [0.9914, 1.0398] INCONCLUSIVE; p 9.06e-01; memory 0.894
    ho-structured-l9 (guard): 0.9825 [0.9561, 1.0097] INCONCLUSIVE; p 9.11e-02; memory 0.979
  10 threads
    wikidata-x4 (primary): 0.9372 [0.9249, 0.9496] FASTER; p 1.59e-07; memory 0.761
    text (guard): 1.0049 [0.9752, 1.0355] INCONCLUSIVE; p 6.38e-01; memory 1.008
    random (guard): 1.0217 [0.9959, 1.0482] INCONCLUSIVE; p 9.54e-01; memory 1.000
    records (guard): 0.9915 [0.9729, 1.0105] INCONCLUSIVE; p 1.72e-01; memory 0.981
    runs (guard): 0.9904 [0.9596, 1.0221] INCONCLUSIVE; p 2.57e-01; memory 1.000
    pattern (guard): 1.0120 [0.9929, 1.0315] INCONCLUSIVE; p 9.02e-01; memory 1.001
    onebyte (guard): 0.9878 [0.9654, 1.0107] INCONCLUSIVE; p 1.32e-01; memory 1.001
    ho-wikidata-l9 (held-out replication): 0.9148 [0.8818, 0.9491] FASTER; p 1.21e-04; memory 0.799
    ho-wikidata-l1 (guard): 1.0035 [0.9910, 1.0161] INCONCLUSIVE; p 7.24e-01; memory 0.798
    ho-prose-l1 (guard): 1.0150 [0.9676, 1.0648] INCONCLUSIVE; p 7.47e-01; memory 0.991
    ho-prose-l9 (guard): 0.9999 [0.9908, 1.0092] INCONCLUSIVE; p 4.95e-01; memory 1.002
    ho-code-l1 (guard): 1.0310 [0.9742, 1.0912] INCONCLUSIVE; p 8.70e-01; memory 1.019
    ho-code-l9 (guard): 0.9754 [0.9004, 1.0567] INCONCLUSIVE; p 2.54e-01; memory 1.018
    ho-structured-l1 (guard): 1.0192 [0.9876, 1.0517] INCONCLUSIVE; p 8.95e-01; memory 0.903
    ho-structured-l9 (guard): 0.9756 [0.9494, 1.0025] INCONCLUSIVE; p 3.56e-02; memory 0.927
  20 threads
    wikidata-x4 (primary): 0.9373 [0.9272, 0.9474] FASTER; p 2.14e-08; memory 0.779
    text (guard): 1.0095 [0.9946, 1.0247] INCONCLUSIVE; p 9.06e-01; memory 1.007
    random (guard): 1.0033 [0.9964, 1.0103] INCONCLUSIVE; p 8.45e-01; memory 0.999
    records (guard): 0.9930 [0.9839, 1.0023] INCONCLUSIVE; p 6.31e-02; memory 0.976
    runs (guard): 0.9624 [0.9079, 1.0201] INCONCLUSIVE; p 8.78e-02; memory 1.003
    pattern (guard): 1.0113 [0.9888, 1.0342] INCONCLUSIVE; p 8.52e-01; memory 1.002
    onebyte (guard): 0.9974 [0.9822, 1.0128] INCONCLUSIVE; p 3.57e-01; memory 1.000
    ho-wikidata-l9 (held-out replication): 0.9276 [0.9161, 0.9393] FASTER; p 2.15e-08; memory 0.816
    ho-wikidata-l1 (guard): 0.9879 [0.9690, 1.0073] INCONCLUSIVE; p 9.80e-02; memory 0.801
    ho-prose-l1 (guard): 0.9678 [0.9353, 1.0014] INCONCLUSIVE; p 2.93e-02; memory 0.997
    ho-prose-l9 (guard): 0.9972 [0.9859, 1.0085] INCONCLUSIVE; p 2.96e-01; memory 1.026
    ho-code-l1 (guard): 0.9588 [0.9014, 1.0198] INCONCLUSIVE; p 8.05e-02; memory 1.005
    ho-code-l9 (guard): 1.0289 [0.9933, 1.0658] INCONCLUSIVE; p 9.49e-01; memory 1.055
    ho-structured-l1 (guard): 0.9870 [0.9647, 1.0098] INCONCLUSIVE; p 1.16e-01; memory 0.898
    ho-structured-l9 (guard): 0.9773 [0.9676, 0.9871] FASTER; p 1.84e-04; memory 0.946

P1, wall on wikidata-x4: Holm at α 0.05 over 3 cells: 8 threads p 6.77e-07 rejects, 10 threads p 1.59e-07 rejects, 20 threads p 2.14e-08 rejects

S, sustained: 10 threads back to back on wikidata-x16: 0.9407 [0.9155, 0.9666] FASTER; memory 0.747

VERDICT: NOT ADOPTED. 4 conditions fail:
  G: ho-code-l1 at 8 threads 1.0220, above the bound 1.02
  G: random at 10 threads 1.0217, above the bound 1.02
  G: ho-code-l1 at 10 threads 1.0310, above the bound 1.02
  G: ho-code-l9 at 20 threads 1.0289, above the bound 1.02
```

**The routing census** (`census.jsonl`, `ab census-timed`, one thread): blocks the shipped
`LazyCells` gates send to the open kernel, of each corpus's blocks, and why the rest are refused.
Only the route counts are recorded here; the census's timings are context, and are not read.

| corpus | admitted to open | refused: density | refused: in-place | refused: sparse |
|---|--:|--:|--:|--:|
| wikidata-x4 | 912 / 1,724 | 620 | 192 | 0 |
| text | 0 / 49 | 49 | 0 | 0 |
| random | 0 / 36 | 0 | 36 | 0 |
| records | 0 / 43 | 0 | 43 | 0 |
| runs | 0 / 2 | 2 | 0 | 0 |
| pattern | 0 / 88 | 0 | 0 | 88 |
| onebyte | 0 / 2 | 0 | 0 | 2 |
| ho-wikidata-l9 | 262 / 473 | 163 | 48 | 0 |
| ho-wikidata-l1 | 2,277 / 2,359 | 82 | 0 | 0 |
| ho-prose-l1 | 4 / 276 | 64 | 208 | 0 |
| ho-prose-l9 | 0 / 32 | 16 | 16 | 0 |
| ho-code-l1 | 0 / 100 | 44 | 56 | 0 |
| ho-code-l9 | 0 / 12 | 12 | 0 | 0 |
| ho-structured-l1 | 44 / 352 | 116 | 192 | 0 |
| ho-structured-l9 | 4 / 40 | 20 | 16 | 0 |

**The four failing cells** are on corpora where the gates admit no block: ho-code-l1 (0/100),
random (0/36) and ho-code-l9 (0/12). There, the port decodes every block on v0.5.6's route.

**The context is in the bundle, not summarised here:**
- the gate's cost, `gate.txt` and `gate.jsonl`;
- the eager/open crossover, `crossover.txt` and `crossover.jsonl`.

**The registration's own flaw, stated by the Coordinator (#145 LOG 7011104819):**
- *The rule:* G compared each guard's point estimate with 1.02, over 39 guard cells whose 95%
  intervals reached about ±9%.
- *Its false-fail rate was never stated.* For a port with no true effect, P(at least one guard
  fails) ≈ 0.97, with ≈ 3.2 failing cells expected; this run had 4.
- *No prospective review caught it.*

The successor registration must state its guard rule's false-fail rate and power before any data.

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
| **held out** | `ho-prose-l1`, `-l9`: the journal's prose at v0.5.6, every `docs/**/*.md` and `*.txt` (7.0 MB) | guards |
| **held out** | `ho-code-l1`, `-l9`: other code at v0.5.6, every `*.py`, `*.just`, `*.yml` and `docs/**/*.asm` (2.6 MB) | guards |
| **held out** | `ho-structured-l1`, `-l9`: structured data that isn't Wikidata, at v0.5.6: every `docs/**/*.json`, `*.jsonl`, `*.csv` and `bench/*.json` (8.8 MB) | guards |

- *Determinism:* `bench/heldout.py` builds the held-out prose, code and structured sets from
  one pinned source tree, the v0.5.6 tag's commit (530ed404), which no later commit can change.
  - *Why not HEAD:* building from the registration's HEAD let a docs commit change every prose
    guard. Archiving the preflight added 2,736 decoded bytes (the Researcher's third correction,
    REVIEW 7008312938).
  - The small ones run as 4 copies, as Wikidata ×4 does.
- *The manifest:* every input's exact compressed SHA-256 is frozen in
  `bench/open-port-inputs.sha256`. Routing depends on the compressed block layout, so the
  compressed bytes are what is fixed.
  - The recipe checks the bundle's inputs against it before any timing (`open_port.py verify`).
    A mismatch stops the run in its untimed setup, so nothing is timed.
  - The scorer checks them again when it judges; a mismatch there is VOID.
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
- *The recipe enforces it* (`open_port.py guard`). A non-dry run refuses to start:
  - if a timed attempt already started in this checkout. The attempt starts at
    `timing-started.txt`, written just before the first `ab run`. A run that stopped in its
    untimed setup is no attempt: a build failing, or the inputs not matching the manifest;
  - if an `open-port/run-*` bundle is on the evidence branch;
  - **failing closed** (the Researcher's second correction, REVIEW 7008014645): if the evidence
    branch can't be fetched afresh or read. A stale local copy of the branch doesn't stand in.

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

  *What "untimed" covers:* no timing is kept or read.
  - The builds' `ab child` measures its own decode internally and prints the timing fields. The
    recipe drops them, keeping only the output's size and digest.
  - `ab env`'s environment probe spins threads and reports `runq.off_cpu_s` and `runq.wait_s`. A
    DRY run drops the probe's `runq` fields from `env.json` (the Researcher's third correction).
- **The preflight bundles are archived beside this note,** in
  `2026-11-19-open-port-registration/preflight/`, each with its `.sha256`. Both ran in the cloud
  container, so `DRY-RUN.txt` marks them and the scorer would void them; they check the recipe,
  not the machine.

  | bundle | at | SHA-256 | outputs | scorer checks |
  |---|---|---|---|---|
  | `run-20261119T111634Z` | 4e86ffc8 | 98ba331d4e49b8f09fbf8e05b8b5b9cde97193471860c3c9c702286da9215957 | 16 of 16 the same | 39 of 39 |
  | `run-20261119T114444Z` | 5308788c | 60e11db7e2b9fbca3a56e0833d1e18eea1ab3e4caf397cce38741394f0038332 | 16 of 16 the same | 63 of 63 |
  | **`run-20261119T115951Z`** | a1dc5aeb | 70ee82a5e3581a896bfe3661402856f6ececb70f33010919123fcc6f455d4f33 | 16 of 16 the same | 73 of 73 |

  The second is the preflight for the second correction. Its `inputs.sha256` passes that
  correction's identity checks, and no file holds a decoder timing field. Its `env.json` still
  holds the environment probe's `runq` timings, which DRY runs now drop.
  - *Why both are superseded:* both built the held-out prose, code and structured sets from the
    branch's HEAD, so their corpora differ from each other and from the manifest's.
  - *The third, `run-20261119T115951Z`, is the preflight for the pinned corpora:*
    - its held-out sets are built from v0.5.6's tree (`heldout.json` names 530ed404);
    - all 18 inputs match the manifest exactly (`inputs-check.txt`);
    - `env.json` holds no `runq` probe, and no file holds a timing field.
  - *Not on the evidence branch:* the one-attempt guard refuses any `open-port/run-*` bundle
    archived there, so a preflight there would block the attempt.
  - *The earlier timed preflight's bundle is not archived,* since its timing records stay
    unread.

## The rule (`bench/open_port.py judge`, frozen with this registration)

**The roster is fixed in the scorer, never read from the records:**
- the 15 corpora of the table, at 8, 10 and 20 threads, 12 pairs each: 1,080 records;
- `wikidata-x16` at 10 threads, 8 pairs: 16 records.

**The registered identities**, from the bundle's own files:
- `commits.txt` names the port's pinned commit and v0.5.6 (530ed404).
- `inputs.sha256` gives every input exactly once, as a 64-hex-digit SHA-256: the 16 corpora of
  the roster and the two bases. It names no other input.
- **Every input is frozen** at its exact compressed SHA-256, in the manifest
  `bench/open-port-inputs.sha256`. That covers the bases, the judged copies (the Researcher's
  second correction) and the 11 generated guards (its third).
  - *Two pass through a compressor:* `ab gen`, with the crate's vendored libbzip2, and Python's
    libbzip2 for the held-out sets.
  - *If the workstation's compressor differs,* the inputs fail `verify` before any timing. That
    is a repairable setup failure, not a VOID.

**Every record's fields are checked before use.** The corpus, thread count and round must be
exactly one of the registered values of their type: threads 8.5 is not 8, and "8" is not 8. The
arm must be a string, and the output digest a nonempty string (an empty `hash` with no `sha256`
can't make two outputs agree). Any other record is VOID, never coerced into the roster and never
an exception.

**Each cell** (corpus, count): the 12 rounds' paired log ratios of wall time, the port over
v0.5.6.
- *Wall:* geometric mean, two-sided 95% t-interval, and a one-sided p for a gain.
- *Peak RSS:* the median of the port's peaks over the median of v0.5.6's.

**VOID**, with no verdict, where:
- a record of the roster is missing, duplicated, malformed or failed;
- a record lies outside the roster, a field has another type or value than the registered ones,
  or a file is empty or missing;
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

`bench/test_open_port.py` checks the scorer, `verify` and the guard, 73 checks, on bundles of
the full roster and on scratch repositories:
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
- a guard's short decode seeing fewer workers, without a VOID;
- **the Researcher's second counterexamples (REVIEW 7008014645), now regressions:**
  - `not-a-digest` for `text.bz2` gives VOID (it had scored ADOPTS);
  - the judged `ho-wikidata-l9.bz2` with another digest, its base frozen, gives VOID (it had
    scored ADOPTS);
  - the first `text` pair at threads 8.5 gives VOID (it had scored ADOPTS);
  - a numeric arm gives VOID (it had raised `AttributeError`);
  - with no local marker and an unreadable evidence branch, the guard refuses (it had passed).
- *their kin:*
  - threads "8" or `true`, round 0.0, a numeric corpus, a numeric output digest, a record that
    isn't an object;
  - another digest for the primary's or the sustained copy, a digest in capitals or too short,
    an input named twice or outside the registration, a receipt line of three fields;
  - a receipt of well-formed digests, giving ADOPTS;
- *the guard:*
  - it passes a readable evidence branch with only a dry run present, or a run that stopped in
    its untimed setup;
  - it refuses a timed attempt started here, an attempt archived on the branch, a remote with no
    evidence branch, and an unreachable remote whose stale clean copy is present;
- **the Researcher's third counterexamples (REVIEW 7008312938), now regressions:**
  - an empty effective digest (`hash: ""`, no `sha256`) gives VOID, as a blank `sha256` does;
  - a generated guard off the manifest gives VOID, for a held-out set, an `ab gen` corpus and a
    level-1 set;
- *the manifest and `verify`:*
  - the manifest names every input once, the held-out copy as its base, and the held-out sources
    as v0.5.6's tree;
  - `verify` passes the registered builds and inputs, and refuses an input off the manifest or
    another port.

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
