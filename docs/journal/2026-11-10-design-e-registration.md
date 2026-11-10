# 2026-11-10: Design E, the line table: registration before any timing

**Status: registered before any CI timing, with no local timing taken. Research only: the
decoder never calls the line table, and the production default doesn't change. What's checked
so far: byte identity, the census and instruction counts.**

- **The design** (Pathfinder, `2026-11-10-by-data-class.md` §3, on #313 at `2e67726b1`): a
  backward table for data with short runs, where designs C and D (`2026-11-10-backward-table.md`)
  remove little. One 64-byte line holds 32 rows: the rows' bytes of L, a mask of each byte's
  first row in the line, and up to 9 bases. That is 2 bytes a row, and a step reads one line.
- **Built:** `src/line_table.rs` (a `#[doc(hidden)]` research module, unused by `decode`).
  - The table's build, and its walk in the LF direction: a plain walk, and two segmented walks
    in the shapes of `bwt::walk_segments` and `bwt::walk_segments_flagged`.
  - `examples/line_table.rs` runs the check, the census and the instruction counts.
  - Research rows in `examples/phases.rs`, which CI's stage harness records and the bench
    report shows (bench 3.5.0).
- **The question CI answers:** what a one-thread inversion costs with E's table in the
  T-vector's place, on the Wikidata blocks. It answers it as a ratio to production's build
  and walk, paired by run.
- **Predicted:** about 2.5× production's inversion at one thread on Wikidata, with the 95%
  interval's lower bound above 2.0. That would close E as built, by the falsifier below. The
  instruction counts already point that way. This entry records the prediction so that the
  bench can contradict it.

Labels: **checked** (bytes equal), **counted** (exact for the input: census, callgrind
instructions), **model** (derived, not observed). Nothing here is **measured**.

## 1. What was built

**The line** (`Line`, 64 bytes, aligned):
- bytes 0–31: the rows' bytes of L;
- bytes 32–35: `first`, the rows that are the first of their byte in the line;
- bytes 36–62: up to 9 bases of 3 bytes, in first-occurrence order. Each base is C(c) plus
  c's copies in the lines before.
- byte 63: the line's count of distinct bytes.

With more than 9 distinct bytes, the ninth base's place holds an index into an overflow list
(`u32`) of the bases past the eighth.

**A step from row i,** at offset o in its line, with c = L[i]:
- `eq` is the line's rows equal to c;
- c's slot s is the count of `first`'s bits below `eq`'s lowest bit;
- LF(i) is base[s] plus the count of `eq`'s bits below o.

The kernel is chosen at run time, as the decoder's kernels are. With AVX2, POPCNT and BMI2,
`eq` is one 32-byte compare. With POPCNT only, it is two 16-byte compares. The baseline uses
the same compares and a software popcount. Miri runs the baseline.

**The build** is one sequential pass. Each line's 32 bytes are copied. Then, for each distinct
byte in first-occurrence order, one compare of the line gives the byte's count: its base is
recorded and its counter advanced by that count. **Soundness:** after the last line, every
byte's counter must stand at its bucket's end (`next == end`, all 256 checked). That proves the
counts are `ll`'s own, so the bases tile [0, n) and LF is a permutation of the rows. This is
the tiling argument of `2026-11-10-backward-table.md` §1, at line granularity. A failed check
leaves the table empty and returns false, as `build_one_pass` does.

**The walks** emit the block's bytes last to first:
- **Plain:** from `orig_ptr`, n steps.
  - These end at Ψⁿ(`orig_ptr`). That is `orig_ptr` itself whenever its cycle's length divides
    n, as in every block that real text gives.
  - Otherwise (a crafted or corrupt column), the walk finds the cycle's length and walks again
    from Ψⁿ(`orig_ptr`).
- **Segmented:** `bwt::walk_segments`'s scheme in the other direction.
  - Pseudo-random starts, and `orig_ptr`, are kept in a bitmap. E's line has no spare bit a row
    to flag them.
  - A lane walks LF from a start until the next row is a start. That segment's bytes, reversed,
    are the forward text from the start it ran into up to its own.
  - The output joins the segments from `orig_ptr`'s, each preceded by the segment that ran into
    it. This holds on every cycle of the permutation.
  - **The bitmap walk's shape:** 12 lanes, 256 segments, bytes pushed to a `Vec`.
  - **The flagged walk's shape:** 32 lanes, 512 segments. Each lane's next line is prefetched,
    its bytes go through a raw pointer, and its buffer grows when full.

## 2. Correctness (checked)

- **The bench corpora,** as `ab gen corpora 4` writes them at master `3e1a7ec`, and the
  Wikidata corpus at its pinned SHA-256 `82f65ace…`. Every block's E walks equal `bwt::walk`'s
  byte for byte, and RLE1's output matches every block's CRC (`line_table check`):
  - the plain walk, both segmented shapes (at their own length rule and forced), and the plain
    and flagged walks again on each of the three kernels, each from its own table;
  - 488 blocks and 258.7 M rows in all.

| corpus | blocks | rows | walks compared |
|---|--:|--:|--:|
| wikidata | 431 | 209,706,557 | 4,741 |
| text | 13 | 11,183,609 | 143 |
| records | 11 | 9,600,002 | 121 |
| random | 9 | 8,000,003 | 99 |
| pattern | 22 | 19,600,000 | 242 |
| onebyte | 1 | 392,160 | 11 |
| runs | 1 | 254,432 | 11 |

- **`tests/line_table.rs`** (CI's `cargo test`):
  - every block of `corpora(1)`, the bench corpora's recipe at scale 1, at levels 1 and 9;
  - a fuzz smoke: the fuzz loop's samples compressed at levels 1 and 9, and crafted blocks of
    random columns with any `orig_ptr`;
  - 1,000 mutations of those streams (a bit flipped, a byte set, a span cut). Every block the
    scanner and MTF accept is checked: 31 valid blocks and 644 mutated ones.
- **Unit tests** (`line_table::tests`):
  - random columns over alphabets of 1 to 256 bytes, every kernel against `bwt::walk`, with 1
    to 64 segments, and every kernel building the identical table;
  - each `eq_mask` against a scalar reference;
  - counts that aren't the column's own are refused;
  - the census on a hand-counted line.
- **Miri** (`cargo +nightly miri test --lib -- miri_line_table`, and CI's Miri workflow, now
  triggered by `src/line_table.rs`):
  - `miri_line_table_walks`: the build and every walk's unchecked reads on small columns. These
    have 1 to 256 distinct bytes a line (the overflow list), a partial last line, several cycles,
    and a cycle that doesn't divide n (the plain walk's second pass).
  - `miri_line_table_grows`: the flagged lanes' buffers grown mid-segment, as asserted by a
    test-build counter.
  - Both pass locally (nightly, 2026-11-10). Miri runs the baseline kernel. The AVX2 path differs
    only in `eq_mask_avx2`, one 32-byte load from a `[u8; 32]`.
- **CI's stage harness** rechecks Wikidata on every run: `phases`' E passes run RLE1 and the CRC
  on E's output, block by block.

## 3. The census (counted)

`line_table census`, every block of each corpus. The T-vector serves every step from one
access, a 4-byte entry. E serves a step from one access unless the row's byte is past the
eighth distinct byte of a line with more than nine; such a step reads the overflow list too.
Each row is stepped from once a walk.

| corpus | rows (steps) | distinct bytes a line (mean) | lines with ≤ 9 | E: one access | E: two | E bytes a row | T-vector bytes a row |
|---|--:|--:|--:|--:|--:|--:|--:|
| wikidata | 209,706,557 | 2.26 | 94.0% | 98.18% | 1.82% | 2.039 | 4 (5 with the column) |
| text | 11,183,609 | 4.88 | 85.0% | 96.48% | 3.52% | 2.077 | 4 |
| runs | 254,432 | 3.88 | 83.6% | 95.79% | 4.21% | 2.163 | 4 |
| pattern | 19,600,000 | 1.00 | 100.0% | 100.00% | 0.00% | 2.000 | 4 |
| onebyte | 392,160 | 1.00 | 100.0% | 100.00% | 0.00% | 2.000 | 4 |
| records | 9,600,002 | 30.54 | 0.0% | 27.06% | 72.94% | 4.818 | 4 |
| random | 8,000,003 | 28.42 | 0.0% | 30.32% | 69.68% | 4.553 | 4 |

- **Pathfinder's count holds:** Wikidata's 2.26 distinct a line and 94.0% of lines at most 9
  match `line_census`. The bench's text corpus has fewer distinct bytes than Pathfinder's
  sample (4.88 against 5.12).
- **On text-like data the live table halves:** 2.04–2.16 bytes a row against 4, and 96–98% of
  steps from one line.
- **On incompressible data E is larger than the T-vector,** and seven steps in ten take two
  accesses. A block of that class would keep the T-vector, chosen by the block's run count.

## 4. Instructions (counted)

callgrind, `line_table count FILE 3`: the first 3 blocks of each file. Each kernel is in an
`#[inline(never)]` function, and these are the functions' inclusive instructions a row, on the
AVX2 kernel this machine and CI's runners select.

| | wikidata | text |
|---|--:|--:|
| T-vector build (`bwt::build`) | 4.92 | 11.04 |
| E build | 4.06 (0.82×) | 6.18 (0.56×) |
| room walk (production) | 10.41 | 10.65 |
| bitmap walk on the T-vector (12 lanes) | 34.16 | 34.35 |
| E walk, flagged's shape (32 lanes, prefetch) | 48.21 (4.6× room) | 48.08 (4.5×) |
| E walk, bitmap's shape (12 lanes) | 61.63 (1.8× bitmap) | 61.36 (1.8×) |
| **inversion, E (build + 32-lane walk) over production (build + room walk)** | **3.41×** | **2.50×** |

- **E's step, as compiled,** is 22–26 instructions on its common path, from the disassembly.
  The parts:
  - the line's address;
  - the byte;
  - the broadcast, compare and mask;
  - the slot's popcount;
  - the overflow test;
  - the base's load;
  - the rank's `bzhi` and popcount.

  The flagged shape's lane bookkeeping (pointer, end test, bitmap probe, loop) adds about 22.
  The T-vector's step is a load, a shift and a byte store.
- **A negative finding against the model:** Pathfinder estimated "about 16 instructions" a step
  against the room walk's 10.5. That compared E's step alone with the room walk's whole loop.
  As built, E's step alone is 22–26. In the room walk's own skeleton, with no per-step pointer
  update or end test, E would still be about 26 + 6 = 32 instructions a row, 3× the room walk's
  (model).

## 5. The hypothesis and its predicted effect per corpus

**Hypothesis (Pathfinder's, restated for what CI can see):** E's line table makes a one-thread
inversion cost no more than about twice production's, so that halving the live table can pay
at many workers. The rationale is the i9's knee: four plain tables at 0.7 × L3, eight of E's.

**Predicted, from the census and the counts (model):**

| corpus | timed here? | E build / build | E walk (32 lanes) / room walk | inversion ratio r |
|---|---|--:|--:|--:|
| wikidata | yes: CI's stage harness, 1 thread | 0.4–0.9 | 2.2–3.6 | 2.0–3.2, centre about 2.5 |
| text | no: counted only | (0.56× instructions) | (4.5× instructions) | (2.5× instructions) |
| records, random | no | never selected: E is larger than the T-vector | | |
| runs, pattern, onebyte | no | counted only: 2.0–2.16 bytes a row | | |

- **Why time won't scale as instructions do:**
  - The T-vector's build is a scatter, and costs more time an instruction than E's sequential
    pass.
  - The room walk is bound by load latency, near 2 instructions a cycle on CI's runners.
  - E's walk has more independent work a step, so it can run at a higher rate, 2.5–3.5 an
    instruction cycle. So the time ratios above are below the instruction ratios.
- **On a runner with a 2 MB L2** (CI's Xeon Platinum 8573C), E's 1.8 MB table can sit in L2,
  where the T-vector's 3.6 MB can't. The walk ratio there may be up to about 30% lower than on
  the EPYC 7763 (512 KB L2). The prediction range covers both.

## 6. The CI bench rows that decide it

**The rows.** The stage harness's research table: "Rows not in the total", in the bench
comment's stage section (bench 3.5.0). Wikidata blocks, one thread, pinned. `STAGE_ROUNDS` = 5
runs of the head's `phases` (each stage's fastest of `STAGE_REPEATS` = 2 passes). Each row is
over the stage it stands against in the same run: the geometric mean of the 5 runs' ratios and
its 95% t-interval on their logs (`bench/report.py`'s `ratio_ci`).

| row (its label in `phases`) | against | role |
|---|---|---|
| **line table E: build + walk, 32 lanes** | build + walk | **decides** |
| line table E: build | build | mechanism: the sequential build's premise |
| line table E: walk, 32 lanes, prefetch | walk | mechanism: the step's cost |
| line table E: walk, 12 lanes | walk | mechanism, beside the T-vector's own "walk, bitmap (before #4)" row |

**The decoder's own figures** (`ab run`, head over base, every corpus and thread count) are a
null check, not a decision. The decoder's code is unchanged, so no interval should exclude zero
beyond the chance rate. A mark there is reported, never claimed.

**The verdict, from the decisive row's interval [lo, hi]:**
- **FALSIFIED** (lo > 2.0): E as built is closed. That covers one thread and, by the knee model,
  the ten-worker plans it was for, without i9 timing. **The threshold's reason (model):**
  - At ten workers, production's inversion costs 3–4× its cost alone (the knee entry's E1, as
    `2026-11-10-backward-table.md` §4 cites it).
  - E's ten tables, about 18 MB, are themselves past the i9's knee (0.7 × 20 MB = 14 MB), so E
    keeps part of that contention.
  - With E's own factor at about 1.5 or more, E gains at ten workers only if r is below about
    2–2.7.
- **BEYOND THE MODEL** (hi < 1.0): E is faster than production's inversion at one thread. It is
  reported and not acted on: adoption would need its own registration under STEWARDSHIP's route.
- **SURVIVES THE SCREEN** (otherwise): E is not ruled out. The next step is Pathfinder's gate, a
  registered thread sweep on text. That needs its own registration, and is not run here.
- **INVALID:** the stage job errored, a head stage run recorded an error, or fewer than 4 of the
  5 head runs carry the row. Handled as in "Protocol".

**The mechanism rows' predictions are reported against their ranges (§5), and decide nothing.**
If "E build / build" has hi ≥ 1.0, the premise that the sequential build is cheaper is
falsified on its own.

## 7. STEWARDSHIP's standing acceptance rule (quoted unchanged)

> **Changes that touch measured code** (`src/**`, `Cargo.toml`, `Cargo.lock`, the bench's inputs)
> merge only when all of these hold:
> - the tests and parity checks pass;
> - CI is green on the head that will merge;
> - the CI paired A/B bench (`.github/workflows/bench.yml`) shows an end-to-end improvement
>   beyond noise, meaning its 95% interval excludes zero in the beneficial direction;
> - no other figure regresses unacceptably.

**What it means here:**
- This PR changes no production behaviour, so it can't show an end-to-end improvement, and it
  doesn't qualify for adoption under this rule, whatever the research rows show.
- Whether the research module and the bench's new table merge, as a research and measurement
  change, is the Coordinator's call. The decoder's null check (§6) is the evidence for
  "nothing measured changed that wasn't meant to".
- Any adoption of E into a decode path would be a new change under this rule, or under the
  target-gated route, with its own registration.

## 8. Protocol

- **The decisive observation** is the first complete CI bench run on this PR's head that carries
  this entry and the code it describes.
  - A later push that changes `src/`, `examples/phases.rs` or `bench/` triggers another run.
    That run is disclosed and doesn't replace the first, unless the first was INVALID.
  - A failed job is classified and re-run per `docs/team/CI-INCIDENTS.md`. A re-run of a job that
    failed in setup (class B) is the same observation.
  - No re-run to pass, and none to pick a runner model.
- **The runner model** is recorded with the verdict (CPU model, reported clock, L2 if known).
  The verdict holds for that model. The other model is NOT SHOWN, not inferred.
- **No i9 timing,** nor any workstation timing of E, until the two-modes diagnostic has been
  reviewed:
  - #318's discriminator, and the two-modes design on #313;
  - the review settles whether a short burst on the i9 measures the decoder at all.
  - Only then can an i9 run of E be registered. Under RULES C5 it would be Louis's run.
- **No local timing** was taken for this entry, nor will any be used as evidence. A smoke run of
  `phases` on the 0.25 M-row `runs` corpus checked only that its output parses (§2); its
  figures were discarded unread.
- **After the run:** a result entry, and a line in the queue's "Rejected" list if FALSIFIED.

## 9. Limits

- **CI times one class only,** Wikidata, at one thread. Text, the class E was proposed for, is
  counted here (census, instructions) but not timed. Pathfinder's own falsifier, "the laned
  walk on this table against the room walk, one thread, text", needs a stage harness on text,
  which the bench doesn't run. Adding one is a bench-policy decision, not this PR's.
- **E's walk isn't production's walk:**
  - It uses the flagged walk's skeleton, not the room walk's passes.
  - It keeps a bitmap of starts, since E's lines have no flag bit.
  - These costs are E's as built. §4 gives the model for the room skeleton.
- **The overflow list is `u32`,** not the 3-byte entries the design names. That is 4 bytes for
  each overflowed base, already in the census's bytes a row.

## Reproduce

```text
target/release/examples/ab gen corpora 4                     # the bench's corpora
cargo build --release --example line_table
target/release/examples/line_table check corpora/*.bz2       # §2
target/release/examples/line_table census corpora/*.bz2      # §3
valgrind --tool=callgrind target/release/examples/line_table count corpora/wikidata.bz2 3   # §4
cargo test --lib line_table && cargo test --test line_table  # §2
cargo +nightly miri test --lib -- miri_line_table            # §2
python3 bench/test_report.py                                 # the research table
```

Base: master `3e1a7ec`.
