# Room walk, constant-index 32-lane round: full-decoder trial protocol (DRAFT for review)

**Status:**
- A draft for the maintainer's review. Nothing is implemented, built for timing or timed.
- On approval, with any changes, it is committed as registered before the harness exists.
- After that, only an amendment committed before any timing can change it.

**Independence:**
- The branch is `research/walk-unroll` (master `45b366d` plus the arm). It contains no
  P74-C code.
- `Kernels`, `Plan`, the MTF kernels and the frozen P74-C trial are untouched.
- Combining this with P74-C is out of scope until both decoder-level effects are known.

**Prior evidence:**
- **Measured:** CI run 41222735026. One copy, complete room walk, B/A 0.877
  [0.863, 0.891] on the counted EPYC 7763 job; Zen 4 replication 0.869
  [0.851, 0.888]; r1 void by its A/A rule.
- **Measured, with every CPU busy (SMT siblings):** 0.92–0.95 on Zen 3 and 0.97–0.99 on
  Zen 4.
- **Registered history:** the structural audit was STRUCTURAL STOP as written (S3, S5).
  Amendment 1 was the gate set the timing ran under.

## 1. How the walk is selected in production (checked on master `45b366d`)

| Path | Inverse kernel | Walk |
|---|---|---|
| `decode`, `decode_with::<Segmented>`, `BzReader` single-threaded | `bwt::Segmented` | `walk_segments_room` → `walk_segments_room_parts` → `room_segments::<32, PF_L1>` (blocks ≥ `MIN_SEGMENTED` = 65,536 rows; the plain `walk` below that) |
| `StreamDecoder::new` → `Plan::for_threads` → `Plan::Workers(n)` (fewer than `COOP_MIN_CORES` = 6 physical cores, so 1–4 workers on 4 cores) | `planned` → `with_inverse::<Segmented>(n, slots, k)` | as above |
| `Plan::Teams { k: 2..4 }` (6 or more physical cores) | `Cooperative<K>` | `coop::walk_room` for blocks ≥ `MIN_COOP`; `walk_segments_room` for smaller blocks |

