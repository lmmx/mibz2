# Synthesis, eighteenth pass: the adaptive seam pays only where the scanner binds; CI cannot resolve sub-percent RSS bounds

An addendum to the seventeenth pass (#1230), which a parallel session wrote over the same window.
This one leaves out what that pass already says (0.5.17's ~2% knee loss and `open-cap-prescan-cost`)
and adds two readings it doesn't make.

**What I read:**
- `origin/evidence d6bc8ca2` (`scanner-adaptive-seam/run-20261127T192609Z`, #1144 at `0fda2381`),
  SHA-256 `c14e53b4…c57c59`, checked against the archived `.sha256`;
- the 0.5.17 release-scaling and crossover rows (`5182d546`, `0ffba60b`, both checked);
- the CI deltas for #1179 and #1222.

Nothing re-run. **Not re-scored:** `scanner-adaptive-seam-window` (held) owns that verdict.

## 1. The adaptive seam is NOT ADOPTED exactly where the coupled model says it must be

The bundle is unpinned, 16 paired rounds, ratios head/base:

| threads | workers | wall [95%] | step | peak RSS |
|---:|---:|---|---|---:|
| 6 | 6 | 0.9970 [0.9937, 1.0003] | inconclusive | 0.908 |
| 8 | 8 | 0.9901 [0.9758, 1.0045] | inconclusive | 0.927 |
| 9 | 9 | **0.9658 [0.9434, 0.9887]** | FASTER | 0.912 |
| 10 (judged) | 8 | 1.0019 [0.9866, 1.0174] | inconclusive | 0.931 |
| 12 (judged) | 8 | 0.9841 [0.9666, 1.0018] | inconclusive | 0.927 |

- in_maint fell 96%, and the clock split shows a common clock.
- The verdict is NOT ADOPTED, and C2 stands.

Read beside `text-knee-i9` (Wikidata's knee is at k = 9) and DECODER.md's tie, the pattern is the
predicted one. A scanner-side cut pays wall only where the scanner binds. Here that is 9 workers.
Both judged counts run 8 workers, where the workers bind and mask the cut.

- **The memory gain is unconditional.** Peak RSS falls 4.2–5.8 MiB (0.91–0.93x) at every count,
  the judged ones included. The registration treats RSS only as a bound (≤ 1.05).
  - Whether a memory-only adoption is admissible is a PERFORMANCE-POLICY question for the claim's
    holder. It is not a re-reading of the wall verdict.
- **A worker-side lever revives it.** #1217's forward lanes cut worker CPU 8–11% on the VM, behind
  an opt-in feature.
  - If `forward-emission-run` passes and the lanes become the default, 8 workers stop binding. The
    scanner then binds at the judged counts, and the seam becomes payable there. So does
    `scanner-a2-fast-build`'s S′ cut.
  - Any dead-ends row for the seam should therefore be scoped to "alone, on the 0.5.16/0.5.17
    worker cost". A composed re-test (lanes + seam, the same judged counts) is the rescue.
  - I open no claim for it until `forward-emission-run` has a verdict. It is gated on that verdict,
    as `low-thread-open-unification` is.

## 2. The CI runner cannot resolve the RSS bounds some claims set

#1179's AVX2 ring CRC allocates nothing, yet its paired CI run moved 4T peak RSS by +7%. #1222's CI
bench moved RSS from −3.2% to +1.4% on routes it left unchanged for Wikidata. #1179's 0.5% RSS
limit is far below what this runner resolves.

- *Possible mechanism, untested:* a faster consumer changes how full the window gets, and with it
  the peak RSS.
- *Implication:* code that allocates nothing should be judged on RSS at a fixed window, or on the
  i9, not on the CI runner's peak.

## 3. Pinned i9 rows are unimodal

The release-scaling and crossover arms are pinned (`cpus` 0..k−1) below 12 threads. None of their
rows shows the sixteenth pass's ~1.5x slow mode. At 8 threads, for example, all 10 rounds of each
arm fall within 0.065–0.070 s.

This fits the slow mode being the unpinned state. It also means pinned release comparisons are not
confounded by mode mix.

## What changed

Nothing but this entry. No claims and no priority changes. `synthesise` stays open.
