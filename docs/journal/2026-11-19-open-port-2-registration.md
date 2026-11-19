# The open kernel's port for 0.5.7: the successor registration

*19 November 2026. The Pathfinder's draft, for the Researcher's prospective review through the
Coordinator (#145 QUEUED 7011091171). Corrected after the Researcher's targeted correction (REVIEW
7012181194) under the Coordinator's decisions (QUEUED 7012199074 and the decisions of 14:45: the full
schedule, and S in G2's family). It is frozen before any data.*

**This is a new experiment, not a retry.** The first registration
(`2026-11-19-open-port-registration.md`) had one confirmatory attempt,
`open-port/run-20261119T125538Z`, and it was **NOT ADOPTED**. That verdict stands. This
registration doesn't reinterpret it, re-judge it or reuse its data as evidence. The first
attempt's paired noise is used only to plan this one.

**Why there is a successor.** The first registration's guard rule G had a false-fail rate it
never stated: about 0.96 for a port with no true effect, over 39 guard cells whose 95% intervals
reached about ±9% at 12 rounds. Four guard cells failed G, all on corpora where the gates admit no
block. The successor keeps everything else as registered, and replaces G's wall-time part with
G2, and moves S into G2's family. G2 keeps G's estimand, the geometric mean, and has a false-fail
rate and per-cell power stated here before any data.

- **The change:** unchanged. The port, #630 (`perf/open-kernel`) at
  9743bbe0d51ba451b8b8ab9e6fd731c238940052, against v0.5.6 (530ed404).
- **The recipe:** `just bench-open-port-2`, on `bench/open-port-2`.
- **The scorer:** `bench/open_port2.py`, with its locked schedule `bench/open-port-2-schedule.json`.
  The first registration's `bench/open_port.py` is unchanged, and still re-judges the first attempt.
- **What it decides:** whether 0.5.7 ships `OpenCells` as the workers' kernel wherever 0.5.6 ran
  `LazyCells`. If not, 0.5.7 ships without it.

## What is the same as the first registration

Nothing in this list is dropped, loosened or re-weighted (PERFORMANCE-POLICY §2):
- **the builds, the machine and the inputs:** the same 18 inputs, checked against the same manifest
  `bench/open-port-inputs.sha256` before any timing (`verify`), and again when judged;
- **the roster:** the same 15 corpora at 8, 10 and 20 threads, the same primary, held-out
  replication and sustained cell, and the same 13 guards.
  - *The corpora with no admitted block stay guards.* They guard binary layout and dispatch cost.
- **the decision run, `runs.jsonl`:** the first registration's recipe. One `ab run` a count, 12
  paired rounds, the 2 s settle, unpinned.
- **P1, P2 and H, from those 12 rounds:**
  - *P1:* a wall gain on wikidata-x4 at every count, Holm at α 0.05;
  - *P2:* wikidata-x4's peak RSS at most 1.00 of v0.5.6's;
  - *H:* ho-wikidata-l9 FASTER at every count.
- **every guard's memory, from those 12 rounds:** peak RSS at most 1.10 of v0.5.6's, median over
  median;
- **the sustained cell's conditions:** 10 threads back to back on wikidata-x16, with no memory
  condition, as registered;
- **every VOID condition,** applied to every file;
- **the context and the census,** reported and not judged.

## What changes: G and S's wall-time rule becomes G2

| | the first registration's G and S (wall time) | the successor's G2 |
|---|---|---|
| cells | the 39 guard cells (G), and the sustained cell (S) | the same 40, in one family |
| estimand | the geometric mean of the paired ratios | the same |
| rounds | 12 (S: 8) | each cell's own, 72 to 576 (S: 84), locked |
| a cell fails when | its geometric mean exceeds 1.02, or its 95% t-interval lies wholly above 1 | its mean log ratio exceeds its locked critical value c (x1.0147 to x1.0171) |
| P(any fails), no effect | about 0.96 (G); 0.148 (S); never stated in advance | 0.011 to 0.033 under the planning models; 0.143 under a stress model |

- **S changes.** This is the Coordinator's decision of 19 November, reversing "S exactly as
  registered", and for the Researcher's prospective review.
  - *S as registered:* 8 rounds, the 1.02 point cap and SLOWER. Under its own noise it has a
    false-fail rate of **0.148** and power **0.55** at a true +2% slowdown.
  - *S in G2:* 84 rounds back to back, its own c, power 0.87 at +2%.
  - *Why:* G2 is stricter against a real regression on S, and keeps the whole family's false-fail
    rate within its budget.
  - *What doesn't change:* S's cell, back to back, and its lack of a memory condition.
