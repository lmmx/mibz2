# Controlled A/A probe and 24-run CI dissection of `pattern` at four threads

*Probe (`ci-pattern-4t-arm-bias`), 27 November 2026. Primary evidence: static audit of `.github/workflows/bench.yml` and `examples/ab.rs`, ELF binary diff at `d3bbe33f`, a 4-arm controlled A/A factorial run on this VM, and all 24 non-scanner `bench-pr-*` CI artifacts (18 in-sample from `#1108` plus 6 subsequent out-of-sample strict-null PR runs, plus `#1034` run 45947633032).*

## Verdict (frozen `stop_test`)

**`NOT REPRODUCED`.** Both the plain A/A (same binary under both arm labels: Arms B and C) and the two-build A/A (same commit `d3bbe33f` built twice as CI builds `bin/ab` and `.base-src` -> `base/ab`: Arms A and D) produce 95% intervals cleanly containing `1` on `pattern` at 4 threads, with zero 10-round false flags. Out-of-sample verification on 6 subsequent strict-null CI runs (60 paired rounds on actual 4-vCPU CI runners) also gives `wall = 0.9970` (run-cluster 95% CI `0.9865–1.0075`, `0/6` flagged), and all 11 strict-null CI runs combined (110 pairs) give `1.0048` (`0.9853–1.0261`).

The 24-run (240-pair) CI dissection uncovers two concrete mechanisms behind `#1108`'s observation:

1. **A symmetric bimodal peak-RSS / scheduling regime (`R² = 0.706` on strict-null):** `pattern.bz2` (`1,116 B -> 19.6 MB`, 22 blocks) runs 7 threads (`ab` + 4 workers + `mibz2-scan` + `sampler`) on 4 vCPUs / 2 physical cores (`cpus = "all"`). Peak RSS jumps from a low mode (`36–38 MB`) to a high mode (`>= 39 MB`, up to `45 MB`, `+4–8 MB`) whenever the consumer lags workers and buffers queue up. Crossing regimes moves wall time by `+10.8%` (`h_hi_b_lo`) or `−9.8%` (`h_lo_b_hi`) symmetrically while moving CPU in the opposite direction (`−3.1%` / `+2.8%`). Within `both_low` pairs (`n = 50` strict-null), `head/base = 1.0001` (`0.9869–1.0135`); OLS RSS-adjusted `head/base` wall is `0.9964` (`0.9872–1.0057`) on strict-null and `1.0001` (`0.9920–1.0082`) across all 24 runs.
2. **A genuine Round-0 (`r0`) cold-start asymmetry in `ab run`:** `ab run` has no warm-up round and always runs arm 0 (`mibz2`, head) first in `r0` (`arm_order(0, 2) = [0, 1]`). Across all 24 CI runs, `r0` alone shows `wall = 1.0605` (`1.0027–1.1216`), `rss = 1.0542`, and high-RSS (`>= 39 MB`) in `12/24` head vs `3/24` base runs (Fisher exact `p = 0.0114`; strict-null `5/11` vs `0/11`, `p = 0.0351`; out-of-sample strict-null `3/6` vs `0/6`). In `r1..r9` (dropping only `r0`), all 11 strict-null CI runs (`99` pairs) give `wall = 1.0012` (`0.9831–1.0196`) and `33/99` vs `33/99` high-RSS (`p = 1.0000`), and even `#1108`'s original 18-run run-cluster 95% CI drops to `0.9956–1.0303` (containing `1`).

No change to the bench, its corpora, guards or statistics follows from this claim alone (`PERFORMANCE-POLICY` §2, measurement changes).

## 1. Static asymmetry audit (`bench.yml` and `examples/ab.rs`)

Every structural difference between the head arm (`mibz2`) and base arm (`mibz2@base/ab`) in CI:

- **Subprocess spawning (`examples/ab.rs:2489-2501`):** Both arms spawn a child `Command` with identical args `["child", "mibz2", &t, file]` (`arm.split_once('@')` strips `@path` before passing `name = "mibz2"`). Neither runs in-process.
- **Executable path & inode:** Head (`mibz2`, no `@`) spawns `std::env::current_exe()` (absolute path `.../bin/ab`, same inode as parent). Base (`mibz2@base/ab`) spawns relative `"base/ab"` (separate file/inode, shorter `argv[0]`).
- **Build directory (`bench.yml:147-195`):** Head builds in `$GITHUB_WORKSPACE`; base builds in detached worktree `$GITHUB_WORKSPACE/.base-src` (`+10` chars on source path) with `CARGO_TARGET_DIR="$GITHUB_WORKSPACE/target"` and explicit `RUSTUP_TOOLCHAIN=1.99.1` (matching `rust-toolchain.toml`).
- **Round order (`examples/ab.rs:2659-2669`):** Over `rounds = 10`, `arm_order(round, 2)` produces 5 head-first (`r0, r3, r4, r7, r8`) and 5 base-first (`r1, r2, r5, r6, r9`) rounds, balanced overall, but **`r0` is always head-first with no warm-up round**.
- **Observers (`Census::watch`, `Threads::watch`, `rusage`):** Identical for both arms; no branch depends on `@` or `exe`.

