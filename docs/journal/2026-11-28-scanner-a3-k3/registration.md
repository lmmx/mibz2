# Three-symbol A3 scanner decode: i9-10900K adoption run against its master parent (registration)

*Frozen 28 November 2026 before any workstation data, under claim [`scanner-a3-k3-pipe-u8`](../../../frontier/claims/scanner-a3-k3-pipe-u8.toml). This is an adoption registration (RULES C6): it decides whether the change merges into the default decoder, and it needs one review by an agent other than its author before its ready tag is set. C2 is never relaxed after the data is seen.*

## 1. What is compared

- **Head:** `ad99d407e720d413f5795de263ec96656e08bcb2`. This is the change's single commit, touching `src/block.rs`, `src/huff.rs` and `src/bits.rs`. It adds three-symbol root entries with a gated triple pass, byte staging with a symbol-256 position list, and two batch shapes: a pipelined refill under the fourth lookup on tables that can hold pairs, and five lookups then a refill on single-symbol tables.
- **Base:** `07ab2e7e4cb0b037933ba5e1a1177b8fa353dd37`. This is the head's only parent and was master when the change was made. It already includes the fast A2 setup (#1117), so the comparison prices A3 on top of A2, as `frontier/DECODER.md` requires.
- **Checks before running:** the recipe fetches both commits into a private repository and refuses to run unless base is head's only parent. The pinned `examples/phases.rs` (blob `403cfbbd…`) is the same in both arms, as for #1034's and #1117's runs.
- **Where things live:** the head is kept on origin as `build/scanner-a3-k3-head`. The recipe (`bench/scanner_a3_k3.py`, `just bench-scanner-a3-k3`), its test and this file live on the reviewed PR branch / ready tag until merged, and neither measured tree contains them.

## 2. Correctness (VOID if any fails)

Gate 1 of the claim's stop test passed before this registration (delta `2026-11-28-scanner-a3-k3-pipe-u8-2.md`):
- No new `unsafe`. The one `unsafe` touched is the existing BMI2 dispatch, which gains a `patches` argument under the same safety argument.
- `cargo test --release` passes, including libbzip2 parity. One new test asserts that every fast LUT entry equals the greedy bit-by-bit decode of its 10 bits, up to 1, 2 or 3 symbols as built. Another asserts that `decode_runs` equals libbzip2's bit-by-bit loop in result, end bit and symbols over 500 random tables and streams. The K≤2 build stays bit-identical to the two-pass reference.
- Miri is clean on `miri_decode_runs_scalar_exact` and `miri_selectors_and_lengths_read_as_reference`.
- Fuzz-smoke is clean (classic, pipeline, service).
- The scanner digests (every block's symbols and CRC) are identical to base on wikidata and all six guard corpora.

On the workstation:
- every corpus decodes in both arms at 1 and 10 workers, with its SHA-256 equal to libbz2's;
- every timed sample checks its size and scanned block count;
- the scanner runs on its pinned CPU with BMI2 active;
- both binaries' `decode_runs_bmi2` contain `shlx`/`shrx` and no `vzeroupper`;
- ambient `RUSTFLAGS` / `CARGO_BUILD_RUSTFLAGS` / `CARGO_ENCODED_RUSTFLAGS` / `CARGO_PROFILE_RELEASE_*` overrides are absent and `base`/`head` binaries have distinct SHA-256 digests.

Any of these failing makes the run VOID, not STOP.

## 3. Grid

The grid is #1117's, with `pattern` added because the claim names it as a guard:
- **Machine:** Rust 1.99.1, i9-10900K, unrestricted 10C/20T topology. The scanner runs on one physical core whose sibling is excluded from the workers' mask.
- **Rounds:** ten paired rounds per cell, five head-first and five base-first in a shuffled order (seed 1153). Every sample is a fresh process.
- **Size:** 17 cells, 340 processes.
- **Serial scan S' (A2 + A3):** one pinned `phases::pass` per sample, on `wikidata ×4` and on the six guard corpora (`ab gen` `text`, `records`, `random`, `runs`, `pattern`, `onebyte`).
- **Real decode wall and peak RSS:** `wikidata ×4` at 1, 8, 9 and 10 workers, and each guard corpus at 8 workers.

## 4. Decision (frozen)

**PASS** only if all of these hold, otherwise **STOP**:

1. **S' cut ≥ 4.0%** on `wikidata ×4`. The upper 95% bound of the paired head/base ratio must be ≤ 0.960, *and* the head's slowest sample must be faster than the base's fastest (disjoint ranges). This is the claim's gate 2 verbatim.
2. **Guards:** no guard corpus's serial scan may be more than 2.0% slower beyond noise. A guard fails if its paired ratio is above 1.020 *and* its lower 95% bound is above 1. This is #1117's reading of "no > 2.0% regression": `runs`, `pattern` and `onebyte` scan in under a millisecond, where a point estimate alone would stop on noise.
3. **Memory:** calibrated paired-GM peak-RSS gate (`docs/notebook/2026-11-28-rss-bound-calibration.md` §5): at `k = 1`, fail if paired GM > 1.005× *and* median delta > +256 KiB *and* 95% CI lower bound > 1; at `k ≥ 4`, fail if paired GM > 1.030× on `wikidata` or > 1.050× on guard corpora with 95% CI lower bound > 1. `ScanScratch` grows by about 2.5 KiB: 1.5 KiB of per-table code lengths and a 2 KiB cold patch list, less 1 KiB of chunk.
4. **No whole-decoder loss on Wikidata:** at no real `wikidata ×4` count may the wall ratio be above 1.020 with its lower 95% bound above 1. This is #1117's rule 4: a scanner gain must not ship through a whole-decoder loss.

Real-decode ratios are reported in full at every cell. Apart from rule 4 they are diagnostic, not gates. They show where the cut pays at the knee: at 8 workers it is scanner-bound with the bounded lanes, and at 9–10 workers in the default build.

**On PASS:** the change merges under its exact-head gates. This run is its adoption evidence and stands in for PERFORMANCE-POLICY.md §2 rule 1, which CI's two cores cannot measure for a scanner cut while workers bind. **On STOP:** the change is not merged, and a scoped row goes in `frontier/dead-ends.tsv` naming this registration.

## 5. What the VM showed (direction only, C5; not evidence for this gate)

These are paired 1-thread `Scanner` passes on a 4-vCPU KVM, pinned, taking the minimum of 3–7 passes per process:
- **Wikidata:** −4% to −7.5% (decode loop −11.5%, triple pass +1.2 ms of build).
- **Text:** −8% to −9%.
- **Runs:** −4% to −8%.
- **Records and random:** within noise. An A/A run of base against itself on this VM gave 0.956 and 1.039 with ±5% intervals there. Cachegrind counts on records and random are 6.5% and 5.2% fewer instructions than base.
