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
