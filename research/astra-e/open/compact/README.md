# Compact thin-run stream for the F-rank fill

The ten-block audit of the F-coordinate candidate found that 76.49% of band/run intersections on admitted blocks write a single table entry. The earlier implementation iterates every capped record twice, coalesces it into a maximal run twice and enters a range fill for each intersection. Here the first scan stores the selected byte and width in one u16 per retained run. A width fits in four bits plus one at the width-16 threshold, including the forced primary singleton. The second pass reads just the selected runs, stores a one-row intersection directly, and caches the active band's packed affine bias until its end.

These changes preserve each retained rank, label and table address exactly. For a band whose bottom ranks begin at j0, top ranks begin at k0 and high label bits are h, its packed entry for j is (h | k0) - j0 + j modulo 2^32. The source is k0 + j - j0 and stays below the next power of two reserved for the rank, so it cannot carry into the label bits. Wide mode sets h to zero and writes the separate label array. The tradeoff is two extra bytes of logical scratch per retained run plus capacity rounding.

Regenerate with `python3 research/astra-e/open/compact/prepare.py`. The input candidate SHA-256 matches commit `775e604`. The production density threshold, original maximal-run selection, forced primary split, endpoint graph, dictionary and writer are those of the parent experiment.

```sh
MOONSHOT_CANDIDATE=research/astra-e/open/compact/candidate.rs cargo test --manifest-path bench/moonshot/Cargo.toml 'candidate::' -- --nocapture
```

The 55,302 exhaustive coordinate checks pass. A separate test compares each table row to a stable-sort oracle, with 1/15-row alternating runs crossing reversed bands, maximum u32 packed entry, negative biases and forced separate wide labels. See `checks.txt`.
