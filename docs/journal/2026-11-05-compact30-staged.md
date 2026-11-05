# Compact30 under the staged walk (C-resolve): registration

Registered before any CI timing of this branch. The decision rule may be amended only in a
commit made before the first bench comment on this PR, and each amendment says what changed
and why.

## The change

The Optimiser proposed this build in REVIEW 6653724458: a retest of compact30's smaller table
under today's staged walk. Compact30 stores 30 T-vector entries in a 64-byte record, plus a
RAW area, about 2.13 bytes a row against Segmented's 4.

- **The port (b4786f66).** Its encoding and constructor are kept verbatim. Its own unstaged
  walk is kept only as a `cfg(test)` reference (`room_unstaged`).
- **The walk** is master's staged busy phase (#171, #181) with a compact step.
- **RAW arrivals: C-resolve.** A RAW arrival saves its payload index and parks. Its byte is
  resumed in a resolve class after all 32 lanes have stepped the offset, into the tile column
  the arrival left empty. Every lane then holds one byte or a START mark at each offset, so
  master's handover, chunk and flush logic carries over unchanged. The choice between RAW
  schemes was made and logged before registration (channel log, 2026-11-05). The ordinary step
  is the same in every scheme, and C-resolve keeps that invariant. Flush-at-wait, D, is the
  named follow-up.
- **Arms.**
  - **C = `Compact30`:** this PR's default at the five sites (`src/lib.rs` ×3,
    `src/parallel.rs` ×2) through `DefaultInverse`.
  - **L = `Compact30Split`:** the same source under `const S: bool = true`. Fields sit at
    60g + 2j and headers at 60N + 4g. It is a layout control.
- **Cooperative teams** (6 or more physical cores) stay on the T-vector.
- **Walk-only rows** `c30` and `c30split` are added to `examples/sweep.rs`, each paired by
  round with the default room walk at 32 lanes, L1 prefetch and 512 segments. The sweep is in
  neither the bench nor the PR tier.

## Measured, untimed

- **Ordinary step, per lane,** in the shipped build (rustc 1.97.0, fat LTO, `codegen-units = 1`;
  listing archived with the branch work):

  | arm | instructions | loads | stores | prefetches |
  |---|--:|--:|--:|--:|
  | P (master) | 8 | 2 | 2 | 1 |
  | C | 12 | 3 | 2 | 1 |
  | L | 20 | 3 | 2 | 2 |

  There is no spill or call in the 8-lane body. The resolve and START arms are out of line.
- **ρ, RAW arrivals per byte,** counted offline by test-only counters:

  | corpus | ρ | largest segmented block |
  |---|--:|--:|
  | wikidata | 0.00708 | 0.00853 |
  | text | 0.01118 | |
  | records | 0.00912 | |
  | random | 0.01100 | 0.0362 |

  On wikidata, 6.2% of arrivals fall in the tail.
- **Tests:**
  - every lib and integration test passes, with compact30 twins of the in-place and corpus
    tests in both layouts;
  - the C/L differential is segment-for-segment against the unstaged reference;
  - every master staged-walk counter is required, plus the RAW counters;
  - the trimmed Miri set passes (16 cases, about 22 minutes of test time);
  - clippy, fmt and MSRV 1.80 (`--lib`) are clean.

## The prediction, recorded before any run

The binding census (`2026-11-05-binding-census.md`) predicts **C slower at 4 threads** by
three independent routes:
- **the no-go archaeology:** +2% to +7%;
- **the walk front's falsifier:** −2% to +11%, central +3% to +5%;
- **the accounting falsifier:** the step's added work lands on the binding term.

The reason: on CI the four-copy walk is mostly a lane-independent per-byte term. C adds 4
instructions and a load to every step, and its L2 gain acts only on the 32–38% miss term.

## The decision rule

The incumbent (P) stays unless C wins. A neutral result keeps P.

- **Correctness (required):** CI green, Miri included; the same bytes on every corpus in
  every bench run.
- **Benefit (required):** `wikidata` 4-thread wall faster beyond noise in at least one run,
  and its point estimate faster in the other. Peak RSS falls by construction (about half the
  table). It is reported, but it is **not** a benefit that can carry a wall regression.
- **No harm (required):** a wall row flagged slower in a run is a harm signal. One more run on
  the same head settles it: flagged again, it is a harm and the rule fails; not flagged
  again, it is reported as unresolved. At most three runs.
- **Power:** the 4-thread rows' 95% half-widths are about ±2–5%. A change inside that band is
  reported as not shown.

**What each outcome says about the regime,** fixed now:
- **C faster at 4 threads beyond noise** on a Zen 4/5 or Intel runner: the L2-footprint term
  binds enough to pay for the step's added work. The memory-side reading of the walk's co-run
  excess gains support.
- **C slower at 4 threads beyond noise:** the per-byte core term binds. That supports the
  core-side reading and closes compact30 as the production table on CI. It would reopen only
  if the sibling-pair matrix shows the walk's excess to be memory-side.
- **Not shown:** neither reading gains, and the PR closes.

**Walk-only sweep, diagnostic, not decided.** Two dispatches of `sweep.yml` on this branch,
`c30` and `c30split` against the default, alone and at four copies. They are reported per
CPU. C − L is the cost of L's 8 extra address instructions at the same footprint, read from
the two paired ratios.

