# MTF's short runs as two vector stores: registration and instruction count

Registered before any timing of this branch. The decision rule may be amended only in a commit
made before the first bench comment on this PR, and each amendment says what changed and why.

## The change

`mtf_avx2` writes each RUNA/RUNB run with `ptr::write_bytes`, which is a `memset` call.
Wikidata's block 1 has 17,384 runs. `memset` cost 612k Ir there, most of it call overhead on
short runs.

A run of at most 64 bytes, with n + 64 ≤ max, is now two 32-byte stores of its byte. The bytes
past the run are inside `ll`'s capacity (`ll.reserve(max)`). They are written again by what
follows, or left past the length set at the end, so no byte below the final length differs.
Longer runs, and runs near the block's end, keep `write_bytes`. The run-length checks
(`RunTooLong`, `BlockTooLong`) and the counts are unchanged, and so is the scalar `mtf`.

Variants measured on block 1, Δ Ir against master:
- one 32-byte store for runs of at most 32: −236,165;
- two stores for runs of at most 64: −326,433 (this branch);
- a 32-byte store loop for runs of at most 256: −29,173.

## Measured: whole-decoder instructions

Inclusive Ir of the driver's `measured` (one `decode_block` on a fresh `Scratch`), with both
arms built from the driver of `2026-11-05-deferred-flag-walk/` (rustc 1.97.0, fat LTO). The
input is `wikidata-independent.bz2`, sha256 `7f5c9a0044e5a305…`. Profiles and outputs are in
`cg/` (`base-*` is master 4d4709de, `branch-*` this branch).

| block | master | this branch | change | output |
|---|--:|--:|--:|---|
| 0 (54 B) | 7,289 | 6,974 | −315 | identical |
| 1 | 20,184,311 | 19,857,878 | −326,433 (−1.62%) | identical |
| 41 | 19,480,138 | 19,207,725 | −272,413 (−1.40%) | identical |
| 97 (56 B) | 7,665 | 7,418 | −247 | identical |
| 233 | 19,989,050 | 19,659,043 | −330,007 (−1.65%) | identical |
| 400 | 19,490,067 | 19,220,433 | −269,634 (−1.38%) | identical |
| 472 (70 B) | 8,503 | 8,192 | −311 | identical |

This is independent of #168 (the walk) and #169 (the build), and counts no saving of theirs.

## The decision rule

The same as `2026-11-05-build-run-vectors.md`:
- correctness, with CI green including Miri, and the same bytes;
- instructions at or below master's (met);
- benefit: `wikidata` at 1 or 4 threads, CPU or wall faster beyond noise in one run, and not
  slower in the other;
- no harm, with at most three runs.

It is closed as not shown if no run shows a gain beyond noise.
