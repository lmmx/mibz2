claim: none
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/direct-literal-4lane-emit.md, examples/direct_literal_emit_probe.rs, frontier/dead-ends.tsv

**What was asked:** Test whether bypassing `Open`'s 2-level dictionary lookup (`lens[code]` + `inline16[code]` + 16B store) on 1-byte literals (`code < 256`, 81.58% of `A7` steps) and stripping the 256 literal entries from `inline16`/`lens` (`-4.25 KiB/blk`) cuts 4-lane forward `A7` (or `A6b-dict + A7`) by `>= 5.0%` of `A7` (frozen stop bar, disjoint `[min..max]` on baseline x86-64 and JCC-aligned `-C target-cpu=native`, warm and L3-cold, `#![forbid(unsafe_code)]`, zero `src/` edits).
**What I found:**
- **Structural census (233 admitted Open blocks of `wikidata.bz2`, 26.57M `A7` steps, 233/233 byte-exact across all 5 arms + post-compact reuse check):** `81.58%` 1B literals (avg sequential streak `8.81`), `11.99%` 2..16B inline words, `6.43%` `LONG` words (`93.57%` unified inline fast-path); `44.59%` of 4-lane Phase-1 iterations are all-4-literal (avg all-4 streak `2.66`); compact dict shrinks avg `inline16+lens` from `17.58` to `13.33 KiB/blk` (`-4.25 KiB`).
- **Unanimous STOP across all 3 builds × 2 cache regimes × 2 suites (7 interleaved repeats, steal-free `schedstat` CPU, 24/24 isolated `A7` cells DISJOINT SLOWER vs `ctrl_4lane_dict`):**
  - Baseline `A7`-only: `ctrl 244.39` warm / `323.96 ms` cold vs `lit_branch_per_lane +12.44% / +7.63%`, `lit_all4_fastpath +23.50% / +12.56%`, `lit_streak_inner +38.98% / +27.04%`, `lit_plus_compact_dict +16.58% / +4.47%` (combined `A6b-dict+A6c+A7`: `+7.25%..+34.64%` slower).
  - Standard native `A7`-only: `ctrl 237.63` warm / `307.99 ms` cold vs `+11.47% / +7.47%` (`per_lane`), `+20.69% / +14.50%` (`all4`), `+35.58% / +31.60%` (`streak`), `+12.70% / +7.66%` (`compact_dict`).
  - JCC-aligned native `A7`-only: `ctrl 220.81` warm / `289.85 ms` cold vs `+10.70% / +6.50%` (`per_lane`), `+20.33% / +13.61%` (`all4`), `+37.09% / +29.51%` (`streak`), `+10.46% / +4.12%` (`compact_dict`; combined `+9.65%` warm / `+9.19%` cold SLOWER).
- **Root cause:** Shipped `inline16[code]` (`17.58 KiB`, L1d-resident) merges `93.57%` of steps into one highly predictable branch whose L1d loads/16B stores hide in the OoO shadow of 4-lane `edges[j]` cache misses. Splitting `code < 256` (`81.6%/18.4%`) or all-4 (`44.6%/55.4%`, `H = 0.992` bits coin flip, streak `2.66`) trades free OoO shadow work for ROB-flushing branch mispredictions that starve 4-lane MLP. Compact `A6b-dict` saves ~0 ms (`Dictionary::code` already short-circuits `handle == 0`).
- **Diagnostic bonus:** JCC alignment (`-x86-align-branch-boundary=32`) alone cut `ctrl_4lane_dict` `A7` by `-7.1%` warm / `-5.9%` cold (`237.6 -> 220.8 ms`, `308.0 -> 289.9 ms`).
**What changed:** Added `examples/direct_literal_emit_probe.rs`, `docs/journal/2026-11-27-explore/direct-literal-4lane-emit.md`, this delta, and a scoped `STOP` row in `frontier/dead-ends.tsv`. Zero `src/` edits.
**What's next:** Keep unified branchless `inline16[code]` in `forward-emission`; focus remaining `A7` effort on branchless levers (higher lane count `K = 6/8` on i9-10900K, prefetching, or sublinear visit-reducing contractions).
**Caveats:** VM 2-vCPU diagnostic (C5); `schedstat` CPU timer avoids concurrent-VM preemption spikes seen in wall `Instant`.
