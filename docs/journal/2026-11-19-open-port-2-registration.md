# The open kernel's port for 0.5.7: the successor registration

*19 November 2026. The Pathfinder's draft, for the Researcher's prospective review through the
Coordinator (#145 QUEUED 7011091171). Corrected after the Researcher's targeted correction (REVIEW
7012181194) under the Coordinator's decisions (QUEUED 7012199074). It is frozen before any data.*

**This is a new experiment, not a retry.** The first registration
(`2026-11-19-open-port-registration.md`) had one confirmatory attempt,
`open-port/run-20261119T125538Z`, and it was **NOT ADOPTED**. That verdict stands. This
registration doesn't reinterpret it, re-judge it or reuse its data as evidence. The first
attempt's paired noise is used only to plan this one.

**Why there is a successor.** The first registration's guard rule G had a false-fail rate it
never stated: about 0.96 for a port with no true effect, over 39 guard cells whose 95% intervals
reached about ±9% at 12 rounds. Four guard cells failed G, all on corpora where the gates admit no
block. The successor keeps everything else as registered, and replaces G's wall-time part with
G2. G2 keeps G's estimand, the geometric mean, and has a false-fail rate and per-cell power stated
here before any data.

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
- **S, exactly as registered** (the Coordinator's decision 3): `sustained.jsonl`, 10 threads back to
  back on wikidata-x16, 8 rounds, failing if SLOWER or if its geometric mean exceeds 1.02;
- **every VOID condition,** applied to every file;
- **the context and the census,** reported and not judged.

## What is new: the guards' wall-time rule, G2

| | the first registration's G (wall time) | the successor's G2 |
|---|---|---|
| cells | the 39 guard cells (S separately) | the same 39 (S as registered) |
| estimand | the geometric mean of the paired ratios | the same |
| rounds | 12 | each cell's own, 72 to 600, locked |
| a cell fails when | its geometric mean exceeds 1.02, or its 95% t-interval lies wholly above 1 | its mean log ratio exceeds its locked critical value c (x1.0144 to x1.0177) |
| P(any of 39 fails), no effect | about 0.96 (not stated in advance) | 0.013 to 0.034 under the planning models; 0.15 under a stress model |

- **The statistic:** the mean of the cell's paired log ratios (the port's wall over v0.5.6's)
  over all its rounds: the decision run's 12, and the extension's. Its exponential is the
  geometric mean, G's estimand. Spikes count in it, as they did in G.
- **Why the critical value is fixed, not studentised** (the Coordinator's decision 1 asks for a
  mean-based test that fails the Researcher's counterexample):
  - *The counterexample:* 57 rounds at −0.001 and 3 at +0.5. Its geometric mean is 1.02434.
  - *A studentised test can't fail it.* The three spikes inflate the cell's own variance: a paired
    t-test gives t = 1.69, p = 0.048, far from 0.05/39. A bootstrap-t is the same in kind.
  - *So each cell's critical value c is fixed before the data,* calibrated by simulation. Every c
    is at most 0.0175, so the counterexample's mean of 0.0240 fails in every cell.
- **The calibration** (`2026-11-19-open-port-2-registration/plan.py`, output `plan.txt` and
  `plan.json`; 100,000 draws a cell and size; seed fixed):
  - *each cell's c* is the largest of the (1 − 0.05/39) quantiles of its mean under no effect, in
    three noise models:
    - the cell's own 12 residuals, centred on its mean;
    - its corpus's pooled 36;
    - all 468 guard residuals.

    The last carries the first attempt's spike rate: |residual| > 0.1 in 5.8% of rounds, and
    > 0.15 in 2.1%.
  - *each model* resamples rounds independently with replacement.
  - *each cell's rounds* are the smallest on the grid giving power 0.80 at a true +2% slowdown,
    under the cell's own residuals (decision 2).
  - *the extension* is one `ab run` a batch: 18 batches (`open_port2.py schedule`), each a count
    and a round count with its corpora, into `guards-t{threads}-r{rounds}.jsonl`.

## Its error rates, stated before the data

**The false-fail rate** (no true effect on any guard), P(at least one of the 39 cells fails G2):

| noise model | P(any fails) |
|---|--:|
| each cell's own residuals | 0.013 |
| each corpus's pooled residuals | 0.012 |
| all the guards' residuals (the observed spike rate) | 0.034 |
| stress: 2% of every cell's rounds a spike of ±0.2 to 0.4 | 0.153 |

- **These are model-based, not exact.**
  - *The models assume* the rounds are independent within a cell.
    - The pairing takes out what both arms share in a round.
    - The arm order follows the harness's fixed AB/BA cycle, which balances position but doesn't
      randomise it.
    - Serial dependence and carryover are not modelled.
  - *The family* multiplies the 39 cells' pass rates, assuming independence across cells.
  - *The stress row* is a spike rate about the first attempt's 95% upper bound for spikes above
    0.15 (3.9%), at larger magnitudes. It shows how size degrades if the i9's next run is spikier.
- **The memory guard's** false-fail rate is about 0 (≤ 0.0015) by resampling the first attempt's
  peak RSS, both arms from one arm's spread. That is an empirical estimate, not a bound.
- **S, restored exactly as registered, has a false-fail rate of 0.148** under its own noise (8
  rounds), and power 0.55 at +2%.
  - *So the overall guard false-fail rate isn't at most 5%.* It is about 0.15 to 0.18 under the
    planning models, almost all of it S.
  - *S is the Coordinator's decision 3:* it isn't what failed, and changing it isn't in this
    registration's scope.

**The power,** per cell, beside G's under the same noise. G2 has each cell's own rounds; G has 12.

| corpus | threads | rounds | c | G2 +1% | G2 +2% | G2 +3% | G +1% | G +2% | G +3% |
|---|--:|--:|---|--:|--:|--:|--:|--:|--:|
| ho-code-l1 | 8 | 288 | 0.0155 (x1.0156) | 0.12 | **0.83** | 1.00 | 0.31 | 0.49 | 0.65 |
| ho-code-l1 | 10 | 336 | 0.0148 (x1.0149) | 0.15 | **0.86** | 1.00 | 0.31 | 0.46 | 0.62 |
| ho-code-l1 | 20 | 336 | 0.0147 (x1.0148) | 0.18 | **0.84** | 1.00 | 0.38 | 0.53 | 0.67 |
| ho-code-l9 | 8 | 168 | 0.0164 (x1.0165) | 0.00 | **0.94** | 1.00 | 0.21 | 0.59 | 0.91 |
| ho-code-l9 | 10 | 600 | 0.0145 (x1.0146) | 0.18 | **0.86** | 1.00 | 0.43 | 0.54 | 0.65 |
| ho-code-l9 | 20 | 192 | 0.0156 (x1.0157) | 0.07 | **0.87** | 1.00 | 0.26 | 0.49 | 0.73 |
| ho-prose-l1 | 8 | 120 | 0.0162 (x1.0163) | 0.07 | **0.80** | 1.00 | 0.24 | 0.51 | 0.77 |
| ho-prose-l1 | 10 | 240 | 0.0144 (x1.0145) | 0.17 | **0.88** | 1.00 | 0.31 | 0.48 | 0.67 |
| ho-prose-l1 | 20 | 120 | 0.0159 (x1.0160) | 0.10 | **0.80** | 1.00 | 0.26 | 0.51 | 0.75 |
| ho-prose-l9 | 8 | 96 | 0.0149 (x1.0150) | 0.13 | **0.87** | 1.00 | 0.31 | 0.55 | 0.79 |
| ho-prose-l9 | 10 | 72 | 0.0173 (x1.0175) | 0.00 | **0.95** | 1.00 | 0.66 | 1.00 | 1.00 |
| ho-prose-l9 | 20 | 72 | 0.0171 (x1.0172) | 0.00 | **0.92** | 1.00 | 0.47 | 0.92 | 1.00 |
| ho-structured-l1 | 8 | 96 | 0.0149 (x1.0150) | 0.09 | **0.91** | 1.00 | 0.19 | 0.55 | 0.87 |
| ho-structured-l1 | 10 | 96 | 0.0155 (x1.0156) | 0.13 | **0.81** | 1.00 | 0.23 | 0.48 | 0.79 |
| ho-structured-l1 | 20 | 96 | 0.0150 (x1.0151) | 0.07 | **0.91** | 1.00 | 0.35 | 0.54 | 0.83 |
| ho-structured-l9 | 8 | 96 | 0.0149 (x1.0150) | 0.12 | **0.88** | 1.00 | 0.21 | 0.51 | 0.79 |
| ho-structured-l9 | 10 | 96 | 0.0147 (x1.0148) | 0.13 | **0.89** | 1.00 | 0.22 | 0.54 | 0.80 |
| ho-structured-l9 | 20 | 72 | 0.0171 (x1.0172) | 0.00 | **0.94** | 1.00 | 0.51 | 1.00 | 1.00 |
| ho-wikidata-l1 | 8 | 72 | 0.0174 (x1.0176) | 0.00 | **0.96** | 1.00 | 0.74 | 0.99 | 1.00 |
| ho-wikidata-l1 | 10 | 72 | 0.0175 (x1.0177) | 0.00 | **0.86** | 1.00 | 0.43 | 1.00 | 1.00 |
| ho-wikidata-l1 | 20 | 96 | 0.0146 (x1.0147) | 0.05 | **0.96** | 1.00 | 0.32 | 0.61 | 0.88 |
| onebyte | 8 | 96 | 0.0148 (x1.0149) | 0.08 | **0.92** | 1.00 | 0.33 | 0.55 | 0.84 |
| onebyte | 10 | 96 | 0.0146 (x1.0147) | 0.09 | **0.93** | 1.00 | 0.35 | 0.55 | 0.83 |
| onebyte | 20 | 72 | 0.0171 (x1.0172) | 0.01 | **0.85** | 1.00 | 0.28 | 0.79 | 0.99 |
| pattern | 8 | 96 | 0.0146 (x1.0147) | 0.12 | **0.92** | 1.00 | 0.19 | 0.62 | 0.98 |
| pattern | 10 | 96 | 0.0147 (x1.0148) | 0.06 | **0.96** | 1.00 | 0.16 | 0.60 | 0.94 |
| pattern | 20 | 96 | 0.0148 (x1.0149) | 0.08 | **0.93** | 1.00 | 0.16 | 0.54 | 0.95 |
| random | 8 | 72 | 0.0173 (x1.0175) | 0.00 | **0.93** | 1.00 | 0.53 | 1.00 | 1.00 |
| random | 10 | 96 | 0.0149 (x1.0150) | 0.11 | **0.89** | 1.00 | 0.19 | 0.48 | 0.84 |
| random | 20 | 72 | 0.0173 (x1.0175) | 0.00 | **0.98** | 1.00 | 0.80 | 1.00 | 1.00 |
| records | 8 | 72 | 0.0173 (x1.0175) | 0.00 | **0.91** | 1.00 | 0.45 | 0.98 | 1.00 |
| records | 10 | 72 | 0.0170 (x1.0171) | 0.01 | **0.81** | 1.00 | 0.35 | 0.59 | 0.88 |
| records | 20 | 72 | 0.0171 (x1.0172) | 0.00 | **0.95** | 1.00 | 0.60 | 0.96 | 1.00 |
| runs | 8 | 240 | 0.0143 (x1.0144) | 0.17 | **0.88** | 1.00 | 0.30 | 0.49 | 0.69 |
| runs | 10 | 168 | 0.0161 (x1.0162) | 0.04 | **0.84** | 1.00 | 0.35 | 0.54 | 0.75 |
| runs | 20 | 288 | 0.0151 (x1.0152) | 0.16 | **0.82** | 1.00 | 0.37 | 0.52 | 0.66 |
| text | 8 | 96 | 0.0147 (x1.0148) | 0.16 | **0.87** | 1.00 | 0.26 | 0.53 | 0.78 |
| text | 10 | 96 | 0.0149 (x1.0150) | 0.14 | **0.86** | 1.00 | 0.22 | 0.53 | 0.86 |
| text | 20 | 72 | 0.0173 (x1.0175) | 0.00 | **0.83** | 1.00 | 0.26 | 0.88 | 1.00 |

- **At +2%,** every cell reaches 0.80 (the lowest is 0.80, ho-prose-l1 at 8 and 20 threads). G
  ranges from 0.46 to 1.00.
- **At +3%,** G2 is 0.997 to 1.00, against G's 0.62 to 1.00.
- **At +1%, G2 is weaker than G on every cell.**
  - *How much:* G2 is 0.00 to 0.18 per cell, against G's 0.16 to 0.80.
  - *Why:* G's point bound at 1.02 fails noisy cells often whatever the effect (its false-fail
    rate); G2 doesn't.
  - *What still holds for smaller losses:* the policy's protections stand, unchanged
    (PERFORMANCE-POLICY §2).
- **No claim is made that G2 dominates G at every slowdown above 2%;** the table states the points
  computed.
- **The power is model-based.**
  - *It assumes* the next run's noise is like the first attempt's, cell by cell, from only 12
    rounds a cell.
  - *Where noise is spikier,* power is higher and size is worse. Where noise is calmer, power is
    lower at the same c.
  - *Nothing is adjusted after the data:* no rounds are added, and no cell is re-tested.

**The time.** The full schedule meets every power target, at about **6.7 hours** on the i9:
- the decision run, about 38 minutes;
- S, under a minute;
- the extension, 4,932 pairs (9,864 decodes, each after its 2 s settle), about 5.75 hours;
- the context and the census, about 15 minutes.

The Coordinator's 2.5 hours isn't reachable with every cell at 0.80. Capped schedules for the
Coordinator's choice:

| schedule | about | cells below 0.80 at +2% (G2's power, G's) |
|---|---|---|
| full (this draft) | 6.7 h | none |
| capped at 192 rounds | 5.6 h | 7: ho-code-l1 ×3 (0.47–0.57; G 0.46–0.53), ho-code-l9 10t (0.32; G 0.54), ho-prose-l1 10t (0.76), runs 8t (0.76), runs 20t (0.61) |
| capped at 144 | 5.0 h | 10 |
| capped at 96 | 4.4 h | 12, several below G |

A cap changes only `bench/open-port-2-schedule.json` (rounds, with c recomputed by `plan.py` at the
capped rounds) and the disclosed cells.

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
  5. S, as registered;
  6. the guards' extension, batch by batch;
  7. the verdict;
  8. the context and the census, as before.
- **The untimed preflight** (`just bench-open-port-2 … dry`) is re-run at the corrected head. It is
  recorded below, and its bundle archived beside this note.

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
- P1, P2, H or S, as registered;
- a guard's memory;
- G2 on any of the 39 cells.

**ADOPTS** otherwise.

`bench/test_open_port2.py` checks the scorer and the guard, 29 checks:
- ADOPTS with guards of no effect;
- G2 failing a guard truly 2% slower (ho-code-l9 10t, text 8t, records 20t) or 3% slower;
- G2 passing guards of no effect with symmetric spikes of ±0.29;
- **the Researcher's counterexample:**
  - as a cell's extension, G2 fails it;
  - directly, its mean exceeds every cell's c;
- S failing as registered, and a ninth sustained round giving VOID;
- P1, P2, H and the memory guard, as registered;
- **the cross-file joins:**
  - text's output digest different in the extension: VOID;
  - the port's binary another path in one batch: VOID;
- a missing or stray batch file, threads 8.5, a NaN wall, a short batch, another port, an input off
  the manifest, a dry run: each VOID;
- the schedule's own consistency;
- the successor's one-attempt guard on scratch repositories.

Disabling G2's comparison fails five of those checks, and disabling the joins fails two.

## Context, reported and not judged

As in the first registration:
- the gate's cost, `gate.txt`;
- the eager/open crossover, `crossover.txt`;
- the per-block census, `census.jsonl`.

`verdict.txt` also reports each guard cell's geometric mean and interval over all its rounds.
