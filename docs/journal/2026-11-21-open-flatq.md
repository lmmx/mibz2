# Open FlatQ natively: level with Vec16, stopped before BP4K

*21 November 2026. Branch `research/open-flatq`, from #666's head
(`research/open-lrb-vec16`, ef248f78) with master merged. Source: Carbon cycle 11,
`research/carbon-word-transport` at 12dcb9f2, `docs/journal/2026-11-20-carbon-cycle11-flatq-bp4k.md`
and `g5_flatq_bp4k.c`.*

**The finding.** Carbon's FlatQ and primary split are exact in safe Rust, but natively they
are level with #666's Vec16 queue: the open kernel on the corpus at one thread moves by
0.990× [0.949, 1.033] (stage 1+2 / Vec16, 12 paired rounds), inside the noise. The C
harness's 0.374–0.468× construct and 0.444–0.573× total did not appear: what FlatQ removes
(the per-byte `Vec`s' growth and length) is what Vec16 already removed. BP4K was therefore
not ported (the brief's rule: only if stages 1 and 2 win natively).

## The C claims

- FlatQ-BP4K: 0.444–0.573× shipped Open's total; 0.554–0.680× Open+U16Q's;
- construct: 0.374–0.468× Open's.
- The C harness's Open baseline is calibrated to native Open within a few percent
  (`2026-11-20-open-u16q.md`), but the LRB fusion won in C and lost natively (#666).

## The port

All in `src/open.rs`, no `unsafe`.

1. **FlatQ** (`construct_slab`). One slab for every byte's queue, partitioned before the pass
   from the counts `runs::front` already makes (`Runs::meta`: each byte's rows `counts` and
   records `per`). Byte `a` gets `min(counts[a], max(limit, 1) · per[a]) + 15` slots; one
   absolute cursor a byte. A thin run moves only its cursor; a byte's rows so far, which a
   thick strip needs, are rebuilt as retained rows + thick rows (Carbon's lazy `all_c`).
   - **The bound, proved.** A byte's retained rows are rows of it (≤ `counts`). They are its
     thin maximal runs' rows (≤ `limit` each) plus the marked row if it falls in a thick run.
     Each maximal run holds at least one record, so there are ≤ `per` maximal runs, and with
     the marked row's thick run among them ≤ `per − 1` thin ones:
     `limit·(per − 1) + 1 ≤ max(limit, 1)·per`. A store writes 16 slots from the cursor for a
     run of ≤ 16 rows (else the run's rows), so a byte's stores end ≤ 15 past its last retained
     row. This is tighter than Carbon's `min(counts, 24·per) + 36` (its `+1` is absorbed).
   - **Checked, not trusted.** The counts must sum to `n`. Stores are `get_mut`, so none
     falls past the slab. After the pass each byte's retained rows + 15 must be within its
     room. Otherwise the block is built again with the `Vec` queues, so wrong counts give a
     slower block, never a wrong one.
   - The fused route (#666's LRB) has no counts before its pass and keeps the `Vec`s.
2. **Primary split** (`Pass::scan`). The runs before the marked row are each tested only for
   whether they end past `p`. The run holding it is taken with the mark looked for. The rest
   have no test of `p`.
3. **The fused route off the decoder** (`FUSED_ROUTE` in `src/lib.rs`), so that the decoder
   runs the records' path, where the slab is. The route and its tests stay.

## Exactness

- `cargo test --release`: the full suite passes, including `open::checks`, `open::b5` and
  #666's `open::fused`. `checks.rs`'s arrow audit now also audits the slab path, at every
  column, mark and limit it covers. The decoder's stream tests now run the slab, since the
  fused route is off.
- New, `src/open/slab.rs`:
  - **Slab against `Vec` on arbitrary columns:** 14,400 cases, with 1–70,000 rows, alphabets
    of 1–256, runs averaging 1–400, four rows each, and limits 0–1000. Edges, codes, words,
    mark and bits are identical, and the slab takes every block.
  - **The bound met with equality:** columns built to drive a byte's retained rows to the
    bound. These are thin runs of exactly `limit` (`limit·per = counts`), the marked row in
    the byte's thick run, single rows (`counts = per`), and limit 0. No store passes its
    room, and over 100 bytes meet it exactly.
  - **Wrong counts:** `per` understated, rows understated, rows moved between bytes, and a
    total off by one. The slab refuses where its partition fails, and the block is still
    exact.
  - **The six admitted blocks** (ignored by default, `MIBZ2_ADMITTED_BWT`): the slab takes
    each, its table is the `Vec`s', and the output is the plain inverse's.

## Stages, one thread (ms)

`attrib.rs` from `2026-11-20-open-stage-attribution/` (master's, with no fused column), on
`corpora/wikidata.bz2` (431 blocks, 209.7 M rows). Pinned to CPU 2, best of 5 passes per block,
summed. Each build ran 12 times, interleaved. The ranges are over the runs, and "min" is the
best per block over all 60 passes, summed.

| build | front | Cells | Open | front + Open | front + Cells | Open, min | front + Open, min |
|---|--:|--:|--:|--:|--:|--:|--:|
| master 4ab98cd7 | 31.6–36.7 | 322–360 | 372–425 | 403–462 | 354–397 | 353.8 | 382.9 |
| #666 Vec16, fusion off (9de3eec1) | 32.7–45.4 | 318–370 | 303–359 | 336–399 | 351–410 | 297.4 | 328.9 |
| stage 1, FlatQ (1846bc80) | 34.4–41.9 | 325–364 | 318–360 | 353–401 | 360–405 | 300.6 | 332.4 |
| stage 1+2, + primary split (3e5eee62) | 34.0–40.0 | 330–363 | 311–354 | 345–394 | 364–402 | 294.9 | 326.0 |

Paired by run (geometric mean, 95% CI over 12 runs):

| ratio | Open | front + Open | Open / Cells |
|---|--:|--:|--:|
| master / Vec16 | 1.185 [1.129, 1.243] | 1.155 [1.100, 1.213] | 1.196 [1.183, 1.208] |
| stage 1 / Vec16 | 0.994 [0.955, 1.034] | 0.992 [0.952, 1.034] | 1.009 [0.998, 1.019] |
| stage 1+2 / Vec16 | 0.990 [0.949, 1.033] | 0.988 [0.946, 1.032] | 0.994 [0.983, 1.005] |
| stage 1+2 / stage 1 | 0.996 [0.961, 1.033] | 0.996 [0.960, 1.034] | 0.986 [0.974, 0.997] |

- **Front + Open against front + Cells, in the same run:** master 1.152, Vec16 0.979
  [0.972, 0.986], stage 1 0.986, stage 1+2 0.973 [0.966, 0.981]. Front + Open stays below
  front + Cells by 2–3%, as on #666. FlatQ doesn't widen the gap.
- **Hot, in one process** (`slab_against_vec_timing`, ignored). Each admitted block was run
  best of 41, interleaved, with the slab and the `Vec` queues on the same build (both with
  the lazy rows and the split). Slab / `Vec` was 0.951–0.992. The slab's own gain is real
  but small: 1–5% of the kernel hot, and below the noise in decode order.
- **Additive or overlapping?** Overlapping. Vec16 took master's Open from 1.185× to 1; FlatQ
  adds nothing measurable on top. Both remove the same cost: the per-run work of a growable
  queue. Vec16 removes its length write-back and loop exit. FlatQ removes its capacity
  check and the 256 headers. What is left (one bounds check, two 32-byte stores, the
  cursor) is common to both.

## Whole decoder (indicative)

`examples/ab.rs`, `--spread`, 10 rounds, on this shared 4-vCPU container, `corpora/wikidata.bz2`.
Ratios are wall time against `mibz2-workers-lazy` on master, paired by round.

| arm | 1 thread | 2 threads | 4 threads |
|---|--:|--:|--:|
| `mibz2-workers-lazy`, head | 1.017 [0.915, 1.130] | 0.962 [0.897, 1.032] | 0.987 [0.922, 1.056] |
| `mibz2-workers-lazy`, #666 with the fused route off | 1.018 [0.952, 1.088] | 0.957 [0.891, 1.028] | 1.050 [0.988, 1.115] |
| `mibz2`, head | 0.943 [0.885, 1.005] | 0.963 [0.907, 1.023] | 1.029 [0.982, 1.079] |
| `mibz2`, master | 0.975 [0.918, 1.035] | 0.938 [0.870, 1.012] | 0.993 [0.953, 1.034] |

- Every interval includes 1. The one exception is the head's `-lazy` CPU at 4 threads,
  0.945 [0.896, 0.997].
- `mibz2` at 1–4 threads never runs Open (CI's default arm doesn't either), so its row is a
  control.

## Memory

- **The slab's size a worker** (its kernel's, kept from block to block, the largest block's):
  1,569–2,698 KiB reserved on the six admitted blocks. Of that, 351–798 KiB is written per
  block (the retained rows).
- **Against the `Vec` queues:** after the six blocks in turn, the `Vec`s keep 1,556 KiB and
  the slab 2,698 KiB, so +1.1 MiB a kernel. Across blocks the bytes' rooms move, so the
  slab's pages are all touched in time.
- **Peak RSS** (ab, `-lazy`): head 40 / 60 / 91 MB at 1 / 2 / 4 threads; master 37 / 50 / 73;
  #666 with the fused route off 37 / 49 / 79. At 20 workers, +1.1 MiB a kernel is +22 MiB or
  more.

## What didn't port

- **BP4K** (bipolar 4 KiB page splicing with a Kahn order of pages, the 8-lane emit): not
  attempted. Stages 1 and 2 did not win natively.
- **Carbon's `R4-DGIM` at `LIMIT = 24`:** not ported. The limit stays 16, the arm the i9
  measured.
- **FlatQ on the fused route:** that route has no counts before its pass. It keeps the
  `Vec` queues, and it is off on this branch.

## Decision rule (unchanged from #666)

Adoption needs an end-to-end gain on a path production runs:
- Open at 8 or more workers (the i9), or
- a routing change backed by front + Open below front + Cells natively.

Neither is shown beyond Vec16: FlatQ is level with it and costs about 1.1 MiB a kernel. CI's
default arm at 1–4 threads never runs Open.
