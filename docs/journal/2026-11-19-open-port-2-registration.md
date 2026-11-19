# The open kernel's port for 0.5.7: the successor registration

*19 November 2026. The Pathfinder's draft, for the Researcher's prospective review through the
Coordinator (#145 QUEUED 7011091171). It is frozen before any data.*

**This is a new experiment, not a retry.** The first registration
(`2026-11-19-open-port-registration.md`) had one confirmatory attempt,
`open-port/run-20261119T125538Z`, and it was **NOT ADOPTED**. That verdict stands. This
registration doesn't reinterpret it, re-judge it or reuse its data as evidence.

**Why there is a successor.**
- *The first registration's guard rule G had a false-fail rate it never stated.* G failed a guard
  cell when its geometric mean exceeded 1.02, or when its 95% interval lay wholly above 1. Over 39
  guard cells whose intervals reached about ±9%, a port with no true effect fails G with
  probability about 0.96.
- *How the first attempt failed:* four guard cells failed G, all on corpora where the gates admit
  no block.
- *What the successor does:* it keeps everything else, and replaces G's wall-time part with a rule
  whose false-fail rate and power are stated here, before any data. The new rule is at least as
  strict as G at catching a true slowdown of 2% or more.

- **The change:** unchanged. The port, #630 (`perf/open-kernel`) at
  9743bbe0d51ba451b8b8ab9e6fd731c238940052, against v0.5.6 (530ed404).
- **The recipe:** `just bench-open-port-2`, on `bench/open-port-2`.
- **The scorer:** `bench/open_port2.py`. The first registration's `bench/open_port.py` is
  unchanged, and still re-judges the first attempt.
- **What it decides:** whether 0.5.7 ships `OpenCells` as the workers' kernel wherever 0.5.6 ran
  `LazyCells`. If not, 0.5.7 ships without it.

## What is the same as the first registration

Nothing in this list is dropped, loosened or re-weighted (PERFORMANCE-POLICY §2):
- **the builds, the machine and the inputs:** the same 18 inputs, checked against the same manifest
  `bench/open-port-inputs.sha256` before any timing (`verify`), and again when judged;
- **the roster:** the same 15 corpora at 8, 10 and 20 threads, the same primary, held-out
  replication and sustained cell, and the same 13 guards.
  - *The corpora with no admitted block stay guards.* They guard binary layout and dispatch cost.
- **the decision run, `runs.jsonl`:** byte for byte the first registration's recipe. One
  `ab run` a count, 12 paired rounds, the 2 s settle, unpinned.
- **P1, P2 and H, from those 12 rounds:**
  - *P1:* a wall gain on wikidata-x4 at every count, Holm at α 0.05;
  - *P2:* wikidata-x4's peak RSS at most 1.00 of v0.5.6's;
  - *H:* ho-wikidata-l9 FASTER at every count.
- **every guard's memory, from those 12 rounds:** peak RSS at most 1.10 of v0.5.6's, median over
  median;
- **every VOID condition,** applied to each of the four files;
- **the context and the census,** reported and not judged.

## What is new: the guards' wall-time rule, G2

| | the first registration's G (wall time) | the successor's G2 |
|---|---|---|
| cells | 39 guard cells, and the sustained cell | the same 40 |
| rounds | 12 (sustained: 8) | tier A 120, tier B 60 (sustained: 40) |
| a cell fails when | its geometric mean exceeds 1.02, or its 95% t-interval lies wholly above 1 | a one-sided exact Wilcoxon signed-rank test of "the port is slower" gives p ≤ 0.0012 |
| P(any fails), no true effect | about 0.96 (not stated in advance) | at most 0.048, by construction |

- **The test:**
  - *the data:* each round's paired log ratio, the port's wall over v0.5.6's;
  - *ties and zeros:* zero differences are dropped, and tied magnitudes take their mean rank;
  - *the null distribution:* exact, computed for the observed ranks;
  - *the level:* α = 0.048 over 40 cells (Bonferroni), so 0.0012 a cell.
- **Why its false-fail rate holds whatever the noise is like.**
  - *With no true effect,* the two builds are exchangeable in every round. The arms' order
    alternates between rounds, so each paired difference is symmetric about 0.
  - *So the test is exact:* each cell's size is at most 0.0012.
  - *Across cells:* Bonferroni holds under any dependence between cells, so P(any of the 40 fails)
    ≤ 0.048.
  - *Unlike the power below,* this bound assumes no noise model.
- **The whole guard family's false-fail rate is at most 0.05:**
  - G2's, at most 0.048;
  - the memory guard's (unchanged), at most 0.0015. That figure comes from resampling the first
    attempt's per-round peak RSS with both arms drawn from one arm's spread.
- **The rounds.**
  - *Tier A, 120 rounds a cell:* runs, ho-prose-l1, ho-code-l1 and ho-code-l9. These are the
    decision run's 12 rounds and 108 more (`guards-a.jsonl`).
  - *Tier B, 60 rounds a cell:* the other 9 guards. These are 12 and 48 more (`guards-b.jsonl`).
  - *The sustained cell, 40 rounds,* back to back, in place of 8.
  - *Each extension* is one `ab run` a count and a tier, with the decision run's arms, settle and
    unpinned scheduling. Its rounds are paired within the round, as the decision run's are.

## Its power, stated before the data

**The planning model** (`2026-11-19-open-port-2-registration/plan.py`; its output is `plan.txt`
beside it, at 1,500 draws, seed fixed):
- *the noise:* for each corpus, each cell's paired log ratios less the cell's median, pooled over
  the three counts and symmetrised;
- *the effects:* a true effect shifts every round's log ratio;
- *the source:* the first attempt's per-cell noise. Its outcome is not used, and nothing is tuned
  to it.

**The power at a true +2% slowdown,** per corpus, against the first registration's rule G, both
under the same noise:

| corpus | rounds | G at +2% | G2 at +2% | G2 at +3% |
|---|--:|--:|--:|--:|
| ho-prose-l1 | 120 | 0.49–0.51 | 0.76–0.78 | 1.00 |
| ho-code-l1 | 120 | 0.51–0.53 | 0.94–0.96 | 1.00 |
| ho-code-l9 | 120 | 0.50–0.52 | 0.94–0.95 | 1.00 |
| runs | 120 | 0.52–0.53 | 0.99–1.00 | 1.00 |
| ho-structured-l9 | 60 | 0.54–0.56 | 0.84–0.87 | 1.00 |
| ho-structured-l1 | 60 | 0.56–0.59 | 0.95–0.96 | 1.00 |
| text | 60 | 0.59–0.60 | 0.94–0.95 | 1.00 |
| pattern | 60 | 0.59–0.63 | 0.97 | 1.00 |
| onebyte | 60 | 0.69–0.71 | 1.00 | 1.00 |
| random | 60 | 0.70–0.72 | 1.00 | 1.00 |
| ho-prose-l9 | 60 | 0.73–0.76 | 1.00 | 1.00 |
| ho-wikidata-l1 | 60 | 0.80–0.81 | 1.00 | 1.00 |
| records | 60 | 0.84–0.85 | 1.00 | 1.00 |
| wikidata-x16 (sustained) | 40 | 0.65 | 0.94 | 0.97 |

`plan.txt` has every cell at +1%, +2%, +3%, +5% and +10%.

**How "at least as strict" is shown.**
- *The claim:* for every one of the 40 cells, and every true slowdown of +2%, +3%, +5% and +10%,
  G2 fails the cell at least as often as G does, under the same noise. The tolerance is 0.02 in the
  simulated rates.
- *The check:* `plan.py` runs both rules on the same simulated noise and lists every cell and
  effect where G2 falls short. It lists none.
- *Why it holds beyond the points checked:* G2's power rises with the effect faster than G's.
  - Its threshold sits below 2%.
  - Its rounds are 5 to 10 times G's.
  - Its rank statistic isn't moved by the spikes that widen G's interval.
- *The worst cell* is ho-prose-l1, which sets tier A's count. Its minimum for parity was 90 rounds;
  120 gives a margin of about 1.3×. Tier B's minimum was at most 40, and its 60 rounds give 1.5×.
  The sustained cell's minimum was 30, and 40 gives 1.3×. The margins cover the noise shape being
  estimated from only 36 rounds a corpus.
- **Disclosed: below 2%, G2 is not uniformly stricter.**
  - *Where:* at a true +1% slowdown, G2 catches less often than G on 10 cells:
    - ho-code-l1, ho-code-l9 and ho-prose-l1, at every count. For example, ho-prose-l1 at 8
      threads: 0.12, against G's 0.27.
    - ho-structured-l9 at 20 threads, by 0.02, within the simulation's error.
  - *Why:* G's point bound at 1.02 fails noisy cells often, whatever the effect. That is its
    false-fail rate, not sensitivity.
  - *The criterion:* this registration's strictness is stated for slowdowns of 2% or more, the
    bound the first registration itself drew. On the other 30 cells, G2 is as sensitive or more at
    +1% too.
- **What the planning model doesn't guarantee:**
  - *Power depends on the next run's noise,* which may differ from the first attempt's.
  - *Nothing is adjusted after the data:* no rounds are added and no cell is re-tested. The realised
    noise is reported beside the verdict.
  - *The false-fail bound doesn't depend on the model.*

**Tier A's corpora are its noisiest, not its failures.** Tiering follows the first attempt's noise,
which is spread, not outcome:
- ho-code-l1 and ho-code-l9 failed G there and are in tier A, for their spread;
- random failed G there too and is in tier B, since its spread is small;
- runs and ho-prose-l1 passed G there and are in tier A.

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
  5. the sustained cell;
  6. the guards' extension;
  7. the verdict;
  8. the context and the census, as before.
- **Time, from the first attempt's pace:**
  - the decision run, about 38 minutes;
  - the sustained cell, 2;
  - the extension, about 3 hours: 5,184 decodes, each after its 2 s settle;
  - the context and census, about 15;
  - in all, **about 3 hours 55 minutes**.
- **The untimed preflight** (`just bench-open-port-2 … dry`), in the cloud container at 9b3ed323
  (this branch's scorer, tests and recipe; nothing they run has changed since):
  - "the builds and all 18 inputs are the registered ones";
  - 16 of 16 outputs the same in both builds;
  - 29 of 29 scorer checks;
  - no timing field, and no `runq` probe.

  Its bundle is `2026-11-19-open-port-2-registration/preflight/run-20261119T141721Z.tar.gz`, sha256
  `0ead7c90dd686d8fea9952cdfb41c840e8ee4836d85f421388c5c7fd0e226a2b`.

## The rule (`bench/open_port2.py judge`, frozen with this registration)

**VOID, with no verdict,** where any of the first registration's VOID conditions holds of
`runs.jsonl`, `guards-a.jsonl`, `guards-b.jsonl` or `sustained.jsonl`, each against its own roster:
- a record missing, duplicated, failed, malformed, or outside the roster;
- a field of the wrong type or value;
- a non-finite or non-positive measurement;
- outputs that differ;
- an identity that doesn't hold;
- another machine;
- other workers on the Wikidata corpora;
- a dry run.

**NOT ADOPTED,** every failing condition named, where any of these fails:
- P1, P2 or H, as registered;
- a guard's memory;
- G2 on any of the 40 cells.

**ADOPTS** otherwise.

`bench/test_open_port2.py` checks the scorer and the guard, 29 checks:
- ADOPTS with guards of no effect;
- G2 failing a guard truly 3% slower, in either tier, and one truly 2% slower within its noise;
- G2 passing a guard with no effect but upward spikes, whose geometric mean exceeds 1.02;
- G2 failing the sustained cell truly 3% slower;
- P1, P2, H and the memory guard, as registered;
- each VOID of the four files' rosters, fields, identities and inputs;
- the exact p, equal to enumeration on 60 small samples with ties and zeros;
- the test's size at two levels;
- the family's constants;
- the successor's one-attempt guard on scratch repositories.

## Context, reported and not judged

As in the first registration:
- the gate's cost, `gate.txt`;
- the eager/open crossover, `crossover.txt`;
- the per-block census, `census.jsonl`.

`verdict.txt` also reports each guard cell's geometric mean and interval over all its rounds.