## 2. Binary identity check (`bin/ab` vs `base/ab` at `d3bbe33f`)

Building commit `d3bbe33f` twice using `bench.yml`'s exact commands (workspace root `/home/antigravity/wt-review-1110` -> `bin/ab`; detached worktree `/tmp/ci_bias/.base-src` -> `base/ab`, sharing `CARGO_TARGET_DIR`):

- **SHA-256:** `bin/ab` (`a41f6142…`, `10,774,160` bytes) vs `base/ab` (`5d1dae63…`, `10,774,152` bytes, `−8` bytes).
- **ELF section sizes (`readelf -SW`):** Every section has identical VMA and size except `.debug_str` (`0x27c519` vs `0x27c50f`, `−10` bytes from the 10-char path length difference).
- **`.text` layout (`nm -n`, `objcopy`, `objdump`):** **1,606 / 1,606** text symbols sit at identical addresses (`diff_addr = 0`). The `1,679` differing bytes inside `.text` (1,654,406 B total) are exclusively RIP-relative immediates referencing reordered `.rodata` string addresses (verified at `.text+2237`: `lea -0x38edd(%rip),%rdi` vs `lea -0x38ee7(%rip),%rdi`, cold panic string shifted by 10 B). No instruction selection or function alignment changes.

## 3. Controlled 4-arm A/A factorial experiments (local VM, `d3bbe33f`)

Executed in `/tmp/ci_bias` with `SCALE = 4` corpora (`bin/ab gen`, `pattern.bz2`: `1,116 B -> 19,600,000 B`; `text.bz2`: `2,762,177 B -> 17,991,040 B`), `--spread --settle 0.10 --max-busy 0.50` (`copy/ab` is a byte-identical `cp` of `bin/ab` on a separate inode).

*Topology caveat:* This VM has **2 vCPUs / 2 physical cores** (Xeon 2.30 GHz, no PMU), whereas CI runners have **4 vCPUs on 2 physical cores**. At `t = 4`, `pattern` is 2× oversubscribed here (`~52 ms` median wall vs `15–24 ms` on CI), so local `t = 4` tests binary/label/inode mechanisms cleanly but does not reproduce CI's 15–20 ms wall–RSS correlation (`corr(w, r) ≈ 0` locally while the same `35–44 MB` RSS spread occurs).

| Experiment | Arms (head vs base) | Isolates | `n` | Wall GM (95% t-CI) | RSS GM (95% t-CI) | CPU GM | High RSS (`>=39 MB`) H/B | 10-rnd flags |
|---|---|---|--:|--:|--:|--:|--:|--:|
| **Arm A** (two-build CI) | `mibz2` vs `mibz2@base/ab` | All CI asymmetries | 30 | **0.9910** (`0.9531–1.0303`) | 0.9911 (`0.9654–1.0175`) | 0.9847 | 9 / 14 | 0S / 0F (of 3) |
| **Arm B** (plain A/A, copy) | `mibz2` vs `mibz2@copy/ab` | Label + inode + `argv[0]` | 30 | **0.9709** (`0.9297–1.0140`) | 1.0073 (`0.9720–1.0439`) | 0.9771 | 12 / 12 | 0S / 0F (of 3) |
| **Arm C** (plain A/A, same ino) | `mibz2` vs `mibz2@bin/ab` | Bare vs `@path` + `argv[0]` | 20 | **0.9784** (`0.9115–1.0503`) | 1.0175 (`0.9707–1.0665`) | 0.9734 | 8 / 5 | 0S / 0F (of 2) |
| **Arm D** (sym two-build) | `mibz2@bin/ab` vs `mibz2@base/ab` | Pure `.base-src` build | 20 | **0.9678** (`0.9007–1.0400`) | 0.9833 (`0.9578–1.0095`) | 0.9821 | 6 / 8 | 0S / 0F (of 2) |
| Control A (`pattern` 1T) | `mibz2` vs `mibz2@base/ab` | Sanity control | 15 | 0.9671 (`0.9095–1.0284`) | 1.0051 (`1.0005–1.0098`) | 0.9952 | 0 / 0 | 0S / 0F (of 1) |
| Control A (`pattern` 2T) | `mibz2` vs `mibz2@base/ab` | Sanity control | 15 | 0.9630 (`0.8536–1.0863`) | 0.9851 (`0.9465–1.0252`) | 0.9964 | 0 / 0 | 0S / 0F (of 1) |
| Control A (`text` 4T) | `mibz2` vs `mibz2@base/ab` | Sanity control | 15 | 1.0045 (`0.8963–1.1258`) | 0.9850 (`0.9584–1.0124`) | 0.9804 | 15 / 15 | 0S / 0F (of 1) |

