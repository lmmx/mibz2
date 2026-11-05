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
