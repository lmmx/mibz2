claim: a6b-endpoint-build
by: antigravity
outcome: done
evidence: origin/evidence a6e427b1 (a6b-endpoint/run-20261127T081357Z.tar.gz, sha256 623a7096f412aabf5d6d06feb14d1fb828fd5e17c1cd12dc31f55c04ccc99daa); recipe bdcfeeeb2154; source faa9df70f98d (+a6b-endpoint-redesign vs default)

Read and verified the completed physical i9-10900K (Comet Lake-S, stepping 5, 10C/20T, rustc 1.99.1) evidence bundle for `#1053` (`a6b-endpoint-build`, successor recipe `#1127` / `#1131`). Archive SHA-256 matches `623a7096f412aabf...` bit-for-bit; all 12 Wikidata windows (`431` blocks, `209,700,196` B) and all 6 guard corpora matched `libbzip2` bit-for-bit, `scan_busy_residual` was `0.00238` / `0.00257`, and zero VOID conditions occurred.

**Frozen Verdict (registration verbatim, C2 preserved):** **STOP/KILL under `#1053`** (`hardware_pass = false`; failed `wikidata/1` and `worker_busy` gates).

Key measured ratios (`candidate / control`, 15 paired rounds, 95% CI):
- **`wikidata/1` wall:** `0.9824 [0.9776, 0.9872]` (`-1.76%`, paired median gain `+24.7 ms`, `1.303–1.315 s -> 1.280–1.290 s`, RSS `+592 KiB` / `1.0077x`) — real 1W gain, misses frozen `<= 0.965` (`>= 3.5%` / `>= 3.0%`) bar.
- **`wikidata/8` wall:** `1.0072 [0.9916, 1.0231]` (RSS **`+4,192 KiB` / `1.0374x`**); **`wikidata/10` wall:** `1.0091 [0.9783, 1.0410]` (RSS **`+5,460 KiB` / `1.0488x`**).
- **`wikidata/8/S` (ideal scanner, pure workers):** `busy_ns_per_w` **`1.0106 [0.9931, 1.0283]`** (`-2.1 ms/w` slower), `wall_s` **`1.0124 [0.9942, 1.0310]`** (`-3.0 ms` slower) — misses frozen `<= 0.965` bar.
- **`text/1`:** `0.9919 [0.9809, 1.0029]` (RSS `1.0176x`); **`text/8`:** `1.0028 [0.9818, 1.0243]` (RSS `0.9952x`).

**Root cause confirmed (matching the pre-registered L3/RSS hypothesis in `a6b-endpoint-build.toml`):** `#1053`'s fixed 64K SoA slabs (`Box<[NodeLinks/u32/u128; 65536]>`, `1.75 MiB` virtual) added **`+524–546 KiB/worker`** resident RSS (`+4.19 MiB` across 8W, `+5.46 MiB` across 10W), pushing 8-worker Comet Lake L3 occupancy past its cliff and erasing the `-24.7 ms` 1W speedup at 8W. Scoped dead-end added to `frontier/dead-ends.tsv` and `frontier/surface.tsv` row `C-A6b` updated; `#1053` remains off by default and is NOT promoted. Any future A6b follow-up must keep per-worker working set compact (`<= 1.01x` 8W RSS, sized to actual `n_nodes`, not fixed 64K arrays).
