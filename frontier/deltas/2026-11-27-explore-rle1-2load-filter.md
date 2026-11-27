claim: explore
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/rle1-2load-filter.md; examples/rle1_dissection.rs

## Verdict: PASS (admitted `rle1-2load-filter-build` at `status = "idea"`, `owned_paths = ["src/rle.rs"]`)
Dissected `src/rle.rs` (`find_runs_avx2`, `unrle_in_place`, `unrle_avx2`) across 14 arms, 8 corpora (`wikidata_x4`, `text`, `random`, `onebyte`, `pattern`, `runs`, `records`, `zeros`), L3-warm & L3-cold, and both `baseline x86-64` and `JCC-32B` builds (`101,234 / 101,234` bit-, CRC-, and error-exact checks; `0 B` extra memory; zero `src/` edits).

## Key Findings
- **`objdump -d` root causes:** Shipped `find_runs_avx2` runs 4 overlapping unaligned loads + 3 `vpcmpeqb` + 2 `vpand` per 32B and restarts on every run; shipped `unrle_avx2` emits **11 `vzeroupper` + 10 calls** (including `memcpy`, `memset`, `slice_index_fail`, `panic_bounds_check`) via `extend_from_slice`/`resize`.
- **Winning drop-in (`P5_m1_128b_u2stage` = `F3_m1_128b` + `U2_m1_64b_raw`, 0/64 regressions):**
  - `find_runs` (`F3_m1_128b`, 2-load 1-`vpcmpeqb`, 128B unrolled 2×`u64` + 2-bit carry + multi-hit drain): `wikidata_x4` **-23.8% / -21.6% warm**, **-10.9% / -11.3% cold** (all DISJ_FASTER); `text` **-36.5% / -40.6%**; `runs`/`onebyte`/`zeros` **-60.8%..-62.2%**.
  - `unrle` (`U2_m1_64b_raw`, 62B stride + raw `vmovdqu` + inline AVX2 fill, 0 hot `memcpy`/`memset`/`vzeroupper`): `text` **-55.1% / -55.9% warm** (`-51.8% / -53.3%` cold, 2.25× faster), `wikidata_x4` **-21.5% / -23.9% warm** (`-13.7% / -12.7%` cold), `runs` **-22.9%..-30.8%**.
  - Whole `unrle_in_place` (`A9`) & 1W share: `wikidata_x4` **-10.96% / -9.19% warm DISJ_FASTER**, **-6.99% / -5.76% cold** (~0.8–0.9% projected i9 1W, stacking orthogonally with `crc32-avx2-ring-barrett-build`); `text` **-15.0% / -16.1% A9 (-2.29% / -2.59% total 1W)**; `runs` **-14.7% / -16.6% A9 (-7.0% / -9.0% 1W)**; `zeros` **-10.5% / -11.2% A9 (-6.0% / -6.6% 1W)**; `onebyte` **-8.8% / -9.7% A9 (-4.8% / -5.7% 1W)**; `pattern` **-11.5% / -13.9% A9**.
- **Scoped negative ablations (`dead-ends.tsv`):** 32B un-unrolled `m1` (`F1_m1_32b`, `+1.0..+7.4%` sparse), 1-load `vperm2i128+vpalignr` (`F5`, Port-5 bound, `-0.2..+13.9%`), and inline variable-length small-copy/fill in `unrle_in_place` (`P3`/`P4`, `+6.1..+14.1%` DISJ_SLOWER on `text`) all STOP.
