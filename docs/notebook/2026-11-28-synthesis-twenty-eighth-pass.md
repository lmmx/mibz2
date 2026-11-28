# Synthesis, twenty-eighth pass: `v0.5.18` shipped worker-bound, bimodal VM-to-i9 priors re-rank the `v0.5.19` batch, and `M5` clears the zero-RSS path to 5.5+ GB/s

*28 Nov 2026, master `e23d8c76`. Claim `synthesise`, continuing from the twenty-seventh pass (`72954c22`, #1358). No new benchmark runs; every measured figure is quoted from the named PR, notebook, delta, or `origin/evidence` bundle, and anything marked **derived** is arithmetic on those figures.*

Read since `72954c22` (15 commits on `master`, plus `frontier/surface.tsv` and the 6 active review-ready PRs): #1357 (0.5.18 i9 preflight readback, `c8489a45`), `edf2ceaf` (`v0.5.18` release), #1360/#1361/#1363 (0.5.18 changelog, release-scaling `16d5e409`, crossover `6e5d4cbb`, `release-0-5-18` closed), #1354 (`sensate-backpressure`), #1356 (`continuation-runbook`), #1362 (composed next-gen stack `M0 -> M5`), #1364/#1367 (`ab-station-ledger` and `bench/station_ledger.py`), #1365 (`L6a-on-1312` and `K=8` prefetch evaluation), #1366/#1368 (`vm-i9-transfer-ledger`), and #1369/#1370/#1371 (`bench-block-size-route` C1a i9 probe queued at `80dc3534`).

## 1. `v0.5.18` shipped, closed the §2a freeze, and `station_ledger` confirms every shipped count stays worker-bound

`release-0-5-18` is closed (#1363) on three physical i9-10900K bundles:

- **Preflight (`c8489a45`, #1357):** `VALID PASS`, recovering the `v0.5.17` regression and beating `v0.5.16` at both recovery cells (8T `0.991 [0.984, 0.997]`, 10T `0.987 [0.977, 0.997]`).
- **Release scaling (`16d5e409`, #1363, `0.5.17 / 0.5.18` on 1x Wikidata 210 MB):** pinned **8T `+3.9%`** (`1.039 [1.034, 1.044]`, CPU `1.038`) and **10T `+4.1%`** (`1.041 [1.036, 1.046]`, CPU `1.035`), peak RSS within `1.6 MiB` of `v0.5.17` at every count (`27.8–67.1 MiB`).
- **Crossover (`6e5d4cbb`, #1363):** cooperative teams remain slower at every count (`mibz2-team2` `1.214–2.353×`, `mibz2-team3` `1.386–2.216×` at `4–10T`); team threshold unmoved.
- **Direct lookup via `bench/station_ledger.py` (#1367):** reading `16d5e409` and `6e5d4cbb` directly (zero hand synthesis; also verified to `<= 0.050 ms` across all 9 Wikidata cells of `f8b21e4c`) confirms shipped `v0.5.18` (`mibz2`) stays **worker-bound at every shipped parallel count (`T = 2..20`, `k = 2..8`)**:
  - `16d5e409`: `W/k - S` falls from `+135.0 ms` (`T=2`) through `+20.0 ms` (`T=8`, `k=6`), `+10.1 ms` (`T=10`, `k=7`) to `+6.6..+6.8 ms` (`T=12..20`, `k=8`), with a **`14.8–24.4 ms` residual** (`wall - max(S, W/k)`) at `T = 8..20` and flat `W` (`-8.1%`, no growth flag).
  - `6e5d4cbb`: identical worker-bound profile on shipped `mibz2` (`+77.7 -> +4.0 ms`, flat `W` `-17.8%`), touching a `tie` (`-0.2 ms`) only at untrimmed odd `T = 9` (`k = 9`). Untrimmed `mibz2-workers-w` crosses to scanner-bound at `T = 10..20` (`-2.5..-20.6 ms`) while triggering **`W_GROWTH(+24.1%)`** (`k=4->20`, `345 -> 428 ms`); `mibz2-team2` (`+657.6%`) and `mibz2-team3` (`+270.0%`) flag severe contention/waiting.

**Consequence:** the §2a candidate-input freeze is lifted and the `v0.5.19` default-path batch is open. Every shipped `v0.5.18` count still converts worker cuts ~1:1 into wall and masks scanner cuts on Wikidata.

## 2. Bimodal VM/CI-to-i9 transfer priors (#1368) refine `synth-27` and re-rank the queue

`synth-27` §2 applied a coarse `0.2–0.5×` discount to all VM worker savings from two levers. `#1368` (`vm-i9-transfer-ledger`, 10 primary same-station same-corpus levers plus the `v0.5.18` cohort and `#1328` CI-EPYC reference) shows the transfer ratio is **sharply split by station and microarchitectural mechanism**:

| Station / Mechanism | VM control | `n` | Median `i9 / VM` | Range | Operational pricing rule |
|---|---|---:|---:|---:|---|
| **Station `S` (serial scanner)** | `base` / `JCC-32B` | `4` / `2` | **`1.05×` / `1.01×`** | `0.99–1.13×` | Transfers **~1:1**; converts to wall when target plan is scanner-bound |
| **`W` — compute / ALU** | `base` / `JCC-32B` | `3` | **`0.45×`** | `0.20–0.47×` | Discount VM `JCC-32B` cuts by `~0.45×`; pre-screen on CI `#1245` first |
| **`W` — MLP / latency-hiding** | `base` / `JCC-32B` | `2` | **`1.37×` / `1.85×`** | `1.24–1.97×` | Comet Lake's 224-entry ROB + miss penalty **amplifies** MLP savings past `1×` |
| **CI EPYC `#1245` (`1T lazy`) → i9** | `base` | `1` (`v0.5.18`) | **`1.04×`** | `0.92–1.10×` | Tracks physical i9 `8T/10T` nearly **1:1** at zero i9 queue cost |
| **Failure modes (sign reversal)** | `base` / `JCC` | `2` | **`-0.28×` / `-0.19×`** | — | `#1053` 8W L3 cliff (`+4.19 MiB`) and `#1119` Haswell JCC-32B flag |

**Two-way synthesis across `#1368` + `#1351` + `#1367` (*derived*):**

1. **Why `#1324` (`forward-anchor-lanes`, bounded `80 KiB/w`) sits alone at P1:** it is the only MLP lever (`1.50×` base / `1.97×` JCC amplification, already measured `-17.10%` 1W / `-11.10%` 8W on `1950f77f` at `0.990×` 8W RSS, `4.961 GB/s`) that flips 8W Wikidata from worker-bound to **scanner-bound**, unlocking ~1:1 wall conversion for the entire `1.05×`-transferring scanner stack.
2. **Post-`#1324` serial-stack order:** once `#1324` merges, **`scanner-adaptive-seam-composed`** (`-9.95 ms` already measured on i9 `d6bc8ca2`, `0.99×`, plus `-4.2..-5.8 MiB` RSS), **`u8-syms-handover-build`** (`-8.5..-10.2%` derived `S` + `-3.6..-4.4 MiB` 8W L3 relief, rescuing `#1343` on `random`/`records`), and **`#1343` (`scanner-a3-k3-pipe-u8`)** (`-5.7..-6.4%` derived `S`, `~-10%` `text`) convert near-1:1 into 8W wall.
3. **Worker compute levers still pay — in two places:** `default-a5a6a-fast-build` (`-2.6..-3.8%` derived `W`), `#1328` (`overlay_aos`, `-3.1%` CI-EPYC at `0 B` RSS), and pooled `mtf-fold` + `F8` (`-4.0..-4.6%` derived post-`lanes` `W`) pay 1:1 on default worker-bound plans (and `1W`/`4W`/`6W` even with `lanes`), AND they hold `W/8` below `~140 ms` so workers do not re-bind 8W once serial levers pull `S` down from `158 ms` to `~134–143 ms`.

## 3. Composed next-gen stack (`#1362` `M0 -> M5`) and `L6a`/`K=8` dissection (`#1365`)

Two VM audits land together with a single coherent architectural picture (both `140/140` SHA-256 bit-exact across 7 corpora):

- **Zero RSS stacking and additive composition in `M5` (#1362):** stacking `#1312` (`M1`) + `#1328` (`M2`) + pooled `mtf-fold` `d321fef8` (`M3`) + `#1343` (`M4`) + `#1311` + `#1282` (`M5`) eliminates the earlier `B4` RSS blowup completely (`8W` `lazy` HWM **`1.000×` `jcc` / `1.033×` `base`** vs `M0`), turns `runs.bz2` from a regression into **`-16.2..-23.1%` CPU and `-15%` HWM**, cuts total `W` by **`-25.8%` (`base`) / `-21.6%` (`jcc`)** and `S` by **`-8.8..-9.5%`**, shifts `W/S` from `9.47 / 8.49` to **`7.73 / 7.43`**, and lets `#1282` (`LAZY_FROM_THREADS = 4`) cut default `4W` CPU by **`-30.0..-33.5%`** and HWM by **`-55.6..-57.9%`**. Pairwise 3-way `git merge-tree` is `5/5 CLEAN` on `master` and `7/10` pairwise CLEAN (`3` trivial mechanical hunks in `Justfile`, `src/lib.rs`, `src/open.rs`).
- **Derived i9 `M5` ceiling updated with `#1368`'s calibrated prior (*derived*, anchored at `1950f77f` `4.961 GB/s`):** `#1362` brackets `M5` at **`5.18–5.42 GB/s`** (`5.52–5.80 GB/s` with `scanner-adaptive-seam-composed`). Plugging `#1368`'s calibrated `1.01×` `S` and `0.45×` `W` directly into `1950f77f`'s `S = 158.1 ms`, `W/8 = 145.0 ms`, `margin = 11.0 ms` puts `M5` at `S ≈ 143.5 ms`, `W/8 ≈ 138.8 ms` (`wall ≈ 154.5 ms`, **`~5.43 GB/s`**), and `M5 + seam` at `S ≈ 133.6 ms` vs `W/8 ≈ 138.8 ms` (`wall ≈ 149.8 ms`, **`~5.60 GB/s`**) — right at a **new 8W tie where `W` re-binds by `~5 ms`**, confirming `M5`'s residual map (`A7` `38.3%` and `A5+A6a` `29.8%` of `W`) as the next worker bottleneck once `M5 + seam` lands.
- **`#1365` cleanly kills `K=8 + prefetch` and proves why `JCC-32B` is mandatory on VM worker probes:**
  - `K=8 + edges[next]` prefetch (`A3`) lost **16/16** whole-decoder Wikidata cells vs `K=4` (`+0.55..+6.28%`): `8 × 4 KiB = 32 KiB` slabs saturate L1d, the rolled loop doubles stack stores (`33 -> 63`), and live `A6c` already warms `edges`. Discarded cleanly.
  - `L6a-on-1312` (`050acb79`, `A2`) beat `#1312` by `-3.2..-4.3%` on `base` yet flattened (`-0.3..+1.7%`) under `JCC-32B`: inserting `13.5 KiB` of fused RLE1+CRC code shifted `Walk::run<4>` from `mod 32 = 16` to `mod 32 = 0` (~`29 ms` Cascade Lake alignment shift), while in-place free-list eviction inside a `10.5 KiB` body copied similar bytes to `#1312`'s cycle permutation. **Rule reinforced:** never queue a physical i9 run on a `base`-only VM worker win; require a `JCC-32B`-verified gain (and CI `#1245` check) first. Follow-on `lanes-zero-stitch-l6a-build` stays `idea` behind `#1324`.

## 4. Tooling, continuations, and workstation queue state

- **Task-local continuations (#1356):** `frontier/tools/continuation.py` (`34` tests pass) puts one nonce-bearing CAS ref (`wait/<claim>--<action>`) per asynchronous gate on `origin` (live ready-to-acquire `56 s`, zero polling duplication, no global watcher). Its follow-up `continuation-runbook--brief-after-1341` waits on `#1341` before embedding `view` in `brief`/`status`.
- **Conceptual vocabulary (#1354):** `frontier/sensate-backpressure.md` distinguishes derived state projection from interpretation/action without adding rules or roles.
- **Workstation queue (`bench/workstation-queue.txt`):** after `v0.5.18` scaling and crossover were dequeued (#1363), `#1369` + `#1370` + `#1371` replenished the queue with **`bench-block-size-route`** (C1a calibration probe at `80dc3534`, ~3–5 min, `block-size-route-i9-probe` now `blocked` awaiting the returned i9 bundle).

## 5. Action table for the 6 open review-ready PRs (live state at `e23d8c76`)

All six open PRs currently have **0 failing and 0 pending checks**:

| Order | PR | Claim / Priority | Live state (`head`) | Exact blocker & next action |
|---:|---|---|---|---|
| **1** | **#1324** | `forward-emission-bounded-slab-rescue` (P1) | `DIRTY` (`9b438827`) | Trivial 1-line `Justfile` rebase onto `e23d8c76` (after `#1369`), then independent C6 registration review -> set ready tag & queue i9 adoption run. **Gatekeeper for the scanner stack.** |
| **2** | **#1341** | `ship-completion-pressure` (P1) | `OPEN` (`09abcd99`) | Ready for final review/merge; unblocks `continuation-runbook--brief-after-1341` (#1356 follow-up). |
| **3** | **#1311** | `segmented-fused-postwalk-rescue` | `OPEN` (`df4f0bbd`) | Pure CI path (zero i9 queue spend); fixes `runs`/`text` guards while preserving unfused `Column` on `random 1T`. Clean on `master` (#1362 §7). |
| **4** | **#1328** | `a6b-zero-rss-endpoint-build` | `OPEN` (`37a2a37a`) | `0 B` RSS `overlay_aos` (`-3.1%` open-route CI on EPYC 9V74); clean on `master` (1-line `Justfile` overlap with `#1282`). |
| **5** | **#1282** | `plan-schedule-and-lazy-threshold` | `OPEN` (`d8d0c8b5`) | `LAZY_FROM_THREADS = 4` + 5T/7T/9T pinned trims + C6 i9 recipe (`-30..-33%` CPU / `-56..-58%` HWM on default `4W` in `#1362`); C6 review before queueing i9 run. |
| **6** | **#1343** | `scanner-a3-k3-pipe-u8` | `OPEN` (`3d2afb8a`) | Rust passes review; still awaiting the 3 C6 recipe fixes requested in `#1345` (calibrated RSS gate, build-env guard + distinct binaries, formatting; holder lease to `17:20Z`). |

## 6. Implied claims & board state

Open claims stand at **9** (3 slots below the 12-claim cap), with 0 open flags and no release due. Every follow-on implied by this pass already exists on the board (`lanes-zero-stitch-l6a-build` from #1365, `default-a5a6a-fast-build`, `u8-syms-handover-build`, `scanner-adaptive-seam-composed`, and `continuation-runbook--brief-after-1341`), so no new claim is filed here. `synthesise` remains `open`.
