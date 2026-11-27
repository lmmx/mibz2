claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/mtf-column-front-dissection.md

Smooth-zone dissection of `src/mtf.rs` (`Column`-route MTF/RLE2 front) on off-Wikidata
corpora (`elf`, `prose`, `pysrc`, `random`, `records`; 70 `Column` blocks + 4 guard
corpora `json`, `rustsrc`, `text`, `wikidata`; zero `src/` edits; 20 exact arms across
4 rounds in `examples/mtf_column_front_probe.rs`, 4,520/4,520 exact checks incl. 10
synthetic edge/error cases).

Verdict (strict C2 under frozen `>= 15.0%` gate): **STOP (scoped — sub-threshold gain
for standalone AVX2 `src/mtf.rs` list/RLE2 layouts)**.

Key findings:
- Symbol census: `nn >= 32` is `28.9%` on `elf` (`20.3%` `>= 64`, `flp32 = 27.3%`),
  `6.6%` on `prose`, `75.7%` on `random` (`47.3%` in `64..128`, `flp64 = 49.6%`), and
  `90.5%` on `records` (`48.5%` `>= 128`).
- Skeleton ablations isolate two bottlenecks: unpredictable `s <= 1` / `run != 0` RLE2
  branching takes `49.5%` of `prose` (`20.0 ms`) and `37.4%` of `elf` (`40.1 ms`), while
  the MTF list takes `43.6%` of `elf` and `70–73%` of `records`/`random`.
- Branchless RLE2 regresses `+5.7%..+57.1%` (32B store + `STEPS[0]` + `counts` RMW on
  run symbols); all-in-register 8xYMM regresses `+5.7%..+46.7%` (12c mux `v`-extract +
  Port-5 saturation); moving-base / variable-`k` chunk loops regress `+15.2%..+84.6%`.
- Unmitigated builds suffer 13–20% Skylake JCC erratum swings even on `wikidata` (`0.0%`
  `nn >= 32`). Under JCC-mitigated builds (`-x86-branches-within-32B-boundaries`, 7
  interleaved repeats), the best arms (`R_t32_flat_unal7` `-8.45%` base / `-9.36%` nat,
  `S_adapt_Q_P2` `-6.32%` / `-10.36%` nat, both disjoint) cut `random` `-10.6%..-15.4%`
  and `records` `-10.4%..-10.7%`, but move `elf` only `-2.8%..-3.9%` and `prose`
  `-0.9%..+2.6%` — below the frozen 15% admission bar.

No `mtf-column-front-build` claim admitted; scoped STOP recorded in `frontier/dead-ends.tsv`.
