# The run-fed leader's second workstation attempt: VALID, INCONCLUSIVE; the sequence closes

**Run:** `run-fed-leader/20261109T201013Z`, Louis's C5 submission (#145, 6750297742).
- **Bundle:** on `evidence` at `65ab4b95`, SHA-256 `7e36a5a75903e31be1c1722382ef0098a9505033a5831b5b3e92f25a811be188`.
- **Head:** the freeze commit `1af3888a`, frozen at `2fab12ca` (#305, §R10 to §R12 of `2026-11-09-run-fed-leader.md`, bench 3.6.0).
- **Read in:** LOG 6750327222.

## The registered reading

The run passed every admission check:
- the preflight returned PASS, with every required collector exiting 0;
- all 140 records report `termination: success`;
- the corpus was `82f65ace…`;
- the diagnostics are complete.

Each arm is read against `mibz2-default33`, over 20 rounds at 10 threads, on the full data with nothing excluded. Intervals are 95% t-intervals.

| arm | wall | CPU |
|---|--:|--:|
| `pipe3-runfed-dir` (primary) | 1.0545 [0.9290, 1.1969] | 1.0487 [0.9142, 1.2031] |
| `pipe3-runfed-ctl` (control) | 0.9828 [0.8597, 1.1236] | 0.9763 [0.8438, 1.1296] |
| `pipe3-both` | 0.9376 [0.8307, 1.0584] | 0.9350 [0.8253, 1.0594] |
| `pipe3-runfed-reg` | 0.9280 [0.7367, 1.1690] | 0.9192 [0.7201, 1.1733] |
| `pipe3-runfed-ls` (secondary) | 1.0162 [0.8525, 1.2114] | 0.9861 [0.8189, 1.1874] |
| `pipe3-runfed-hs` (secondary) | 1.0079 [0.8189, 1.2406] | 0.9664 [0.7710, 1.2114] |

**Verdicts:**
- **Primary:** INCONCLUSIVE.
- **Control fidelity:** FAILED. The ctl/default33 interval, [0.86, 1.12], lies far outside the registered [0.98, 1.02].
- **Secondaries:** INCONCLUSIVE.
- **Captures:** VALID, with no perturbation shown.

## The close (the Researcher's wording, REVIEW 6750405677)

> The registered run-fed-leader timing sequence on the i9 is closed. Each of its two separately
> reported attempts was VALID and primary-INCONCLUSIVE, and each failed control fidelity.
> Neither establishes the registered performance benefit or clean run-fed attribution. The
> second attempt's interval describes that attempt alone; the attempts are not pooled and no
> third attempt follows. The cause of the reported wall/CPU bimodality remains unresolved.

The first attempt is `run-fed-leader/20261109T080633Z`, recorded in
`2026-11-09-run-fed-leader-workstation-result.md`. The repeat policy was R11.6, decided under C6 as amended in #306. Louis confirmed the authority in the Coordinator's session (LOG 6747734847).

## Observed, descriptive only

No pair was excluded, replaced or reweighted because of anything below.

**The walls were bimodal in every arm.**
- **Fast runs:** 28 records at about 0.13 to 0.16 s and about 1.0 CPU-s.
- **Slow runs:** 112 records at about 0.29 to 0.37 s and about 2.1 CPU-s.
- **Pattern:** the fast records came in short streaks. Each arm's coefficient of variation was 0.17 to 0.29.

**The host was busier than for the run just before it.** During this run the median `load1` was 3.1 and `others_busy` was 0.175. During `service-guarded/20261109T195345Z`, about 20 minutes earlier, they were 0.41 and 0.09. That run's 10-thread walls were tight, at 0.126 to 0.158 s.

**The Researcher's reading (REVIEW 6750405677):**
- **What the precision depends on.** The variance that matters is that of the paired log difference. A state shared by both members of a pair cancels; a switch of state between them does not.
- **What isn't shown.** These observations establish neither that this harness can't reach fidelity nor that the load caused the two modes.
- **Descheduling alone doesn't fit.** Wall and CPU doubling together rules out pure descheduling with unchanged CPU work, assuming comparable accounting and work. Spinning, contention, frequency, or a difference in the work executed all remain open.
- **The endpoint labels aren't a clock trace.** The endpoint H labels are not an in-run frequency trace.
- **The two runs aren't a controlled comparison.** They don't make a quiet-versus-busy experiment.

## For later registrations on this harness

The Researcher's advice: run a bounded, untimed audit of the accounting and the mechanism before any quiet-host condition.
- **The audit's limits.** These two load readings can't choose a threshold or prove one sufficient.
- **What must be fixed before outcomes:** any eligibility window, timeout or refusal, and stopping rule.
- **What is never allowed:** filtering for the favourable mode, or retrying until a run passes.

This grants no new run authority, and it is not a third attempt.
