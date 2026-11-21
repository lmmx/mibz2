# The open kernel's walk in eight lanes, and R4-DGIM at LIMIT 24, natively

*21 November 2026. Research branch `research/open-bp4k`, cut from #668 (merged as 70b18ca4).*

Carbon's two most ambitious remaining Open candidates, ported into the native decoder in safe Rust,
and measured against #668 (Vec16 queues and the 16-byte emit).

## The source

`research/carbon-word-transport` at 58c0f9fa:

- cycle 2 (BVSMC-1P and BVSMC-2P): `docs/journal/2026-11-20-carbon-cycle2-bvsmc.md`, `g5_bvsmc.c`;
- cycle 9 (R4-DGIM): `docs/journal/2026-11-20-carbon-cycle9-r4-dgim.md`, `g5_r4dgim.c`;
- cycle 10 (W24, BP1P): `docs/journal/2026-11-20-carbon-cycle10-w24-bp1p.md`, `g5_w24_bp1p.c`;
- cycle 11 (BP4K): `docs/journal/2026-11-20-carbon-cycle11-flatq-bp4k.md`, `g5_flatq_bp4k.c`.

## Carbon's C claims (Xeon 8481C VM, one thread, 25 rounds, the six admitted blocks)

- BVSMC-2P: 0.699–0.818× the C Open's construct + emit.
- R4-DGIM + BVSMC-2P: 0.580–0.682× the C Open; 0.971–0.996× GIM-RV + BVSMC-2P.
- W24-R4 + BP1P (2 KiB pages): 0.518–0.647× the C Open.
- FlatQ + BP4K (cycle 11, with FlatQ, which #667 found level natively): 0.444–0.573×.

The C Open baseline (`Open-qi16`) predates #668's Vec16 queues and 16-byte emit.

## What was ported

1. **The walk in eight lanes** (`src/open/lanes.rs`, `Kernel::walk_with`).
   - *The cuts:* the retained rows are cut at S rows spread evenly from the marked one. Each cut's
     code is replaced by a sentinel whose `lens` entry is `LONG`, so a segment ends on the 16-byte
     emit's own length lookup, in its cold path.
   - *The lanes:* eight lanes walk eight segments; a lane that ends takes the next.
   - *Placing the bytes:* three ways, chosen by the `WALK` constant.
     - `TwoPass` (BVSMC-2P, S = 64): a lengths pass, a prefix sum along the marked cycle, and a
       writing pass into each segment's slice of `out`.
     - `Pages` (BP4K, S = 48; the shipped choice on this branch): one pass. The marked segment is
       written in place. The others go into 4 KiB pages, taken top-down from dead `out` two pages
       clear of it, then from an overflow past `out[n + 16]`. The pages are moved into place in
       Kahn order, with evictions to the overflow on a cycle.
     - `Pool` (BVSMC-1P): `Pages` with every page from the overflow.
   - *Restoring and bounding:* the cuts are restored before return. A walk not ended in m + 1
     rounds (no permutation) is abandoned to the serial walk. The lanes walk from 4,096 retained
     rows.
2. **R4-DGIM and LIMIT 24** (`src/open/r4.rs`).
   - *The elimination:* on `prev`-only tapes (16-byte top slots, 8-byte bottom slots) with rule
     R4's overwrite. The bottom coordinates go into the surviving top slots, with no `bottom`
     array.
   - *The words:* words of at most 64 bytes live in an arena, joined by copying, and rules past
     that. Block words of at most 16 bytes go straight into `inline16`. The bands are read
     backward, then reversed into `finish`'s order.
   - *The queues:* LIMIT 24, the queue fill a fixed block of 24 rows (U24Q).
   - *Not ported:* the FAR = 8 prefetch, which has no safe-Rust form.
   - *The old elimination:* `endpoint.rs` stays, behind `Kernel::endpoint_lists`, for the arrow
     audit.

No `unsafe` anywhere. `src/open.rs` keeps its no-`unsafe` note.

## Exactness

- **The full suite:** `cargo test --release` passes in full. That includes `open::checks` (whose
  arrow audit now runs every case through both eliminations, at limits to 24), `open::b5` and
  `emit_matches_reference`.
- **New tests in `src/open/lanes_checks.rs`:**
  - *Arbitrary tables (60,000 cases, 150,204 lanes walks):* each walk against the serial walk.
    Permutations give exactly the same bytes; arbitrary maps end and the tables are restored.
  - *Larger tables (400 cases):* across many pages, with evictions (201 evictions, at most 51
    overflow pages).
  - *Small columns (366,735 cases):* every binary column to 10 rows and every ternary column to 6,
    at every row, at limits 0, 1, 4, 16 and 24, by each walk. All match the plain inverse.
  - *Arbitrary and periodic columns:* 240 arbitrary columns (runs of 1 to 10,000 rows, to 900,000
    rows, one cycle or many) and 300 BWTs of periodic texts, by each walk.
- **The admitted blocks:** the six (`MIBZ2_ADMITTED_BWT`, ignored by default) are byte-exact by
  every walk at limits 16 and 24.
- **Mutation checks:** dropping the eviction copy or a pass-1 length fails the page tests.
  Dropping R4's `bots[fb].prev` relink fails the arrow audit.
- **Miri:** not run, as no `unsafe` was added.

## Native stages

**The method:** `docs/journal/2026-11-20-open-stage-attribution/attrib.rs` on
`corpora/wikidata.bz2`, one thread (`taskset -c 2`). Each run takes the best of 5 per block, summed.
There are 8 interleaved runs per build, in rotating order (`stages.sh`, `paired.py`, raw summary
`stages.txt`).

**The builds:**

| build | what |
|---|---|
| master before #668 | ad65b3b8 |
| #668 (baseline) | 70b18ca4 |
| 2P | commit 1 with `WALK = TwoPass` |
| Pages | commit 1 |
| W24+R4 | commit 2 with the serial walk |
| both | commit 2 |
| R4 at 16 | commit 2 with `LIMIT = 16` |
| both, 2P | commit 2 with `TwoPass` |

**Open's time, and against Cells** (ms over the 431 blocks, the range over 8 runs; ratios to #668,
paired by run, with 95% intervals):

