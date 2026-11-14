# Cells census, 14 November 2026: serial share and split feasibility

The count behind the Cooperative Cells decision (rule: #145 LOG 6878326831). Untimed: these are
counts of instructions and bytes, with no timing. The Optimiser makes the decision, not this
entry.

**The rule.** s = (Ir of `Kernel::slots` + Ir of `Kernel::induce_cells`) / (that + Ir of
`Kernel::write_cells`), on Wikidata blocks, weighted by block size n. Proceed if s ≤ 0.40
**and** a block's output splits into k = 3 spans at top-level factor boundaries with each
span within ±10% of n/3.

## Result (Wikidata, all 431 blocks of the pinned corpus, nothing subsampled)

| | |
|---|---|
| s, weighted by n (196 admitted blocks) | **0.742** |
| s, pooled (Σ serial Ir / Σ Cells Ir) | 0.742 (production binary: 0.744) |
| s per block: min, quartiles, max | 0.715; 0.729, 0.735, 0.746; 0.813 |
| slots alone, pooled | 0.301 |
| Ir per row of admitted blocks: slots, induction, write | 4.48, 6.55, 3.83 |
| k = 3 within ±10%, blocks | **8 of 196** (4.1% of admitted n) |
| k = 3 least worst-span deviation: median, quartiles | 0.52 (0.35, 0.78) |
| k = 2 within ±10%, blocks | 51 of 196 (26.0% of n) |
| k = 4 within ±10%, blocks | 2 of 196 |
| largest top-level factor, share of n: median | 0.449; 146 of 196 blocks have one over 1.1·n/3 |

Both conditions fail on Wikidata: s is about 0.74 against ≤ 0.40, and the k = 3 split is
feasible on 8 of 196 admitted blocks. The serial part is the larger share on every admitted
block (the lowest s is 0.715). Splitting only the writer (`write_cells`), Amdahl's law limits
the gain on the Cells stage to 1/s ≈ 1.35× with any number of workers.

**Admission** (Wikidata, 431 blocks): 196 admitted, 185 refused for density, 2 refused on the
cell budget, 48 never reach Cells (`runs::takes_runs` false: the column's path). The density
refusals are mostly tiny blocks (median n = 40) plus a few full blocks with r/n ≥ 0.1. Admitted
blocks hold 84% of the n that reached Cells. Gen text: 49 blocks, all refused for density.
Gen pattern: 88 blocks, all refused as sparse. Neither has an admitted block, so neither has a
serial share. They were not run under callgrind.

**Cross-span dependence.** At the best k = 3 cuts, about half the bytes of spans 2 and 3
(51%) are long copies whose source lies in an earlier span. Copies from first occurrences are
78% to 84% of each admitted block's bytes (`long_copy_bytes` / n). So even where the spans
balance, a later span's writer reads most of its bytes from earlier spans' output.

## Method

- `examples/cells_census.rs` (feature `census`, off by default) decodes every block with
  `decode_block` and `bwt::Cells` on one thread, as `decode` does, with each block's CRC
  checked. After every Cells attempt it reads a `Census` record off the kernel
  (`src/cells.rs`, compiled only with `census`). The record has the outcome, the rules, their
  depth, and the spine. The writer's work is replayed from the rules and `pos` as
  `write_cells` left them: top-level factors at their output places, and every long copy as
  (destination, source, length). For every block, the bytes from the arena plus the
  long-copied bytes equal the period (`accounted: true`). No block has a cycle shorter than n.
- **Top-level factors** are the marked spine's plain factors plus the row's byte, at their
  places after the rotation. The **split** for each k is the one at those boundaries whose
  worst span deviation from n/k is least. It is found by bisection on the tolerance with an
  exact feasibility check, not greedily. **Cross-span bytes** count, for each span, the source
  bytes of its long copies that lie before the span's start.
- **Instructions:** `valgrind --tool=callgrind --dump-before='*Kernel>::slots'` on the census
  binary run with `--quiet` (no record output), giving one dump per Cells attempt (383).
  `census_s.py` takes each phase's inclusive Ir per dump: self cost plus callees such as memset
  and memmove. It is checked against `callgrind_annotate --inclusive=yes` on a dump. The three
  functions were already `#[inline(never)]`, so no hook was needed for the attribution.
- **Production cross-check:** the same corpus under callgrind on the production build
  (`examples/bzcat 1`, no `census` feature, `callgrind-prod-bzcat1.txt`). `slots` and
  `induce_cells` compile to the same instructions in both builds and count the same Ir:
  801.78M and 1171.85M over all attempts (to 150 Ir). `write_cells` compiles differently in
  the census build: 492 instructions against 469, and +1.0% Ir (676.35M against 669.77M). With
  production's writer count, pooled s is 0.744 instead of 0.742. The conclusion is unaffected.

## Uncertainty and limits

- Ir is not time. s counts instructions, and the induction is a dependent pointer chase. Its
  share of cycles may well be larger than its share of Ir, which would push s further from
  0.40. A timed s could differ in either direction, but a move from 0.74 to 0.40 would need
  the writer to cost about 4.3× its Ir share in cycles relative to the serial part.
- The split uses only top-level (spine) factor boundaries, as the rule says. A cooperative
  writer that cut inside a long top-level factor (descending into its rule) would balance far
  more blocks. That is a different design and is not measured here.
- Only one corpus has admitted blocks. The ±10% feasibility is per block, and the result is
  stated both as a count of blocks and weighted by n.
- The measured census binary was built before `rustfmt` reformatted
  `examples/cells_census.rs` (whitespace only). A rebuild from the committed source writes
  byte-identical records on all three corpora. Its binary hash differs, through the line
  tables. Both hashes are in `inputs-binaries.sha256`. rustc 1.99.1, valgrind 3.22.0, base
  commit ca6f2366a.

## Files

- `census-{wikidata,text,pattern}.jsonl`: one record a block.
- `phases-wikidata.jsonl`: per attempt, the callgrind dump (part) and inclusive Ir of each phase.
- `summary-wikidata.json`: `python3 census_s.py census-wikidata.jsonl CALLGRIND_DIR phases-wikidata.jsonl`.
- `callgrind-prod-bzcat1.txt`, `valgrind-*.log`: the callgrind summaries. The 384 raw dumps
  (56 MB) are not kept.
- `inputs-binaries.sha256`: the corpora and binaries. `SHA256SUMS`: the records.

Reproduce: `cargo build --release --features census --example cells_census`, build the
corpus as `bench-cells-vs-prod` does, then
`cells_census wikidata wikidata.bz2 > census-wikidata.jsonl`, and the callgrind run above with
`--quiet`.
