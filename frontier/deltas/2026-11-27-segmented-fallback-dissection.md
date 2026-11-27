claim: segmented-fallback-dissection
by: antigravity
outcome: done
evidence: docs/journal/2026-11-27-explore/segmented-fallback-dissection.md, examples/segmented_dissection_probe.rs, frontier/claims/segmented-fallback-build.toml, #1032

**What was asked:** Dissect `Segmented` fallback inverse-BWT phases across off-wikidata corpora (including primary `ab gen text`) and test concrete, 100% byte-exact levers against a frozen >= 10.0% gate on both baseline x86-64 and `-C target-cpu=native` (zero `src/` edits).
**What I found:**
- Across 6 local corpora (82.2 MB, 86 full blocks), `Segmented` handles **90.7% (78/86)** of full blocks, including **100% (25/25)** of primary `ab gen text` and **98.0% (50/51)** of `prose/text + source`. Inverse BWT is 52.5–65.8% of worker time on text/prose/source/JSON, with T-vector `build` alone taking **35–46%** of inverse BWT.
- **PASS (`C_trustfill+rev`, zero extra memory, 100% byte-exact on 78/78 blocks):** stacking (1) sequential `LF` column build with 8B equal-run fast path + `walk_back_room` (`-23.4%..-24.8%` on `Column`), (2) AVX2/u64 vectorized `assemble_reversed` (`-45%` on baseline assembly), and (3) O(1) invariant-checked `runs::fill_avx2` skipping redundant per-run tiling branches (`-13.5%..-14.0%` on `RunsFallback`) cuts `prose/text + source` inverse-BWT time by **-15.44% baseline / -16.05% native** (steal-free thread CPU, disjoint `[161.15..170.73]` vs `[193.56..200.61]` and `[156.27..163.13]` vs `[187.10..221.09]`; wall median **-15.77% / -15.91%**, 1/7 repeat had a 26–33 ms VM hypervisor preemption). Every corpus improves `-10.5%` to `-26.4%` (`-18.4%` across all 6; primary `text` `-11.7%` base / `-15.9%` nat), with zero Wikidata impact.
- **Ruled out:** KKP 2-step fused `LF²` (`Vec<u64>`) cuts walk `-15%` but blows up build `3.9–4.2×` (`+104..121%` net slower, +3.6 MiB/worker); `PF_NTA`, `PF_OFF`, unstaged direct-write walk, branchless un-unrolled walk, and scalar-1B `fill` branch all regressed.
**What changed:** Added `examples/segmented_dissection_probe.rs` and `docs/journal/2026-11-27-explore/segmented-fallback-dissection.md`; marked `segmented-fallback-dissection` `done`; opened `segmented-fallback-build` at `idea`; recorded KKP 2-step `LF²` in `frontier/dead-ends.tsv`.
**What's next:** Admit and build `segmented-fallback-build` (`src/bwt.rs`, `src/runs.rs`) under `PERFORMANCE-POLICY.md` §2 and B5/Miri.
**Caveats:** VM 2-vCPU single-thread diagnostic (C5); wall gain on `text` and zero Wikidata regression will be settled by CI's paired bench in `segmented-fallback-build`.