| build | Open | front+Open | front+Cells | Open / #668 | front+Open / #668 | Open/Cells, same run | Open, min per block |
|---|--:|--:|--:|--:|--:|--:|--:|
| master before #668 | 372–418 | 403–455 | 356–389 | 1.128 [1.074, 1.184] | 1.107 [1.054, 1.164] | 1.153 [1.137, 1.168] | 351.6 |
| **#668** | 326–357 | 360–394 | 371–403 | 1 | 1 | 0.970 [0.961, 0.979] | 303.6 |
| 2P | 323–366 | 353–443 | 346–429 | 1.016 [0.959, 1.077] | 1.022 [0.942, 1.109] | 1.036 [1.025, 1.048] | 315.2 |
| Pages | 286–347 | 315–385 | 338–400 | 0.917 [0.856, 0.981] | 0.917 [0.854, 0.985] | 0.939 [0.927, 0.951] | 282.6 |
| W24+R4, serial walk | 290–343 | 320–379 | 345–399 | 0.925 [0.885, 0.968] | 0.926 [0.884, 0.969] | 0.938 [0.929, 0.946] | 281.5 |
| **both** | 270–329 | 300–368 | 346–408 | **0.872 [0.820, 0.928]** | **0.879 [0.825, 0.936]** | **0.882 [0.872, 0.892]** | **266.6** |
| R4 at 16 + Pages | 279–317 | 310–381 | 354–418 | 0.882 [0.852, 0.913] | 0.903 [0.853, 0.956] | 0.885 [0.876, 0.895] | 270.3 |
| both, 2P | 323–374 | 371–413 | 366–410 | 1.034 [0.988, 1.081] | 1.041 [1.001, 1.083] | 1.008 [0.996, 1.021] | 311.0 |

Front+Open against front+Cells in the same run is 0.973 for #668 and 0.893 for both.

**By block** (the median of the per-block ratio to #668, best over the 8 runs, blocks of at least
100,000 rows, by run density r/n):