- **The statistic:** the mean of the cell's paired log ratios (the port's wall over v0.5.6's)
  over all its rounds. For a guard, these are the decision run's 12 and the extension's; for S,
  `sustained.jsonl`'s. Its exponential is the geometric mean, G's estimand. Spikes count in it, as
  they did in G.
- **Why the critical value is fixed, not studentised** (the Coordinator's decision 1 asks for a
  mean-based test that fails the Researcher's counterexample):
  - *The counterexample:* 57 rounds at −0.001 and 3 at +0.5. Its geometric mean is 1.02434.
  - *A studentised test can't fail it.* The three spikes inflate the cell's own variance: a paired
    t-test gives t = 1.69, p = 0.048, far from 0.048/40. A bootstrap-t is the same in kind.
  - *So each cell's critical value c is fixed before the data,* calibrated by simulation. Every c
    is at most 0.0170, so the counterexample's mean of 0.0240 fails in every cell.
- **The calibration** (`2026-11-19-open-port-2-registration/plan.py`, output `plan.txt` and
  `plan.json`; 50,000 draws a cell, a size and a power; seed fixed):
  - *each cell's c* is the largest of the (1 − 0.048/40) quantiles of its mean under no effect, in
    three noise models:
    - the cell's own residuals, centred on its mean (12 rounds; S's 8);
    - its corpus's pooled residuals (S's own);
    - all 468 guard residuals.

    The last carries the first attempt's spike rate: |residual| > 0.1 in 5.8% of rounds, and
    > 0.15 in 2.1%.
  - *each model* resamples rounds independently with replacement.
  - *each cell's rounds* are the smallest on the grid giving, at a true +2% slowdown, power at
    least **max(0.80, G − 0.005)** under the cell's own residuals. G is the first registration's
    rule's power in that cell under the same residuals: 12 rounds, or S's 8.
    - *Why:* parity, the Coordinator's decision of 19 November. G2 is at least as strict as G at
      catching a real 2% slowdown, cell by cell, and never below 0.80.
    - *Why 0.005:* where G's power is about 1 (1.000 in four cells), exact parity would chase
      Monte Carlo error. With 50,000 draws, a power near 0.8 has a standard error of about 0.002,
      and one near 1 much less. So the target is G less 0.005.
  - *the grid:* 12, 24, 36, 48, 60, 72, 84, 96, 120, 144, 168, 192, 216, 240, 288, 336, 384, 432,
    480, 576, 672, 768, 864 and 1,000 rounds.
  - *the guards' extension* is one `ab run` a batch: 19 batches (`open_port2.py schedule`), each a
    count and a round count with its corpora, into `guards-t{threads}-r{rounds}.jsonl`.
  - *S's rounds* are one back-to-back run, `sustained.jsonl` (`open_port2.py schedule
    --sustained`).
  - *The schedule* is locked in `bench/open-port-2-schedule.json`.

## Its error rates, stated before the data

**The false-fail rate** (no true effect on any cell), P(at least one of the 40 cells fails G2):

| noise model | P(any fails) |
|---|--:|
| each cell's own residuals | 0.012 |
| each corpus's pooled residuals | 0.011 |
| all the guards' residuals (the observed spike rate) | **0.033** |
| stress: 2% of every cell's rounds a spike of ±0.2 to 0.4 | 0.143 |

- **The budget:** each cell's c holds its size to at most 0.048/40 = 0.0012 under each of the three
  models, so the family's is at most 0.048 under any one of them.
  - *The memory guard's,* by resampling the first attempt's peak RSS with both arms drawn from one
    arm's spread, is about 0 (≤ 0.0015).
  - *Together, at most 0.05 under the stated calibration.*
- **These are model-based, not exact.**
  - *The models assume* the rounds are independent within a cell.
    - The pairing takes out what both arms share in a round.
    - The arm order follows the harness's fixed AB/BA cycle, which balances position but doesn't
      randomise it.
    - Serial dependence and carryover are not modelled.
  - *The family* multiplies the cells' pass rates, assuming independence across cells.
  - *The stress row* is a spike rate about the first attempt's 95% upper bound for spikes above
    0.15 (3.9%), at larger magnitudes. It shows how size degrades if the i9's next run is spikier:
    0.033 at the observed rate, 0.143 under this stress.

**The power,** per cell, beside the first registration's rule under the same noise. G2 has each
cell's own rounds; G has 12, and S's registered rule 8.

| cell | threads | rounds | c | target at +2% | G2 +1% | G2 +2% | G2 +3% | G +1% | G +2% | G +3% |
|---|--:|--:|---|--:|--:|--:|--:|--:|--:|--:|
| ho-code-l1 | 8 | 288 | 0.0158 (x1.0159) | 0.800 | 0.10 | **0.811** | 1.00 | 0.31 | 0.485 | 0.65 |
| ho-code-l1 | 10 | 336 | 0.0149 (x1.0150) | 0.800 | 0.15 | **0.857** | 1.00 | 0.31 | 0.460 | 0.62 |
| ho-code-l1 | 20 | 336 | 0.0149 (x1.0150) | 0.800 | 0.17 | **0.836** | 1.00 | 0.39 | 0.528 | 0.67 |
| ho-code-l9 | 8 | 168 | 0.0167 (x1.0168) | 0.800 | 0.00 | **0.922** | 1.00 | 0.21 | 0.594 | 0.91 |
| ho-code-l9 | 10 | 576 | 0.0146 (x1.0147) | 0.800 | 0.18 | **0.849** | 1.00 | 0.43 | 0.544 | 0.65 |
| ho-code-l9 | 20 | 192 | 0.0158 (x1.0159) | 0.800 | 0.07 | **0.854** | 1.00 | 0.26 | 0.490 | 0.73 |
| ho-prose-l1 | 8 | 144 | 0.0150 (x1.0151) | 0.800 | 0.10 | **0.894** | 1.00 | 0.24 | 0.506 | 0.77 |
| ho-prose-l1 | 10 | 216 | 0.0152 (x1.0153) | 0.800 | 0.15 | **0.827** | 1.00 | 0.31 | 0.481 | 0.67 |
| ho-prose-l1 | 20 | 144 | 0.0149 (x1.0150) | 0.800 | 0.13 | **0.874** | 1.00 | 0.26 | 0.506 | 0.75 |
| ho-prose-l9 | 8 | 96 | 0.0152 (x1.0153) | 0.800 | 0.12 | **0.857** | 1.00 | 0.31 | 0.554 | 0.79 |
| ho-prose-l9 | 10 | 84 | 0.0159 (x1.0160) | 0.995 | 0.00 | **0.997** | 1.00 | 0.66 | 1.000 | 1.00 |
| ho-prose-l9 | 20 | 84 | 0.0163 (x1.0164) | 0.916 | 0.00 | **0.970** | 1.00 | 0.47 | 0.921 | 1.00 |
| ho-structured-l1 | 8 | 84 | 0.0157 (x1.0158) | 0.800 | 0.08 | **0.855** | 1.00 | 0.19 | 0.546 | 0.87 |
| ho-structured-l1 | 10 | 96 | 0.0155 (x1.0156) | 0.800 | 0.13 | **0.813** | 1.00 | 0.23 | 0.473 | 0.79 |
| ho-structured-l1 | 20 | 84 | 0.0159 (x1.0160) | 0.800 | 0.05 | **0.855** | 1.00 | 0.35 | 0.542 | 0.83 |
| ho-structured-l9 | 8 | 84 | 0.0161 (x1.0162) | 0.800 | 0.09 | **0.801** | 1.00 | 0.21 | 0.505 | 0.79 |
| ho-structured-l9 | 10 | 84 | 0.0161 (x1.0162) | 0.800 | 0.09 | **0.806** | 1.00 | 0.22 | 0.538 | 0.80 |
| ho-structured-l9 | 20 | 96 | 0.0148 (x1.0149) | 0.994 | 0.00 | **0.999** | 1.00 | 0.51 | 0.999 | 1.00 |
| ho-wikidata-l1 | 8 | 84 | 0.0160 (x1.0161) | 0.983 | 0.00 | **0.999** | 1.00 | 0.74 | 0.988 | 1.00 |
| ho-wikidata-l1 | 10 | 96 | 0.0149 (x1.0150) | 0.995 | 0.01 | **0.998** | 1.00 | 0.43 | 1.000 | 1.00 |
| ho-wikidata-l1 | 20 | 72 | 0.0170 (x1.0171) | 0.800 | 0.01 | **0.801** | 1.00 | 0.32 | 0.611 | 0.88 |
| onebyte | 8 | 84 | 0.0159 (x1.0160) | 0.800 | 0.05 | **0.848** | 1.00 | 0.33 | 0.551 | 0.84 |
| onebyte | 10 | 84 | 0.0157 (x1.0158) | 0.800 | 0.06 | **0.862** | 1.00 | 0.35 | 0.546 | 0.83 |
| onebyte | 20 | 72 | 0.0169 (x1.0170) | 0.800 | 0.01 | **0.862** | 1.00 | 0.28 | 0.793 | 0.99 |
| pattern | 8 | 84 | 0.0162 (x1.0163) | 0.800 | 0.08 | **0.807** | 1.00 | 0.19 | 0.620 | 0.98 |
| pattern | 10 | 84 | 0.0163 (x1.0164) | 0.800 | 0.02 | **0.873** | 1.00 | 0.17 | 0.595 | 0.95 |
| pattern | 20 | 84 | 0.0160 (x1.0161) | 0.800 | 0.05 | **0.852** | 1.00 | 0.16 | 0.538 | 0.95 |
| random | 8 | 96 | 0.0153 (x1.0154) | 0.995 | 0.00 | **0.999** | 1.00 | 0.53 | 1.000 | 1.00 |
| random | 10 | 84 | 0.0162 (x1.0163) | 0.800 | 0.07 | **0.806** | 1.00 | 0.19 | 0.484 | 0.84 |
| random | 20 | 84 | 0.0161 (x1.0162) | 0.995 | 0.00 | **0.999** | 1.00 | 0.80 | 1.000 | 1.00 |
| records | 8 | 84 | 0.0160 (x1.0161) | 0.972 | 0.00 | **0.984** | 1.00 | 0.45 | 0.977 | 1.00 |
| records | 10 | 84 | 0.0161 (x1.0162) | 0.800 | 0.02 | **0.879** | 1.00 | 0.35 | 0.593 | 0.88 |
| records | 20 | 84 | 0.0164 (x1.0165) | 0.960 | 0.00 | **0.985** | 1.00 | 0.59 | 0.965 | 1.00 |
| runs | 8 | 216 | 0.0152 (x1.0153) | 0.800 | 0.15 | **0.831** | 1.00 | 0.31 | 0.492 | 0.69 |
| runs | 10 | 168 | 0.0165 (x1.0166) | 0.800 | 0.03 | **0.821** | 1.00 | 0.35 | 0.538 | 0.75 |
| runs | 20 | 288 | 0.0153 (x1.0154) | 0.800 | 0.16 | **0.813** | 1.00 | 0.37 | 0.519 | 0.66 |
| text | 8 | 96 | 0.0149 (x1.0150) | 0.800 | 0.15 | **0.860** | 1.00 | 0.26 | 0.525 | 0.78 |
| text | 10 | 96 | 0.0153 (x1.0154) | 0.800 | 0.13 | **0.838** | 1.00 | 0.22 | 0.535 | 0.86 |
| text | 20 | 84 | 0.0160 (x1.0161) | 0.869 | 0.01 | **0.948** | 1.00 | 0.26 | 0.874 | 1.00 |
| wikidata-x16 (S) | 10 | 84 | 0.0162 (x1.0163) | 0.800 | 0.02 | **0.865** | 1.00 | 0.31 | 0.548 | 0.82 |

- **At +2%, every cell meets its target:** G2 is 0.801 to 0.999, against the first registration's
  0.46 to 1.00.
  - *The tightest:* ho-structured-l9 at 8 threads, 0.801 against 0.800, and pattern at 8 threads,
    0.807. These are within the Monte Carlo error stated above.
  - *Where G's power is about 1* (ho-wikidata-l1, random, records, ho-prose-l9, ho-structured-l9 20t,
    text 20t), G2 reaches 0.948 to 0.999 with 84 or 96 rounds.
- **At +3%,** G2 is 0.997 to 1.00, against 0.62 to 1.00.
- **At +1%, G2 is weaker than the first registration's rules on every cell.**
  - *How much:* G2 is 0.00 to 0.18 per cell, against their 0.16 to 0.80.
  - *Why:* the 1.02 point bound fails noisy cells often whatever the effect (its false-fail rate);
    G2 doesn't.
  - *What still holds for smaller losses:* the policy's protections stand, unchanged
    (PERFORMANCE-POLICY §2).
- **Parity is claimed at +2%, cell by cell, within the 0.005 tolerance.** At +3% G2 is the more
  powerful everywhere. No claim is made at other effects beyond the table.
- **The power is model-based.**
  - *It assumes* the next run's noise is like the first attempt's, cell by cell, from only 12
    rounds a cell (S's 8).
  - *Where noise is spikier,* power is higher and size is worse. Where noise is calmer, power is
    lower at the same c.
  - *Nothing is adjusted after the data:* no rounds are added, and no cell is re-tested.

**The time,** the parity schedule (the Coordinator's decision: no cell below max(0.80, G − 0.005)
at +2%), about **6.75 hours** on the i9:
- the decision run, about 38 minutes;
- S, 84 rounds back to back, about 3 minutes;
- the guards' extension, 4,932 pairs (9,864 decodes, each after its 2 s settle), about 5.75 hours;
- the context and the census, about 15 minutes.

The Coordinator's 2.5 hours isn't reachable with every cell at 0.80. A capped schedule would leave
cells weaker than G at +2%; this successor exists to fix that, so none is used.

## One attempt, its own

- **This registration allows one timed attempt on the i9.** Its bundle or failure record is
  kept, whatever happens.
  - A VOID gives no adoption evidence.
  - An inconclusive required gain (P1 or H) is NOT ADOPTED.
  - Neither opens a retry.
- **Its evidence path is its own:** `target/open-port-2/`, archived as `open-port-2/run-*.tar.gz`.
  The first registration's `open-port/` bundle can't be confused with it.
- **The guard** (`open_port2.py guard`) refuses a second attempt:
  - one started in this checkout (`timing-started.txt`, written just before the first `ab run`);
  - one archived as `open-port-2/run-*`;
  - **failing closed** when the evidence branch can't be fetched afresh or read.

  The first registration's archived bundle doesn't block it.
- **Untimed setup may be repaired.** A run that stops before `timing-started.txt` is no attempt:
  a build failing, or the inputs not matching the manifest.

## The run

- **The order:**
  1. the builds;
  2. the inputs, verified against the manifest;
  3. `timing-started.txt`;
  4. the decision run;
  5. S, its 84 rounds back to back;
  6. the guards' extension, batch by batch;
  7. the verdict;
  8. the context and the census, as before.
- **The untimed preflight** (`just bench-open-port-2 … dry`), in the cloud container at 7dbcf5f4
  (S in G2, the full schedule):
  - "the builds and all 18 inputs are the registered ones";
  - 16 of 16 outputs the same in both builds;
  - 30 of 30 scorer checks;
  - no timing field, and no `runq` probe.

  Its bundle is `2026-11-19-open-port-2-registration/preflight/run-20261119T145510Z.tar.gz`, sha256
  `c1f0ab76a588a7aa015c11bb49bd85b2b8f7cfd448406b1c8829b75bdd20792e`. The earlier drafts' preflights, at 9b3ed323 and fd896bf7, are superseded; they
  stay in this branch's history.

## The rule (`bench/open_port2.py judge`, frozen with this registration)

**VOID, with no verdict,** where any of the first registration's VOID conditions holds of
`runs.jsonl`, `sustained.jsonl` or any batch file, each against its own roster:
- a record missing, duplicated, failed, malformed, or outside the roster;
- a field of the wrong type or value;
- a non-finite or non-positive measurement;
- outputs that differ within a file;
- an identity that doesn't hold;
- another machine;
- other workers on the Wikidata corpora;
- a dry run.

**Also VOID, across files** (the Researcher's correction, decision 4):
- a corpus's output size and digest differ between any two files;
- either build's arm (its exact binary path) differs between any two files;
- a batch file is missing, or a `guards-*` file lies outside the schedule.

**NOT ADOPTED,** every failing condition named, where any of these fails:
- P1, P2 or H, as registered;
- a guard's memory;
- G2 on any of the 40 cells, S among them.

**ADOPTS** otherwise.

`bench/test_open_port2.py` checks the scorer and the guard, 30 checks:
- ADOPTS with guards of no effect;
- G2 failing a guard truly 2% slower (ho-code-l9 10t, text 8t, records 20t) or 3% slower;
- G2 passing guards of no effect with symmetric spikes of ±0.29;
- **the Researcher's counterexample:**
  - as a cell's extension, G2 fails it;
  - directly, its mean exceeds every cell's c;
- G2 failing S truly 2% slower; S with a round past its 84, or at the first registration's 8, giving VOID;
- P1, P2, H and the memory guard, as registered;
- **the cross-file joins:**
  - text's output digest different in the extension: VOID;
  - the port's binary another path in one batch: VOID;
- a missing or stray batch file, threads 8.5, a NaN wall, a short batch, another port, an input off
  the manifest, a dry run: each VOID;
- the schedule's own consistency;
- the successor's one-attempt guard on scratch repositories.

Disabling G2's comparison on the guards fails five of those checks, on S one, and disabling the joins two.

## Context, reported and not judged

As in the first registration:
- the gate's cost, `gate.txt`;
- the eager/open crossover, `crossover.txt`;
- the per-block census, `census.jsonl`.

`verdict.txt` also reports each guard cell's geometric mean and interval over all its rounds.