None of binary identity, arm label, inode sharing, or `argv[0]` reproduces a `>= 1.010` head-arm penalty; every 95% interval (both t-interval and 2000-resample bootstrap) contains `1`.

## 4. Dissection of 24 non-scanner CI runs (240 pairs on 4-vCPU runners)

To inspect the bias directly on CI's 4-vCPU / 2-core runners, we downloaded all 18 non-scanner `bench-pr-*` artifacts from `#1108` (5 strict-null + 13 non-scanner decoder edits) plus **6 subsequent out-of-sample strict-null PR runs** (`7f8e5c0b`, `5aeb37a4`, `911fa47e`, `01181d78`, `2bf2d40a`, and `01849b50` whose code is feature-gated behind `forward-anchor-lanes` and absent from `bench.yml`'s default build). Note: `#1108` described its 20 runs as all on Xeon 8573C; inspecting `result.json` shows the 24 runs span five 4-vCPU / 2-core runner CPUs (`AMD EPYC 7763`, `9V45`, `9V74`, `Intel Xeon 8573C`, `6975P-C`).

### 4.1 In-sample vs out-of-sample pools (`pattern` 4T)

| Pool | Runs (pairs) | Raw wall GM (run-boot 95% CI) | Raw RSS GM | High RSS (`>=39 MB`) H/B | Flags S/F | OLS `R²` (slope) | RSS-adj wall intercept (95% CI) | `both_low` `<39 MB` wall (n) |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| `#1108` 18 non-scanner | 18 (180) | 1.0186 (`1.0021–1.0342`) | 1.0196 | 71 / 48 | 3 / 0 | 0.540 (0.850) | **1.0019** (`0.9920–1.0120`) | 0.9988 (80) |
| `#1108` 5 strict-null | 5 (50) | 1.0142 (`0.9675–1.0564`) | 1.0179 | 20 / 16 | 2 / 0 | 0.733 (0.907) | **0.9981** (`0.9842–1.0121`) | 1.0131 (19) |
| **OOS 6 strict-null (new)** | **6 (60)** | **0.9970** (`0.9865–1.0075`) | **1.0028** | **18 / 17** | **0 / 0** | 0.676 (0.852) | **0.9947** (`0.9822–1.0073`) | 0.9922 (31) |
| **All 11 strict-null** | **11 (110)** | **1.0048** (`0.9853–1.0261`) | **1.0096** | **38 / 33** | **2 / 0** | **0.706 (0.881)** | **0.9964** (`0.9872–1.0057`) | **1.0001** (50) |
| All 24 non-scanner | 24 (240) | 1.0132 (`1.0004–1.0255`) | 1.0154 | 89 / 65 | 3 / 0 | 0.570 (0.853) | **1.0001** (`0.9920–1.0082`) | 0.9969 (111) |

### 4.2 Bimodal RSS regime symmetry (all 11 strict-null CI runs, 110 pairs)

Across all 480 individual `pattern` 4T CI runs, peak RSS clusters strongly at `36–38 MB` (`237/480`) with a heavy upper tail extending to `44–45 MB` (`154/480 >= 39 MB`), giving `pattern` 4T the highest per-round log-wall SD (`0.094`) and strongest wall–RSS correlation (`r = +0.848`) of all 7 corpora at 4T (next highest `corr`: `text` `+0.427`; `onebyte` `−0.048`, `records` `+0.012`). Stratifying the 110 strict-null pairs by `rss_kb >= 39,000`:

| Regime pair (`>= 39 MB`) | `n` | Wall H/B GM (95% CI) | RSS H/B GM | CPU H/B GM |
|---|--:|--:|--:|--:|
| `both_low` | 50 | **1.0001** (`0.9869–1.0135`) | 0.9980 | 1.0055 |
| `both_high` | 11 | **1.0035** (`0.9570–1.0524`) | 1.0019 | 1.0112 |
| `h_hi_b_lo` | 27 | **1.1075** (`1.0791–1.1366`) | 1.1302 | 0.9693 |
| `h_lo_b_hi` | 22 | **0.9019** (`0.8772–0.9272`) | 0.9060 | 1.0280 |

