claim: explore
by: claude-code
outcome: done
evidence: docs/notebook/2026-11-28-explore-block-size-route.md; master 9280643; examples/route_census.rs (ROUTE_FULL, ROUTE_SHADOW=first); corpora/wikidata.bz2 82f65ace re-encoded by bzip2 1.0.8 at -1/-2/-3/-5/-7/-9 (2ae39adf, aa55dcfa, 380cbfda, 78ee1f9e, 58c8897f, 817e6d89; not checked in)

**What was asked:** the recurring `explore` claim: a phenomenon off wikidata that no claim or dead-end covers, away from the scanner/worker tie.
**What I found:** I priced the decoder on the same 209.7 MB of text compressed at bzip2 levels -1..-9, that is, at block sizes from 100k to 900k rows. 4-vCPU VM, 2 MiB L2 a core, interleaved rounds.
- *Whole decode is U-shaped in level.* At t4, -1 is 1.39x -9 (220 vs 158 ms median). lbzip2 runs the other way (545 vs 1186 ms). Part of the gap is the scanner: -1 has 60% more symbols, so k* falls from 7.2 to 5.1.
- *The rest is a route choice.* `OpenCellsOf::cap` admits on r/n alone, and every block at every level takes Open. Production's inverse over `Segmented`'s on the same blocks: 1.17-1.20 at -1, 1.06-1.09 at -2, about 1.0 at -3, 0.71-0.76 at -5, 0.48-0.51 at -9. The timing order (shadow before or after) moves nothing. On the whole -1 file, `Segmented` alone beats production by 6-19%.
- *It is n, not only r/n.* At a fixed r/n the ratio is flat from 100k to 300k rows and drops between 300k and 500k, where `Segmented`'s 4n-byte vector passes the 2 MiB L2. Inferred: there are few misses for Open to save in a block that fits in L2. No PMU here.
- *Worth, on this VM (derived from per-block timings):* the per-block oracle is -19% / -13% / -9% of the -1 / -2 / -3 inverse. A two-parameter rule (below 400k rows, Open only for r/n < 0.075) gets -17% / -9% / -5% and leaves -5 and -9 unchanged.
**What changed:** the notebook note; `examples/route_census.rs` gains `ROUTE_FULL` (its fixed 100,000 cut silently dropped all 2,097 -1 blocks) and `ROUTE_SHADOW=first` (the order control). New claim `open-admission-block-size` (idea, P3, striated `src/open.rs`), with its stop test frozen. No surface row changed: the rows price wikidata -9, which this leaves alone. `explore` stays open.
**What's next:** `open-admission-block-size` must calibrate on CI's runner or the i9 first. Its step 1 STOPs if no bin below 400k rows loses by >= 5% there. Route and window statements should name their block size as well as their corpus.
**Caveats:**
- The crossover is a cache property, and this VM's L2 is 8x the i9's 256 KiB. The thresholds here are not portable, and on the i9 the effect could move or vanish.
- One text corpus and one encoder. Single-thread kernel timings, with no multi-thread A/B of a rule and no 8W RSS check.
- A stream's last short block is the same case, but it is negligible on large files.
