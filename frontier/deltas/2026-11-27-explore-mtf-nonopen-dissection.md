claim: explore
by: antigravity
outcome: STOP (strict C2; scoped dead-end recorded)
evidence: docs/journal/2026-11-27-explore/mtf-nonopen-dissection.md; examples/mtf_nonopen_dissection.rs

## Summary

Dissected non-Open `src/mtf.rs` (`mtf_checkpoints_avx2` and `mtf_avx2`) via `objdump -d`,
exact symbol census across 7 corpora (648 blocks + 12 synthetic edge/error cases), and
4-profile interleaved `CLOCK_THREAD_CPUTIME_ID` benchmarks (`base`, `nat`, `jcc`, `nat_jcc`),
with zero production `src/` edits.

## Findings

1. **Root cause of `mtf_checkpoints_avx2` 1.24x-1.33x tax:** `objdump -d` shows 10 `vzeroupper`
   and 18 `call`s (inline `memset` on every run despite 93.9-100% of runs <=64 B, inline
   `memmove` on every `nn >= 32`, per-symbol `s2u` lookups, 6-8 stack reloads/symbol, and a
   1-byte `*base = v` STLF hazard).
2. **Zero-call 3-way blueprint (`T_zerocall_adapt`, 3,959/3,959 exact):** Pre-reserving
   `checkpoints` (`max / CHECKPOINT + 2`) with inline run fills and compact AVX2 checkpoint
   copies yields `0` hot-loop calls, `0` XMM spills, and `vzeroupper = 1`, cutting large
   checkpointed blocks (`text` -9.8..-13.4%* Haswell/Broadwell, -11.9..-17.0%* newer Xeon;
   `records` -10.3..-18.3%* / -19.7..-28.0%*; `wiki_all` -5.8..-18.5% / -13.5..-19.8%*) with
   zero Row-32 regression (`random` -2.4..-11.2% / -13.3..-23.5%*).
3. **Why verdict is strict C2 `STOP`:** No candidate clears the frozen `>= 10.0%` disjoint
   gate on BOTH `wiki_nonopen` AND `text` across all three builds (`base`, `nat`, `jcc`).
   `wiki_nonopen`'s 198 blocks average 45 B / 42 syms (0.004% of `wikidata.bz2`, 0 ckpts),
   hitting a mandatory 2.3 KiB/block setup floor (`counts` + `checkpoints[0]` + `s2u`,
   capping Haswell/Broadwell speedup at -0.57..-8.88%). Unmitigated `base`/`nat` builds also
   swing +-15-23 ms from Intel 32B JCC alignment.
4. **Plain `mtf_avx2`:** Inlining zero-call run/tail loops regresses +9..+39% on
   `wiki_nonopen`/`wiki_all`/`text`/`runs`, confirming `dead-ends.tsv` Row 29.

## Next

- Recorded scoped dead-end row in `frontier/dead-ends.tsv` and updated `frontier/surface.tsv`
  row `M`; no build claim admitted. Does not close a future i9-gated large-block
  `mtf_checkpoints_avx2` redesign under JCC-aligned builds (`#1119`/`#1139`).