The regime flip is completely symmetric (`1.1075 × 0.9019 = 0.9989`): whichever arm lands in the high-RSS regime pays `~+10.5%` wall for `~−3.0%` CPU (consumer preempted behind 4 workers on 2 cores -> `ready` buffers accumulate `+4–8 MB` -> extra zero-page faults and `runq` wait).

### 4.3 Round-0 (`r0`) cold-start asymmetry vs `r1..r9`

Splitting Round 0 (always head-first, no warm-up round) from Rounds 1–9:

| Pool | Subset | `n` | Wall GM (95% CI) | RSS GM | High RSS H/B | Fisher `p` |
|---|---|--:|--:|--:|--:|--:|
| All 24 non-scanner | `r0` only | 24 | **1.0605** (`1.0027–1.1216`) | **1.0542** | **12 / 3** | **0.0114** |
| All 24 non-scanner | `r1..r9` | 216 | 1.0081 (run-boot `0.9938–1.0221`) | 1.0112 | 77 / 62 | 0.1492 |
| `#1108` 18 non-scanner | `r0` only | 18 | 1.0691 (`0.9931–1.1509`) | 1.0491 | 9 / 3 | 0.0750 |
| `#1108` 18 non-scanner | `r1..r9` | 162 | 1.0132 (run-boot `0.9956–1.0303`) | 1.0164 | 62 / 45 | 0.0602 |
| **All 11 strict-null** | **`r0` only** | **11** | **1.0381** (`0.9869–1.0921`) | **1.0594** | **5 / 0** | **0.0351** |
| **All 11 strict-null** | **`r1..r9`** | **99** | **1.0012** (`0.9831–1.0196`) | **1.0042** | **33 / 33** | **1.0000** |
| OOS 6 strict-null | `r0` only | 6 | 1.0350 (`0.9927–1.0791`) | 1.0697 | 3 / 0 | 0.1818 |
| OOS 6 strict-null | `r1..r9` | 54 | 0.9929 (`0.9694–1.0170`) | 0.9956 | 15 / 17 | 0.8339 |

Across all 21 cells in `r0` (24 runs), `pattern` 4T is the only cell with a statistically significant Round-0 penalty (`1.0605`, present across 4 distinct CPU models: `EPYC 7763` `1.021`, `9V45` `1.153`, `9V74` `1.084`, `Xeon 6975P-C` `1.067`). `r0` `pattern` 4T is the first invocation in `ab run` whose anonymous RSS exceeds `~34 MB` (`onebyte` peaks at `30–34 MB`, `pattern` 1T/2T at `17–24 MB`), so arm 0 (`head`) hits cold OS page allocation right after `t=1`/`t=2` leave two vCPUs idle, triggering the high-RSS queue-backup regime (`12/24` vs `3/24`); once warm (`r1..r9`), strict-null is `33/99` vs `33/99` (`wall = 1.0012`). Over a 10-round geometric mean, `+6%` in `r0` contributes `~+0.6%` (`1.0605^0.1 = 1.0059`); the remaining gap in `#1108`'s 18-run window came from finite-sample spread in the heavy-tailed bimodal RSS regime (especially `EPYC 9V45/9V74` decoder-edit runs) and 21-cell selection.

## 5. Direct evidence for `#1034` (`782f4feb`, run 45947633032)

Inspecting `runs.jsonl` of `#1034`'s exact-head CI run `45947633032` (`Intel Xeon Platinum 8573C`, `pattern` 4T):

- **High-RSS regime counts (`>= 39 MB`):** Head landed in the high-RSS regime on **`5/10` rounds** (`r0: 41,572 kB`, `r2: 39,564`, `r4: 42,248`, `r6: 43,140`, `r9: 43,648`) versus **`0/10` base rounds** (all `35,704–38,048 kB`), yielding raw `wall = 1.0734`, `rss = 1.0794`, `cpu = 0.978` (`−2.2%`).
- **RSS-adjusted wall ratio:** Adjusting each paired round by the 11-run strict-null OLS wall–RSS slope (`0.881`, `R² = 0.706`) yields **`1.0035`** (95% t-CI **`0.9725–1.0355`**). That matches the high-RSS signature (`+7.9%` RSS -> `+7.0%` wall with `−2.2%` CPU).

This note records evidence only and does not resolve `frontier/flags/2026-11-27-scanner-coalesced-ci-acceptance.md`.
