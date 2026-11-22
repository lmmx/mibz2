# Carbon Cycle 55 (17 / 20) — Multi-Slab Zero-Copy `single_slab_scanner`, `8T` i9 Helper Unlock, and Slab-Ramp Kill

- **Date:** 2026-11-22
- **Branch:** `research/carbon-word-transport`
- **Base Candidate:** Cycle 54 / Interim Handoff 3 (`dd4eb205`)
- **Saved Drop-Ins:** `docs/journal/2026-11-22-carbon-cycle55/{par_scan.rs,parallel.rs}`
- **Safety:** `#![forbid(unsafe_code)]` on `src/par_scan.rs` and `0` new `unsafe` in `src/parallel.rs`

---

## 1. Hypotheses & Root-Cause Discovery

Cycle 55 investigated cold-start I/O chunking (`c55b`) and `2–8T` front-end behavior across `wikidata-l9`, `wikidata-l1`, and `records-l9`, uncovering two concrete front-end bottlenecks:

1. **Bug / Bottleneck 1 (`c55a`, `src/parallel.rs:738`):**
   - On the 10-core i9 at `threads = 8`, `trimmed_here(8)` selects `n = 6` `OpenCells` workers (`leaving 2 cores free`). However, `let helpers = if n <= 6 { 0 } else { 2 };` evaluated `helpers = 0` (`prepare_runs = false`), forcing `8` threads on the i9 to fall back to the serial `single_slab_scanner` with worker-side `runs::front`!
   - **Fix (`c55a`):** `let helpers = if n >= 7 { 2 } else if n >= 6 && threads >= 8 { 1 } else { 0 };` (and `1` helper at `7..=9` threads in `parscan_with`). On the 10-core i9 at `8T`, `6` `OpenCells` workers + `mibz2-scan` + `mibz2-scan-1` = exact `8` compute threads with `prepare_runs = true`, at `0.7946x` peak RSS (`49.63 MiB` vs `62.46 MiB` master).

2. **Bug / Bottleneck 2 (`c55c`, `src/par_scan.rs:518, 772`):**
   - In `SlabRope::view_for_cand`, `fits_zero_copy` required `(last_slab && min_want <= in_slab) || (min_want <= in_slab && next_cand_byte.is_some_and(...))`. Because `single_slab_scanner` (`1..=6T`) passes `next_cand_byte = None`, **whenever a second `4 MiB` slab was already buffered in `rope` (`last_slab == false`), `fits_zero_copy` became `false` for every block in slab 0**, copying `256 KiB` (`INIT_NEED_MORE`) into `stitch` on every block — and on `records-l9` (`~840 KiB` compressed blocks), overrunning twice (`256 KiB -> 512 KiB -> 1,024 KiB`) on every interior block!
   - **Fix (`c55c`):** Return `RopeView::ZeroCopy` whenever `min_want <= in_slab` and `next_cand_byte.is_none()`, and size `single_slab_scanner`'s overrun retry to `.max((140 << 10) * level as usize)` so the single block straddling each `4 MiB` seam stitches once instead of overrunning three times.

3. **Killed Candidate (`c55b`, Progressive `1 MiB -> 2 MiB -> 4 MiB` Reader Slab Ramp in `src/read.rs`):**
   - **One-line kill:** Ramping initial reader slabs `1 MiB -> 2 MiB -> 4 MiB` introduced two extra early slab seams (`1 MiB` and `3 MiB`), raising `records-l9` `2–6T` wall/CPU to `1.0615x–1.0814x` without improving `wikidata-l1` (`0.995x–1.059x`). Reverted to steady `4 MiB` slabs.

---

## 2. Qualification & Same-Binary Push-Streaming A/B (`c55a + c55c` Kept)

- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `tests/par_scan`, and `c41_bench verify` (`1,500`-case streaming mutation fuzz + `1B..1MiB` chunking): **ALL PASS**.
- `i9` Peak RSS (`VmHWM`) at `8T` (with `1` helper scanner + `prepare_runs` enabled): **`49.63 MiB` vs `62.46 MiB` (`0.7946x`, `-12.83 MiB`)**.

### Impact of `c55c` on `2–6T` (`single_slab_scanner`) Across Corpora (`7` Interleaved Rounds)

| Corpus / Threads | Cycle 53 `Cand/MastA` | Cycle 55 `Cand/MastA` | Cycle 55 `Cand/M329` | Cycle 55 `A/A` | Cycle 55 `CPU Ratio` |
| :--- | ---: | ---: | ---: | ---: | ---: |
| `records-l9` `2T` | `1.0190x` | **`0.9981x`** | **`0.9542x`** | `0.9896x` | **`1.0045x`** |
| `records-l9` `4T` | `1.0484x` | **`1.0061x`** | **`0.9910x`** | `1.0149x` | **`1.0180x`** |
| `records-l9` `6T` | `1.0369x` | **`1.0129x`** | **`0.9893x`** | `1.0160x` | **`1.0087x`** |
| `wikidata-l1` `2T` | `1.0045x` | **`0.9992x`** | **`0.9820x`** | `1.0154x` | `1.0460x` |
| `wikidata-l1` `4T` | `1.0269x` | **`1.0012x`** | **`0.9945x`** | `0.9983x` | **`1.0014x`** |
| `wikidata-l9` `10T` | `0.9605x` | **`0.9580x`** | **`0.9613x`** | `0.9780x` | **`0.9866x`** |
| `wikidata-l9` `12T` | `0.9896x` | **`0.9448x`** | **`0.9551x`** | `0.9938x` | **`0.9963x`** |

Next cycle (`Cycle 56`, `18/20`): Screen single-pass `mtf::mtf_checkpoints_scalar` (`src/mtf.rs`) + division-free `runs::cuts` (`src/runs.rs`).
