claim: explore
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/cooperative-decoder-dissection.md, examples/coop_dissection_probe.rs, frontier/dead-ends.tsv

**What was asked:** Cold primary-evidence dissection of `src/coop.rs` (`126 KB`): where `Cooperative` triggers in production, 1/2/4-block scaling vs `Workers`, exact phase breakdown, and whether a `>= 10%` lever warrants a `coop-build` claim.
**What I found:**
- **Production trigger audit (verified at runtime):** `Plan::for_threads` ignores block count; on `<= 10` cores (including CI and the 10-core i9-10900K via `Machine::service_target()`), `StreamDecoder::new` ALWAYS returns `Plan::Workers` (lines `3578` & `3585`), making `PIPELINED_PLAN` (`L3588`) and `SERVICE_PLAN` (`L474`) unreachable via `StreamDecoder::new`. Only Linux `phys >= 11, threads >= 11` (or `6..7` oversubscribed) trigger `Cooperative<2>` by default.
- **Loses to `OpenCells` on Wikidata & Amdahl-capped elsewhere (2-core Xeon VM, 7 interleaved repeats, baseline & native):** On 1-block `wikidata`, `1t OpenCells` (`3.07 ms`) beats `Cooperative<2>` (`8.22 ms`, `0.37x`) and `Cooperative<4>` (`5.78 ms`, `0.53x`) because `Cooperative` never runs `Open`. On `text`/`prose_src`/`records`/`random`, `build` (`1.56–1.83x`) and `walk` (`1.81–2.01x`) scale cleanly at `k=2`, but serial `Huffman + MTF/front + cuts + asm + RLE1+CRC` is **`61.5%..80.8%` of `k=2` wall** (`mtf_checkpoints` alone is `58%` of `records`/`random` wall). Cold `Pool::new` (`110–290 us`) and warm barriers (`1–77 us`) are negligible (`< 1%`).
- **Candidate levers (`C_all`: parallel sequential `LF` build + `TILE=256` staged walk + trusted `fill_share` + AVX2 `assemble_reversed`, 100% byte-exact):** Cuts `Cooperative` inversion `-25%..-33%` (DISJOINT) on `records`/`random` and `-13%..-19%` on `text`/`prose_src`, but Amdahl limits full-block wall gain to **`-5.2%..-6.2%` (`< 10%`)** — no `coop-build` claim opened.
- **Negative finding (`dead-ends.tsv`):** Porting `mtf_avx2`'s register `(f0, f1)` + 7-row `second()` to `mtf_checkpoints_avx2` regresses **`+42.2%..+42.6%`** on `records` and **`+24.7%..+42.4%`** on `random` because 87.5% of indices are `nn >= 32` on uniform `Column` blocks (hitting 224B SIMD scan vs shipped `PagedList`'s `ptr::copy` of `~112`B).
**What changed:** Added `examples/coop_dissection_probe.rs` (`--features moonshot`), `docs/journal/2026-11-27-explore/cooperative-decoder-dissection.md`, and 2 rows to `frontier/dead-ends.tsv`; left `explore` open.
**What's next:** Keep focus on `OpenCells` / `Segmented` / scanner / vectored RLE1+CRC; `Cooperative::invert_runs` will inherit AVX2 `assemble_reversed` for free from `segmented-fallback-build`.
**Caveats:** 2-core VM diagnostic (C5); `k=4` reflects 2x oversubscription.
