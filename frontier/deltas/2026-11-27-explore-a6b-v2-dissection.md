claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/a6b-v2-dissection.md

Smooth-zone second-pass dissection and safe-Rust redesign probe of post-`#1046` `A6b`
(`17%` of `#1062` `W4`, `~155 ms` 1W / `~22 ms/w` 8W on i9-10900K) across 7 rounds
(`examples/a6b_v2_dissection.rs`, `#![forbid(unsafe_code)]`, 2,682/2,682 exact checks
on 431 wikidata blocks + 6x2 synthetic stress blocks incl. 40k/70k spirals + gen wrap;
zero `src/` edits).

Verdict (strict C2 under frozen `>= 15.0%` all-3-profile gate): **STOP (scoped —
sub-threshold on unaligned baseline/native; no standalone `src/` rewrite claim)**.

Key findings (233 admitted Open blocks, 7 interleaved repeats pinned to CPU 0):
- Census: `gen_cache.pairs` hits only `1.06%` (`1,134 / 106,866`); empty operands are
  `38.26%` of `2.67M` joins (`<= 7B`+empty `~73%`, `<= 15B`+empty `85.3%`, `LONG 14.74%`);
  `97.37%` of bands are whole-node; `len == 2` digrams are `22.79%` of dict calls
  (`100.00%` ASCII); `x == y` is `0/2.67M`; `append_rules` `max_stack = 16`.
- Killed mechanisms: 64 KiB direct ASCII digram table (`D4_a16k_s8_flat`) and 32 KiB
  `Short8` table (`D3_s8_flat`) both lose to a unified 16 KiB `contents_4k` (`D2`,
  L1d/L2 eviction); 16B AoS `Node16` inline-5 (`V4_aos16`, `-1.5%..-8.2%`) suffers
  inline-5 spills + 16B link stride; register `(wx, wy)` in `eliminate` regresses
  `+1.8..+4.2 ms` (GPR spills + 3x join bloat); split 4B links (`spl4`) are inconsistent.
- Best safe-Rust designs (zero-call empty `join` + no `pairs` + `gen8+fp8` fingerprint
  in 16 KiB `contents_4k` + `tag < 8` 64b dict + 64K zero-bc dict/bands/rules):
  - `V3_i7_lazy` (SoA-inline7 `words64`, `209 KiB` L2-resident) clears `-15%` on the
    32B JCC-aligned control (`-16.05%` R7 / `-16.39%` R5, disjoint; `dict -29..-35%`,
    `setup -33%`), but stops at `-11.70%` (`base`) and `-11.15%` (`nat`, disjoint).
  - `V2_u128_base`/`V2_u128_lazy` deliver `-12.22%..-13.61%` across all 3 builds
    (disjoint; `C+E_cpu -2.0%..-3.8%`, `A7 -1.1..+0.04%`, ~`+19..21 ms` 1W / `+2.7..3.0 ms/w`
    8W on i9-10900K) — clean opportunistic harvest (~80 LOC) if folded into a future
    `src/open/endpoint.rs` cleanup, but not worth a standalone claim over `A7+A9`/`A5+A6a`.

Updated `frontier/surface.tsv` (`C-A6b`) and recorded scoped STOP in `frontier/dead-ends.tsv`.