**If the rule fails,** the journal records the regime evidence, the PR closes, and no further
compact30 production build is made unless the pair matrix reopens it.

## Result: rule failed; PR #195 closed

The code stays on `perf/c30-staged` (head ba94ff5c). This entry, and the evidence in
`2026-11-05-compact30-staged/`, land on master on their own. The bench comments are kept as
`bench-run1-9v74.md` and `bench-run2-7763.md`. Both runs compare head ba94ff5c with base
448d23d, and both gave the same bytes on every corpus.

| `wikidata` | run 1: EPYC 9V74 (Zen 4) | run 2: EPYC 7763 (Zen 3) |
|---|---|---|
| 1-thread wall | +11.4% (+10.1, +12.7) slower | +13.7% (+11.8, +15.6) slower |
| 2-thread wall | +9.1% (+8.1, +10.2) slower | +12.8% (+11.8, +13.9) slower |
| 4-thread wall | +4.8% (+3.2, +6.4) slower | +7.7% (+5.9, +9.6) slower |
| 4-thread CPU | +7.0% (+5.9, +8.2) slower | +10.5% (+8.7, +12.4) slower |
| 4-thread amplification, head over base | 1.67× → 1.60×, −3.9% (−5.3, −2.5) | 1.68× → 1.64×, −2.7% (−4.6, −0.7) |
| 4-thread peak RSS | 77 → 71 MB (−7.7%) | 77 → 70 MB (−8.7%) |

**The rule:**
- **Benefit:** not met. Neither run is faster at 4 threads.
- **Harm:** the 4-thread wall was flagged slower in run 1, and flagged again in the settling run
  on the same head, so the rule fails.
- **Peak RSS** fell, by construction. It cannot carry a wall regression.

### What it says about the regime

Both terms the census contested are real, and on CI the per-byte core term dominates.

- **The footprint term.** The smaller table cuts the walk's co-run amplification beyond noise
  on both models. The cut is −3.9% on Zen 4's 1 MB L2 and −2.7% on Zen 3's 512 KB L2: larger
  where more of the table can stay in L2, which is consistent with the hit-rate model. The cut
  alone does not show the cause. The L2 reading rests on the walk-only sweeps below.
- **The core term.** The step's added work (12 instructions and 3 loads against 8 and 2) costs
  +11% to +14% of 1-thread CPU.
- **How the terms combine.** Amplification is CPU at 4 threads over CPU at 1
  (`bench/report.py`). The 4-thread change is therefore the 1-thread change times the
  amplification change, by definition:
  - 1.114 × 0.961 ≈ 1.071, against +7.0% measured;
  - 1.136 × 0.973 ≈ 1.105, against +10.5% measured.

  The agreement is an identity, not a check (corrected after REVIEW 6660203300). What it does
  give is the split. In logs, the core term is about 3 to 5 times the footprint term.

The census's central prediction was +3% to +5% at 4 threads. The measured +4.8% and +7.7% are at
and above it.

### The walk-only sweeps (diagnostic): an L2-capacity cliff

Two dispatches of `sweep.yml` on the branch, 5 rounds each. Figures are against the room walk at
32 lanes, L1 prefetch and 512 segments; the files are under `sweep-42036331632/` and
`sweep-42036326478/`.

| walk only | Xeon 8573C, 2 MB L2 a core | EPYC 9V45 (Zen 5), 1 MB L2 |
|---|---|---|
| `c30` alone | −9.4% (−12.2, −6.5) faster | +21.4% (+20.0, +22.8) slower |
| `c30`, four copies, by CPU | +5.4% (slower), +2.7%, −0.8%, +4.1% | −5.9%, +27.7% (slower), +28.4% (slower), +0.8% |
| `c30split` alone | +14.1% slower | +78.3% slower |
| `c30split`, four copies | +36% to +46% slower | +30% to +77% |

Compact30's table for a 900k-row block is about 1.95 MB.

- **Alone on the 8573C,** it fits the 2 MB L2. There C's walk beats the room walk by 9.4%,
  despite its heavier step.
- **At four copies,** each core holds two tables (about 3.9 MB), and the advantage is gone.
- **On Zen 5's 1 MB L2** it never fits, and the extra step work shows plainly.

So the gain is L2 residency, not compactness as such. A table that stayed resident at four
copies would need about 1 byte a row on the 8573C, and about 0.5 byte on Zen 4/5.
- **One byte a row** is the block's own bytes, with nothing left for the rank checkpoints that
  let compact30 (2.13 bytes) undercut a stored row index (about 2.5 bytes at 900k rows).
- **Half a byte** is below the block's bytes. It needs an entropy-compressed rank structure, of
  the wavelet or run-length FM-index kind.
- **Either costs** several dependent loads and popcounts a step. Compact30's 4 extra
  instructions already sink it.

So residency at four copies is out of reach at any step cost the walk can bear. This is an
inference from the step counts, not a measurement.

**Parked, not pursued:** a model-specific dispatch to compact30 when the table fits the per-core
L2 at one thread a core. Today that means 2 MB L2 at 1–2 threads.
- No whole-decoder run landed on an 8573C. Its build cost and its 1-thread decoder figure are
  unmeasured.
- The i9 (256 KB L2) and the AMD runners never qualify.
- It would be revisited only if wikidata-pq's release target is such a machine, at one thread
  a core.

`c30split` loses everywhere, as its 20-instruction step predicts. The split layout buys nothing
at this footprint.
