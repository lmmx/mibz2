# Pre-release §5c worker-count recalibration for v0.5.18 (frozen registration)

**Pinned decoder:** `7be505133b3f5e00e9de983e3177aa619bfb2ccc` (`origin/master`, post-`v0.5.17` `01acd74d17bfea0ab891e75eb4fc2ef3838f5cd8`)
**Recipe:** `just bench-recalibrate-0518` on branch `bench/recalibrate-0518` (zero changes to `src/`, `Cargo.toml`, or `Cargo.lock` relative to `7be50513`)
**Policy:** `docs/RELEASING.md` §5c and §5.4; `docs/PERFORMANCE-POLICY.md` §2

## 1. Why §5c recalibration is required before v0.5.18

Between `v0.5.17` (`01acd74d`) and the pinned `v0.5.18` candidate (`7be50513`), worker-cost and scanner-cost paths changed across multiple merged PRs:

- `#1246` (`27f54d0c`, `src/open.rs`): compact `u32` anchor table and `copy_slice` in Open emission.
- `#1117` (`f9114a34`, `src/block.rs`, `src/huff.rs`, `src/runs.rs`): fast A2 scanner Huffman setup and zero-copy symbol hand-off (reducing scanner work per block).
- `#1294` (`e65b91fa`, `src/open/endpoint.rs`, `src/open.rs`): A6b `u16` induction table (smaller L1/L2 working set per worker).
- `#1274` (`26466d6d`, `src/runs.rs`): AVX2 `runs::front_avx2` 64-symbol `vptest` front-end (faster per-worker MTF/RLE2 run build).
- `#1312` (`9f4ec45a`, `src/open/lanes.rs`, `src/open.rs`): default 4-lane forward-anchor Open emission walk (faster per-worker emission on 4 lanes with 32 KiB disjoint slabs).
- `#1234` (`faa976e5`), `#1268` (`2975e4c2`), `#1335` (`ffd2084a`): zero-`vzeroupper` `runs/fold`, `ab` harness, and `abi3-py310` Python delivery.

Because both worker per-block cost (emission, induction, MTF/RLE2) and scanner per-block cost moved substantially—and `v0.5.17` itself had a known 8/10-thread regression on Wikidata relative to `v0.5.16`—the shipped i9 worker trims (`src/parallel.rs`: pinned 10 -> lazy `7/28`, pinned 8 -> `6/24`, pinned 6 -> `5/20`, pinned 4 -> eager `3/12`; unpinned 10/12/16/20 -> lazy `8/32`, unpinned 8 -> `8/32`, unpinned 6 -> `6/24`, unpinned 4 -> `4/16`) must be re-swept before releasing `v0.5.18`.

## 2. MONODROMY traffic-model prediction (§5c.3, frozen before data)

Under MONODROMY's shared-L3/DRAM contention and scanner-saturation model (records 45–55; scanner contention residual confirmed at `a7ac01d8` showing 76.4% of the 8W/4W scanner cycle excess comes from shared-L3 miss stalls):

- **Worker-side per-block work dropped more than scanner-side work** (`#1312` saved ~17.2% 1T / ~10.6% 8W worker busy time; `#1274` cut `runs` front-end cost; `#1294` shrank induction scratch; `#1117` cut scanner setup modestly).
- Therefore, **the scanner saturation crossover $k^* = a / b$ shifts downward slightly** (from ~8–9 workers toward ~7–8 workers on Wikidata x4), while the lower L1/L2 worker footprint from `#1294` slightly reduces per-worker LLC pressure at 4–6 threads.
- **Frozen predictions:**
  - **Unpinned 10, 12, 16, 20 threads:** shipped `8/32` (`UNPINNED_FROM_TEN = 8`) will either hold or tie `7/28` within noise; `9/36`, `9/27`, and `10/40..10/80` will NOT beat `8/32` (scanner is saturated and extra workers add L3-miss stalls on the scanner core).
  - **Unpinned 8 threads:** shipped `8/32` and `7/28` will be within 1–2%; if `7/28` wins on Wikidata, it will be by reducing L3-miss interference on the scanner.
  - **Unpinned 6 and 4 threads:** shipped lazy `6/24` and `4/16` will beat both one fewer worker (`5/20`, `3/12`) and eager (`oce-6-24`, `oce-4-16`).
  - **Pinned 10, 8, 6, 4 threads:** shipped `7/28`, `6/24`, `5/20`, and `oce-3-12` will hold at 4/6/8; at pinned 10, `7/28` will tie or narrowly beat `6/24` and `8/32`, and beat `9/36` and `10/40`.

## 3. Experimental design & frozen decision rule

- **Machine:** Louis's physical Intel Core i9-10900K (10 cores, 20 CPUs, 20 MiB L3, `cpu_quota = none`), `rustc 1.99.1`, default release profile (`src/`, `Cargo.toml`, `Cargo.lock` byte-identical to `7be505133b3f5e00e9de983e3177aa619bfb2ccc`, verified by `git diff --quiet`).
- **Corpora:** `wikidata-x4` (4x `bench/wikidata.json`, primary), `text` (16 MiB, primary guard), `random` (16 MiB, secondary guard).
- **Rounds & settle:** 12 paired rounds per cell (`ab` rotates and reverses arm order). Pinned 4/6/8 hold an auto-placed spinner on a free core (`--hold-cpu auto --hold-max-threads 9`, 0.25 s settle). Pinned 10 and every unpinned cell (4, 6, 8, 10, 12, 16, 20) run with **2.0 s settle** (`SETTLE_HI=2`) to exclude slow clock mode (b) hysteresis (`RELEASING.md` §5.4).
- **Holm families (`bench/two_arm.py`, `--alpha 0.025`, `--primary wikidata-x4,text`, `--check-workers`):**
  - **Pinned (14 candidate cells):** refs `7/28` @ 10, `6/24` @ 8, `5/20` @ 6, `oce-3-12` @ 4.
  - **Unpinned (25 candidate cells):** refs `8/32` @ 8/10/12/16/20, `6/24` @ 6, `4/16` @ 4.
- **Decision rule:**
  - If `two_arm.py` returns `NOT ADOPTED` for both families (or `A FINDING, NOT ADOPTED` with no valid trim gain): keep the shipped trims in `src/parallel.rs` unchanged and record the verdict in the `v0.5.18` release receipt and `CHANGELOG.md` (§5c.4).
  - If any candidate cell passes `ADOPTS` under Holm `alpha = 0.025` with no primary/guard loss or RSS violation: retuning `src/parallel.rs` is a separate C6 trim PR (re-verifying `bench-release-preflight` on the resulting candidate SHA before releasing `v0.5.18`).
- **Diagnostic (decides nothing):** system-wide IMC DRAM reads/writes (`bench/counters.py --events imc`, 5 reps) when `kernel.perf_event_paranoid <= 0`, skipped cleanly otherwise.