**How MTF is chosen:**
- `decode_block` picks `mtf::mtf` (plain) when `wants_checkpoints()` is false (the
  default, and so `Segmented`'s), and `mtf::mtf_checkpoints` otherwise.
- So a kernel type with the default `wants_checkpoints` runs the shipped plain MTF,
  exactly as `Segmented` does.

## 2. What is under test (the integration; not yet written)

- **The kernel:** a research-only `bwt::SegmentedUnrolled` (doc-hidden).
  - It is `Segmented` with one change: its walk calls the arm (`room_segments_unrolled`
    through a `walk_segments_room_unrolled` with the same `MIN_SEGMENTED` fallback and
    assembly).
  - It keeps the default `wants_checkpoints` (false), the same `TVector`, the same
    `SegScratch` and the same buffers.
- **How it is selected:** only by `StreamDecoder::with_inverse::<SegmentedUnrolled>(n,
  slots, k)`, the call `Plan::Workers(n)` makes with `Segmented`.
  - No `Kernels` field, `Plan` variant, environment variable or per-block branch is
    added.
  - `new`, `planned`, `decode`, `BzReader`, `decode_parallel`, `Cooperative` and `coop`
    are unchanged, so every production default stays `Segmented`.
- **Out of scope:**
  - the cooperative walk (`coop::walk_room`, team plans);
  - the single-threaded `decode` path. Its walk would change the same way, but it isn't
    a configuration under test.

## 3. Arms (one binary)

| Arm | Decoder |
|---|---|
| **A** | `with_inverse::<Segmented>(w, 2w, Kernels::detect())`, production's `Plan::Workers(w)` decoder |
| **B** | `with_inverse::<SegmentedUnrolled>(w, 2w, Kernels::detect())` |
| **A′** | A again: the A/A null |

**The arms differ only in the inverse-kernel type.**
- A and B are separate monomorphisations of the worker and `decode_block`, so their glue
  code is laid out separately.
- §9 W4 checks that those instantiations differ only in the inverse call, and that the
  MTF, build, RLE1 and CRC symbols they call are the same functions.

## 4. CPU allocation and affinity: P74-C decoder trial, harness v1.1 (unchanged design)

- **Placement:** w workers plus the scanner, the feeder and main, on w ≤ 4 physical cores
  with their SMT siblings idle, derived from sysfs.
- **Inside each fresh child:**
  - `Plan::for_threads(w) == Workers(w)` is asserted;
  - the thread inventory is w + 3;
  - every thread's `Cpus_allowed_list` is within the allocation, at spawn and at the
    end;
  - `--check` waits for the previous decode's threads to leave (v1.1).
- **On the i9:** w ∈ {1, 2, 3, 4} on four physical cores.
- **On CI** (exploratory): w ∈ {1, 2} on the runner's two physical cores.

## 5. Corpora (to be frozen with hashes before any timing)

1. `wikidata-20261026`, different-dump held-out (primary): the P74-C trial's frozen
   windows (`cf13f439…`).
2. `wikidata-fitted` (`82f65ace…`) and `wikidata-heldout` (`7f5c9a00…`): secondary.
3. **Controls:** `random`, `records`, `mixed-independent` and `text`, as frozen for P74-C.
   - `text` is built from the P74-C v1.0 tree (`535296c`), so it is byte-identical to that
     trial's corpus (`59236f2d…`).
4. **Walk-structure controls (new, frozen with their hashes before timing):**
   - **`text-l1`:** `text` at bzip2 -1. The blocks are smaller, but there are still 512
     segments a block, so segments are about 9× shorter. That puts more mid-pass takes
     and boundary handling on the arm's duplicated cold paths.
   - **`text-l5`:** `text` at bzip2 -5.
   - **`zeros`:** one symbol, so the segments are degenerate.
   - **`period-3`:** `abc` repeated, short cycles.

   The last two are small (a few blocks each), and are timed for the arm's boundary and
   I-cache behaviour.

## 6. Schedule

The P74-C trial's schedule:
- 20 paired rounds;
- the 6 permutations of (A, B, A′) three times, then two more, with the corpus order
  rotated by a registered seed;
- a warm-up round, discarded;
- every measurement a fresh child process;
- correctness (every output's SHA-256) checked untimed on every child.

## 7. Statistics

These are the same as the P74-C decoder trial's §7:
- per-round log ratios with the ρ₁ correction;
- two-sided 95% t intervals, read as one-sided 2.5% bounds;
- intersection–union for conjunctive gates, with no Holm adjustment;
- the RSS max rule;
- A/A calibration by k* on m rows.

**Figures:**
- throughput (MB/s of output) and its B/A;
- CPU-s/GB and its B/A;
- peak RSS, by the max rule and the paired figures;
- MB/s and ns/B of output per arm.

## 8. Acceptance (three outcomes per gate; NEGATIVE takes precedence)

These are the P74-C decoder trial's gates, with the same thresholds:

0. **Correctness:** any B output that differs from A's or from the frozen hash is
   NEGATIVE overall. An input, audit or plan failure aborts the run.
1. **Primary:** B/A throughput at w = 4 on `wikidata-20261026`.
   - **PASS:** lower ≥ 1.02.
   - **NEGATIVE:** upper < 1.02.
   - **INCONCLUSIVE:** otherwise.
2. **Secondary Wikidata at w = 4:** a material-deterioration flag if the point < 1.00 or
   lower < 0.97.
3. **Controls at every w** (§5 items 3 and 4):
   - **PASS:** lower ≥ 0.97.
   - **NEGATIVE:** upper < 0.97.
   - **INCONCLUSIVE:** otherwise.
4. **CPU-s/GB at w = 4, on every corpus:**
   - **PASS:** point ≤ 1.00 and upper ≤ 1.01.
   - **NEGATIVE:** lower > 1.01.
   - **INCONCLUSIVE:** otherwise.
5. **Peak RSS, on every corpus at every w:**
   - **PASS:** M_B ≤ 1.01 · M_A.
   - **NEGATIVE:** the paired lower bound > 1.01.
   - **INCONCLUSIVE:** otherwise.
6. **A/A calibration:** PASS unless miscalibrated.

**The overall verdict and stop rule** are as for P74-C:
- **PASS** proposes a default-change PR for approval. Nothing is enabled by this trial.
- **NEGATIVE or INCONCLUSIVE** archives the arm at decoder level. There is no retuning.

**The deciding run:** the i9-10900K, after the P74-C i9 trial and when the maintainer
authorises it.

**Prediction (model, stated before measurement):**
- The walk saving of 0.43–0.77 c/B (measured, one copy) against a single-worker decoder
  of about 10.8–13 c/B suggests about +3% to +6% throughput, with workers on separate
  physical cores.
- It shrinks where the scanner or memory bandwidth bounds the run. With SMT siblings
  busy it is about +1% to +3%.
- So the primary can come out PASS or INCONCLUSIVE.

## 9. Structural audit (on the exact timed binary, before timing; any failure aborts)

| Gate | Condition |
|---|---|
| W1 | the walk-unroll amendment-1 gates (`bench/walk_unroll_gates.py`) on `room_segments_unrolled::<32, 1>` in this binary: 32 cores, ≤ 11.0 instructions, S3′, S4, S5′ |
| W2 | `room_segments::<32, 1>` (shipped) is present, and its hot step is unchanged: 16 instructions, 3 loads, 2 stores, 1 prefetch |
| W3 | B's worker reaches `room_segments_unrolled` and never `room_segments::<32, 1>` on segmented blocks; A's the reverse. Shown by each instantiation's call graph. |
| W4 | `decode_block::<Segmented>` and `decode_block::<SegmentedUnrolled>` (or the worker loops they are inlined into) call the same MTF, build, RLE1 and CRC symbols. Normalised for addresses, their machine code differs only in the inverse calls; any other difference is reported and stops the run. |

**The registered history stays as it is:** the arm's structural STRUCTURAL STOP as
written, and amendment 1's gates and their rationale.

## 10. What each host can show

- **The i9 (4 physical cores, siblings idle):** decides. Workers there don't share cores,
  which is the one-copy condition the stage gain was measured under.
- **CI (2 physical cores):** exploratory only, at w = 1 and 2. A worker can share a core
  with the scanner or feeder, so results there are expected to be smaller.