| r/n | Pages | R4 at 16 + Pages | both |
|---|--:|--:|--:|
| 0.059–0.064 | 0.963 | 0.919 | 0.906 |
| 0.064–0.066 | 0.955 | 0.902 | 0.883 |
| 0.066–0.071 | 0.943 | 0.887 | 0.876 |
| 0.071–0.083 | 0.932 | 0.887 | 0.885 |
| 0.083–0.206 | 0.886 | 0.851 | 0.852 |

**Hot, one process** (`lanes_timing`, the six admitted blocks, best of 31, a 32 MiB sweep between
runs). The construction is 4.5–4.8 ms at LIMIT 16 with the lists. Its parts:

- the elimination, about 1.0 ms;
- the bands and words, about 1.1 ms;
- the tapes' setup, about 0.3 ms.

R4 cuts the construction by about 11% at 16 and 17% at 24. The walk is 4.9 ms serial, 5.1 ms in
two passes, 4.2 ms in pages (0.85×) and 4.6 ms in the pool.

**What the stages show:**

- **Two passes lose natively.** The second walk costs more than the eight lanes save. (Carbon's VM
  had a 2 MiB L2; this container has 8 MiB.)
- **The one-pass pages pay**, by more on dense blocks, as Carbon's Amdahl note predicts.
- **R4's elimination and inline words pay independently** of the walk.
- **LIMIT 24 adds about 1% over R4 at 16.** Cycle 10's co-equilibration is real, but small natively.

## The whole decoder

**The method:** `examples/ab.rs`, `--spread`, 10 rounds, on the shared 4-vCPU container (indicative
only; `ab-summary.md`, `ab.jsonl`). The ratios are wall time against `mibz2-workers-lazy` at #668.

| arm | 1 thread | 2 threads | 4 threads |
|---|--:|--:|--:|
| `mibz2-workers-lazy`, head | 0.914 [0.856, 0.975] | 0.924 [0.860, 0.993] | 0.972 [0.907, 1.043] |
| `mibz2`, head (control) | 0.875 [0.808, 0.948] | 0.952 [0.897, 1.011] | 0.993 [0.940, 1.049] |
| `mibz2`, #668 (control) | 0.922 [0.860, 0.989] | 0.934 [0.879, 0.992] | 1.042 [0.946, 1.148] |

**What the controls say:** `mibz2` runs Cells below 8 workers, where nothing changed. Its head and
#668 arms differ by up to 5% at 1 thread, which is this machine's noise. The `-lazy` arm's 1- and
2-thread gains are about the stage figures. At 4 threads its interval includes 1.

## Memory

**Peak RSS, `-lazy` at 1/2/4 threads:** head 38/51/75 MB; #668 37/49/75 MB.

**`out`, per worker:** it is reserved at n + 16 + 256 KiB (1,134 KiB for a 900 kB block, against
n + 16). The overflow the pages took on the admitted blocks was 184–216 KiB at LIMIT 16 and
188–204 KiB at LIMIT 24 (46–54 pages), with 58–85 evictions. A block needing more than 64 overflow
pages grows `out` by doubling. Kahn's arrays and the lanes' state are a few KiB.

**The kernel's own scratch** (b5's worst case a row): 15.91 bytes a row against #668's 16.29. The
tapes and arena replace the lists, `bottom` and the short words' spelling.

At 20 workers the `out` reserve is about +5 MiB.

## Not ported, and why

- **The FAR = 8 prefetch** (R4-DGIM): it has no safe form, and the tapes are L2-resident.
- **FlatQ and the primary split** (cycle 11's producer half): #667 found them level with Vec16
  natively.
- **2 KiB pages** (BP1P): superseded by 4 KiB (BP4K), as in cycle 11.
- **No unchecked stores:** every 16-byte store is a bounds-checked slice copy, and no unsafe
  variant was needed for this comparison.

## Decision rule

Adoption needs an end-to-end gain on a path production runs, the i9 at 6 threads or more, where
Open runs. This is a clear native kernel win: Open at 0.872× of #668, and Open/Cells at 0.882 against
0.970. So by the rule it goes to an i9 recipe, as #669 did for Vec16. Nothing here changes the
1–4-thread default arm, which never runs Open.
