# The one-pass build in production's one-core kernel (pre-registered)

- **The change:** `bwt::build` writes each entry whole, `ll[s] | i << 8` at slot s.
  - Today it first copies the last column into the table (an init pass), then
    read-modify-writes each entry's link in.
  - The cooperative kernel's fused build has worked this way since #81, but
    production's one-core build has not.
  - The entry format (32 bits: the link above the byte) and every table are unchanged.
- **Motivation, not a forecast:**
  - In the compact-Ψ run (`2026-10-29-compact-psi.md` §5.3), the `v` arm's build ran at
    1.28× production's at one block and 1.49× at four. That arm stores the link alone, a
    different table.
  - A one-pass build that keeps the byte must read L at each slot, so its gain is not v's.
    No gain is claimed before the paired runs below.

## What it reaches

`bwt::build` serves every one-core path:
- the serial decoder;
- `Plan::Workers`, the default below 6 physical cores, so on CI's runners and on 4-core
  machines;
- the cooperative kernel's fallback for blocks below `MIN_COOP`.

From 6 physical cores the default plans teams. Their fused build is untouched, so on the
i9 the rows from 6 threads up are a control.

## Soundness

The design is `coop::build_fused`'s at one core:
- **Writes:** each write is checked against its bucket's end.
- **The end check:** after the scatter, every bucket's cursor must equal its end. That
  proves every slot below n was written exactly once.
- **The length** is set only after that check.

**Counts that aren't the column's own** (never the decoder's, whose MTF stage computes
them from the same column):
- They are refused before the scatter, or at the first write past a bucket's end.
- `build` then runs the two-pass build unchanged, so its table is what it always was.

**Tests:**
- the one-pass table equals the two-pass one on runs, random and mixed columns of every
  length from 0 to 40 and 3,000;
- wrong counts are refused and give the two-pass table (one count moved between buckets,
  and one short);
- a Miri test of the unchecked writes and the length set after them;
- the existing exact-table test (`run_stores_equal_one_pass`), unchanged.

**The decoder's output** equals `bzip2 -d`'s on the bench corpus, its -3 recompression and
16 crafted inputs, at 1, 2 and 4 threads (VM).

## Measured before registration (VM, indicative only)

- **The run:** `bench/stages.py`, 6 rounds, one core, the bench corpus, master against
  this branch, paired.
- **The build stage:** **1.117 (1.051–1.186)**. The walk: 1.003. The whole one-core decode:
  1.029 (0.993–1.066).
- **The VM's noise:** MTF, whose code is unchanged, read 1.052, so differences of about 5%
  are within it.

## The measurements that decide (fixed now)

**1. CI's bench on the pull request** (the paired base-against-head A/B the repository
merges on):
- the whole decoder at 1, 2 and the runner's CPUs;
- the stage table (`bench/stages.py`).

**It passes if:**
- the build stage is faster, with its interval above 1;
- the whole decoder is not slower at any thread count (no interval wholly below 1);
- every run gives the same bytes.

A gain in the whole decoder beyond noise is hoped for, not required. The build is about a
sixth of a one-core decode, so even a 1.12× build is about 2% of the whole.

**2. The i9** (`just bench-one-pass`): `bench/variants.py`, this build against the
`v0.2.0` tag, whose `src/` differs from it by this change alone. 1, 2, 4, 6, 8 and 10
threads, 10 rounds, paired.
- **Predicted:** faster at 1–4 threads, by a few percent of the whole decoder. At 4,
  memory pressure may make the init pass's traffic count for more.
- **At 6–10 threads:** within noise of 1 (the control).
- **Any slowdown** at 1–4 beyond noise rejects the change.

## The result (measured; merged as #114, 6a341f6)

### 1. CI's bench (the pull request, head f121097, 10 paired rounds, a 2-core runner with SMT)

**The registered criteria, all met:**
- **The build stage is faster:** −6.0% (−8.5%, −3.5%).
- **No thread count is slower:** none slower in wall time, across 21 corpus and thread-count
  rows.
- **The same bytes:** on every run.

**What the criteria didn't gate, recorded as measured:**
- **No whole-decoder gain beyond noise.** The Wikidata blocks: +0.8% (−1.2%, +2.8%) at
  1 thread, +0.8% (−0.7%, +2.3%) at 4.
  - The build is 16.7% of a one-thread decode there, so its −6% predicts only about −1%
    overall, within the noise.
- **CPU-seconds rose slightly in three rows,** each interval excluding no change:
  - Wikidata at 4 threads: +1.1% (+0.2%, +2.1%);
  - records at 2: +1.0%; records at 4: +2.0% (+1.0%, +2.9%).
  - The runner's 4-thread runs share SMT siblings. The bench's own summary read "Worse (CPU
    cost), and no measure better".

### 2. The i9 (`just bench-one-pass`: this build against `v0.2.0`, 10 paired rounds)

Files: `2026-10-29-one-pass-build/` (`variants-20261029T230112Z.*`, `one-pass-console.log`).

| threads | speed | CPU-seconds | the plan |
|--:|--:|--:|---|
| 1 | **1.048 (1.026–1.070)** | **0.954 (0.934–0.975)** | the serial decoder (`bwt::build`) |
| 2 | 1.025 (0.994–1.056) | 0.980 (0.953–1.006) | workers (`bwt::build`) |
| 4 | **1.035 (1.010–1.060)** | **0.971 (0.945–0.998)** | workers (`bwt::build`) |
| 6 | 0.999 (0.973–1.026) | 1.002 | teams (the fused build, unchanged): the control |
| 8 | 1.007 (0.986–1.028) | 0.994 | the control |
| 10 | 0.998 (0.961–1.036) | 1.001 | the control |

**The registered i9 reading:** faster at 1–4 threads, the control within noise of 1, no
slowdown at 1–4. **Met.** The bytes were the same on every run.

### 3. The two machines disagree, and both stay in the record

**Where the code changes,** the i9 shows a whole-decoder gain of 3.5–4.8%, at less CPU.
CI shows none beyond noise, and a 1–2% CPU rise on its shared-core runs.

**Not established:** why. The hypothesis is the extra loads, of L at each slot.
- On SMT siblings they compete with the sibling's work.
- On the i9's physical cores they cost less than the init pass's 3.6 MB of stores they
  replace.
- These measurements don't separate that from the runners' noise. The CI CPU rise is a
  small, real signal for SMT-shared 2-core machines, and is kept as such.

### 4. The shipping decision, made separately

**The pre-registered experiment succeeded** on both of its registered measurements.

**Merging was a separate engineering decision,** made by the user after both results.
- **The basis:** the i9 shows a consistent gain at the thread counts the change reaches,
  with less CPU, and none at the control.
- **Against it, weighed:** CI's inconclusive whole-decoder result and its small CPU rise,
  recorded above, not erased.
