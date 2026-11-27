claim: explore
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/runs-front-and-fill-dissection.md; examples/runs_front_fill_dissection.rs

## Verdict: PASS (admitted `runs-front-and-fill-build` at `status = "idea"`, `owned_paths = ["src/runs.rs"]`)
Dissected `src/runs.rs` (`runs::front_avx2` and `runs::fill_avx2_trusted`) across 7 rounds, 8 `Runs`-routed corpora (`wikidata`, `text`, `mdjournal`, `elf`, `json`, `runs`, `pattern`, `onebyte`), L3-warm & L3-cold (64 MiB scrub), and both `baseline x86-64` and `JCC-32B` builds (`160/160` synthetic edge/error cases + `300/300` corpus blocks + `TT` exact; `0 B` extra heap; zero `src/` edits).

## Key Findings
- **Nanosecond phase split & `objdump -d` root causes:** Pass 2 is **85–91%** of `front_avx2` on real corpora (Pass 1 = **10–15%**, Pass 3 = **0.1–0.4%**). Shipped `front_avx2` (962 insns, 5 `vzeroupper`, 7 calls, 27-insn / 142 B hot loop) runs 4×16B loads + 4×`vpshufb` (4 Port-5 uops) + reloads `STEPS` + `movabs $0x100000000; or` on every literal to maintain private `Runs.per`, which `rg -n '\.per\b' src/` proves has **zero production readers** anywhere in `src/` (only `src/runs/tests.rs` and research-only `src/ideal/mtf.rs`).
- **Winning drop-in (`U1_u32`: 64-sym `u64` `vptest` Pass 1 + branchless 256-bit `YMM_STEPS` + `[u32; 256]` stack `cnt` with `Runs.per` gated to `#[cfg(any(test, feature = "ideal-mtf"))]`):**
  - Compiles to a **19-insn / 100 B (3 uop-cache-line) hot loop** with `YMM_STEPS` hoisted into `%rdi` (freed by removing `1u64 << 32`), 3 Port-5 uops/lit, 2 loads (vs 4), 0 `movabs`/`or`, 0 hot-loop spills/`call`/`vzeroupper`, `0 B` extra heap (`-1 KiB` `Runs` struct & stack).
  - **11-rep results (`0/64` regressions across all 8 corpora & 4 regimes):**
    - `runs::front`: `wikidata` **-19.4..-19.7% base warm / -14.8..-17.7% base cold / -17.3..-17.7% F JCC warm / -17.3..-17.4% F JCC cold**; `text` **-18.5% F base / -14.9..-15.9% F JCC**; `mdjournal` **-16.8..-18.6% F**; `json` **-16.0..-19.6% F**; `elf` **-5.0..-11.8% F**; `runs` **-5.6..-9.1% F**; `pattern`/`onebyte` neutral-to-faster.
    - `runs::front + fill_avx2_trusted`: `text` **-10.5..-17.3%**, `mdjournal` **-11.3..-19.6%**, `json` **-8.0..-13.2%**, `elf` **-4.4..-12.7%**, `runs` **-5.0..-9.9%**.
- **Scoped negative ablations (`dead-ends.tsv`):** `s<17` fast path (`+10..+18% S`), hot-loop cold calls (`+25..+80% S`), `second_tiered`/`second_vmovdqu` (`+9..+17% S`), L1d-tiled stack bitmaps (`c` stack spill / 12k tile-call `vzeroupper`, capping at `-9.5..-10.7%` JCC vs `-17.7%`), single `u16` store (`+6..+9%`), `while w != 0` (`+8..+15%`), direct `f.counts` (`+4%`), 32-sym Pass 1 (`+5.5%` JCC / `+15%` base), and all `fill_avx2_trusted` length-branching / `vbroadcastss` arms (`+5..+84% S`) STOP. Shipped `fill_avx2_trusted` is confirmed at silicon optimum.
