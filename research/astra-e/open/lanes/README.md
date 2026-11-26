# Four residual walks writing byte fragments

The candidate keeps the width-16 queued endpoint constructor and opend density line from the journal at `9c31461`. Its residual writer decodes variable-length labels during four independent walks and assembles the resulting byte fragments with one bulk copy per fragment. The earlier `segments.rs` ranked writer cut at approximately m/128 seeds, staged u32 word codes, and decoded each code in the serial final assembly. This candidate uses 16–32 seeds and does its word decoding during the walks.

For m retained rows choose a power-of-two stride at least ceil(m/32) and take rows congruent to the mark. Each seed walks to its next seed, assigning consecutive decoded bytes into one of four reusable tapes. The residual graph is a permutation. Each seeded cycle partitions uniquely into half-open paths between consecutive seeds. Across all seeds every edge is therefore visited at most once. The sum of payload lengths on those paths is bounded by the original block length because each residual edge represents a disjoint original-row path. The marked cycle has a seed by construction. Starting at that seed and following the fragment successor graph spells that cycle in text order, including the first edge at the mark through the last edge before its return. The normal repeat-to-n rule applies unchanged.

At each full four-lane iteration the four edge addresses are available before any code is expanded. A finished lane claims the next seed. Once all seeds have been assigned and a lane finishes, at most three active fragments remain, and the writer drains these serially. Each word is expanded once and every selected output byte is copied once more. Unmarked seeded components add speculative work. The tapes collectively reserve approximately n bytes initially. Uneven lengths and previous high-water marks can increase retained capacity. The small-block fallback uses the reference writer directly.

Construction and scorer entry are reproducible with `python3 research/astra-e/open/lanes/prepare.py` and `research/astra-e/open/lanes/candidate.rs`. The generator pins the Open-Return source hash and uses its original endpoint module. The writer tests compare against the original serial walk over forward cycles, reversing permutations, shuffled permutations and singleton cycles, with packed and separate codes and variable-length words.

```sh
MOONSHOT_CANDIDATE=research/astra-e/open/lanes/candidate.rs cargo test --manifest-path bench/moonshot/Cargo.toml fragment_assembly -- --nocapture
```

All 856 comparisons passed. The raw log is in `fixtures.txt`.
