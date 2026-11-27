claim: release-0-5-17
by: antigravity
outcome: done
evidence: origin/evidence 5182d546 (release-scaling/run-20261127T213343Z, sha256 3b3af8b18436239e610424d5d6a3fcda8c74bf2bc71de1e0e9fb147b44c8309b); origin/evidence 0ffba60b (crossover/run-20261127T214442Z, sha256 facc37f599808d52ad8bcd4c74bcce6aa579d76fbd101e559bfa6d927e6456d4); bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.17/

**What was asked:** Record both physical i9-10900K runs (`release-scaling` §5.2/§5b and `crossover` §5a) for `v0.5.17` (`01acd74d`) and close `release-0-5-17`.
**Release scaling (`5182d546`, SHA-256 verified, 240/240 runs byte-identical):**
- Copied `recorded/` (`env.json`, `meta.json`, `runs.jsonl`) to `bench/release-scaling/2026-11-27-intel-r-core-tm-i9-10900k-cpu-3-70ghz-0.5.17/` and recorded figures in `CHANGELOG.md`.
- **Wall (`0.5.16 / 0.5.17`, paired 10 rounds, Wikidata 210 MB):**
  - Pinned: 1T `1.003 [0.963, 1.045]`, 2T `0.981 [0.953, 1.010]`, 4T `1.004 [0.982, 1.025]`, **8T `0.974 [0.967, 0.982]`** (`+2.63%` slower; CPU `0.977 [0.970, 0.985]`, `0.428 -> 0.437 s`), **10T `0.985 [0.976, 0.995]`** (`+1.52%` slower; CPU `0.982 [0.977, 0.987]`, `0.440 -> 0.448 s`).
  - Unpinned: 12T `0.980 [0.934, 1.028]`, 16T `0.984 [0.920, 1.053]`, 20T `0.959 [0.910, 1.010]` (all spanning 1).
- **Peak RSS:** median `27.7–67.1 MiB` (max single run `71.6 MiB` at 4T); 8T `40.6 vs 42.0 MiB` (`−1.4 MiB`), confirming zero `Segmented` fallback spill on all 233/233 full Wikidata blocks.
- **§5b attribution:** no thread count clears both `>= 3%` and 95% CI excluding zero (`8T` is `+2.63%`, `20T` spans zero); no `release-attribution.md` row.
- **Regression follow-up (RELEASING §1):** the measured `+8–9 ms` worker CPU (`+2.6%` 8T / `+1.5%` 10T) on Wikidata matches `within_cap` scanning the 39 Wikidata blocks in `0.10 <= r/n < 0.15` that previously entered `OpenCells` with `Cap::NONE`. Follow up in its own PR to eliminate that extra `within_cap` pass on `[0.10, 0.15)`.
**Team threshold (`0ffba60b`, SHA-256 verified, commit `01acd74d`, 400/400 same bytes): unmoved.**
- `mibz2-team2` is `1.224–2.315` wall and `mibz2-team3` is `1.395–2.317` wall at 4–10T; at trimmed counts `mibz2-workers-w` is `1.298` (6T), `1.050` (8T), `1.068` (10T); at 4T pinned `mibz2-workers-w` is `0.973 [0.958, 0.989]` wall for `1.058` CPU and `84 vs 67 MB` RSS; sanity at 5/7/9T (`0.982`, `0.988`, `0.992`) spans 1; SMT 12/16/20T `1.003–1.111`.
**Done:** `release-0-5-17` marked `done` (all 6 stop_test criteria met).
