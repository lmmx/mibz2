# Fast A2 scanner setup: i9-10900K adoption run against its merge base (registration)

*Frozen 27 November 2026 before any workstation data, under claims [`scanner-a2-fast-build`](../../../frontier/claims/scanner-a2-fast-build.toml) and [`scanner-a2-fast-i9-run`](../../../frontier/claims/scanner-a2-fast-i9-run.toml). This is an adoption registration (RULES C6): it decides whether PR #1117 merges into the default decoder, and needs one review by an agent other than its author before its ready tag is set. C2 is never relaxed after the data is seen.*

## 1. What is compared

- **Head:** `689e55f7e4307167fa3fd0c744e16a3f85528490`, PR #1117 (direct selector runs, multi-length code-length LUT, single-write table build in `src/block.rs` and `src/huff.rs`) merged with master.
- **Base:** `2af67a675b39d9523e96686c98c72c984f799868`, that merge's second parent, the master commit it was merged onto. `git diff base head` is exactly #1117's change.
- The recipe fetches both into a private repository and refuses to run unless head's second parent is base. The pinned `examples/phases.rs` (blob `403cfbbd…`) is the same in both, as for #1034's run.
- The recipe, its test and this file live on master (branch `bench/scanner-a2-fast`); the head is kept on origin as `build/scanner-a2-fast-head`. Neither measured tree contains them.

## 2. Correctness (VOID if any fails)

Gate 1 of the claim's stop test passed before this registration (delta `2026-11-27-scanner-a2-fast-build.md`; independent review noted on the PR): 100% safe Rust in the changed code, LUTs bit-identical to the two-pass reference over 12,000 builds, selector and length readers identical in result, error and end bit on 20,000 cases, `cargo test --release` with libbzip2 parity, Miri and fuzz-smoke clean. On the workstation, every corpus decodes in both arms at 1 and 10 workers with its SHA-256 equal to libbz2's, every timed sample checks size and scanned block count, the scanner runs on its pinned CPU with BMI2 active, and both binaries' `decode_runs_bmi2` contain `shlx`/`shrx` and no `vzeroupper`. Any failure is VOID, not STOP.

## 3. Grid

Rust 1.99.1, i9-10900K, unrestricted 10C/20T topology, scanner on one physical core whose sibling is excluded from the workers' mask (#1034's placement). Ten paired rounds per cell, five head-first and five base-first in a shuffled order (seed 1117), every sample a fresh process. 15 cells, 300 processes:

- **Serial scan S' (A2 + A3):** one pinned `phases::pass` per sample, on `wikidata ×4` (`0b820d14…ce08e`) and on the five guard corpora (`ab gen` `text`, `records`, `random`, `runs`, `onebyte`).
- **Real decode wall and peak RSS:** `wikidata ×4` at 1, 8, 9 and 10 workers (k = 8 and 9 because the lever's value is conditional on the knee); each guard corpus at 8 workers.

Estimated time: about 5–10 minutes with cached inputs, plus four builds.

## 4. Decision (frozen)

**PASS** only if all hold, otherwise **STOP**:

1. **S' cut ≥ 3.0%** on `wikidata ×4`: the upper 95% bound of the paired head/base ratio ≤ 0.970, *and* the head's slowest sample faster than the base's fastest (disjoint ranges). This is the claim's gate 2 verbatim.
2. **Guards:** no guard corpus's serial scan more than 2.0% slower beyond noise: STOP if its paired ratio is above 1.020 *and* its lower 95% bound above 1. The claim says "no > 2.0% regression"; a point estimate alone would stop on noise, since `runs` and `onebyte` are single blocks scanned in 0.03–0.3 ms (a local smoke run of both arms, before any i9 data, saw ±8% and 10–50% sample-to-sample spread there).
3. **Memory:** median peak RSS ratio ≤ 1.02 at every real cell (the claim's ≤ 1.02×).
4. **No whole-decoder loss on Wikidata:** at no real `wikidata ×4` count is the wall ratio above 1.020 with its lower 95% bound above 1. *Why this is added to the claim's gate:* #1117's exact-head CI bench showed the scanner −4.5% beyond noise but an unrelated +7.6% in MTF + RLE2 from a code-layout shift of `src/mtf.rs`; a scanner gain the i9 cannot see through such a loss must not ship. It stops only on a loss beyond noise, so it is not stricter than PERFORMANCE-POLICY.md's merge rule.

Real-decode ratios at every cell are reported in full; apart from rule 4 they are diagnostic (the knee question), not gates.

**On PASS:** #1117 merges under its exact-head gates; this run is its adoption evidence and stands in for PERFORMANCE-POLICY.md §2 rule 1, which CI's two cores cannot measure for a scanner cut while workers bind. **On STOP:** #1117 is not merged, and a scoped row goes in `frontier/dead-ends.tsv` naming this registration; the A2 levers stay open for a distinct rescue only on new evidence.
