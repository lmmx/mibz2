# Slack-only hybrid scanning on the path to 6 GB/s: micro-structure, work-bound capture, and architectural hazards

*Recurring [`explore`](../../../frontier/claims/explore.toml) investigation (27 November 2026), cold from primary i9-10900K traces (`character-stage1` `bb72107b…2204`, `pooled-scan` `c71c4a56` / `97e397ad…8867`), `src/parallel.rs`, `src/read.rs`, `research/pooled-scan` (`1f6a578c`), and open PRs `#1031` (`scanner-seam-window`) and `#1034` (`scanner-coalesced-u8-decode`). Smooth zone only: zero `src/` edits, no frozen gate reopened (C2). **(derived)** marks discrete-event simulation outputs and conditional post-stack projections kept strictly separate from measured i9-10900K numbers per [`frontier/DECODER.md`](../../../frontier/DECODER.md).*

## Bottom line

1. **Idle slack is structurally usable (Q1):** Across the post-`#1031+#1034` scan-bound regime (`S' = 145–150 ms`, `W'/8 = 125–140 ms` at `k = 8`), worker `job_wait` slack jumps to **10.9–20.4 ms/worker** (`545–1,093` bursts/run), with **63.1%–70.2% of idle worker-ms** in bursts at least as long as one full-block scan (`~0.14–0.17 ms` median) and **41.0%–51.0%** at `>= 2×`. However, **21.9%–32.7% of idle bursts** (`43%–50%` of idle-ms) terminate on tiny runts (`n ≈ 8–30`, `3 µs` exec). Filtering helper claims to **`full_only`** via `Finder`'s candidate byte gap (`next_bit - cand_bit >= 512 B`, O(1) before decoding) cuts helper handoffs **−35% to −41%** (`105 → 68` blocks at `145/135`; `195 → 116` at `145/125`) with identical wall time.
2. **Slack-only hybrid reaches within `+4.6–5.1 ms` of the 9-core work bound `(8·W'/8 + S')/9` and eliminates `#942`'s `k = 6` regression (Q2, derived):**
   - **Zero `k = 6` regression:** Because helpers never steal while `job_queue` is non-empty, `k = 6` steals only `3–4` blocks/run (`0.5–0.7 ms`) and lands at **`0.997×–0.999×`** vs shipped serial across all regimes (eliminating `#942`'s `+10.5%..+14.9%` `k = 6` slowdown).
   - **Exact 8W `6 GB/s` (`<= 140.0 ms`) threshold:** Realized 8W wall obeys **`wall_8W ≈ (8·W'/8 + S')/9 + 4.85 ms`** (`2.62 ms` startup + `1.15 ms` validated handoff residual + `~1.08 ms` scheduling/drain tail), or **`+3.50 ms`** with a small initial chunk (`in_wait = 0.05 ms` vs `1.40 ms`). Breaking `140.0 ms` at `k = 8` requires **`(8·W'/8 + S')/9 <= 135.15 ms`** (`<= 136.50 ms` small chunk 0): e.g. **`W'/8 <= 133.9 ms` at `S' = 145 ms`** (`<= 135.4 ms` small chunk 0), or **`131.82 ms` (`+8.18 ms` margin)** at `(S' = 145, W'/8 = 125)` with `mtf-fold-l1-batch-redesign`.
   - **`÷9` overhead elasticity vs `1:1` startup sensitivity:** Serial-scanner overhead divides by `k + 1 = 9` via automatic helper stealing (`+5 ms` on scanner adds only `+0.4–0.7 ms` to 8W wall), whereas synchronous startup or global-lock stalls hit wall **1:1**.
3. **Zero-copy `Arc<Chunk>` + async feeder `Finder` eliminates both `#942` plumbing taxes (`pool_find 6.11 ms` and `pool_copy 5.27 ms`) and composes cleanly with `#1031` and `#1034` (Q3):**
   - Primary `per_k.tsv` shows `mibz2-read` is **91.6% idle** at `k = 8` (`feeder_A0 = 12.34 ms`, `feed_wait = 134.10 ms`). Running K4's `4.82 ms` AVX2 `Finder::find` (`9.84 GB/s`) on `mibz2-read` — and sending chunk 0 *before* populating its `OnceLock<Vec<Cand>>` — gives **`0.00 ms` scanner overhead AND `0.00 ms` chunk-0 startup penalty**.
   - Helpers skip the ~11/1,724 (`0.64%`) blocks straddling 4 MiB chunk seams (leaving them to `#1031`'s existing `ChannelSource` seam window), reading all other candidates zero-copy from `&chunk.bytes[start..end]` (`0.00 ms` `pool_copy`) and scanning into a worker-local `Vec<u16>` (`~200 KiB` L2-hot, zero cross-core `free_syms` coherence traffic with `#1034`).
   - **Mandatory anti-`#770` invariant:** The serial scanner MUST jump ahead to the next unclaimed block when its chain cursor hits an actively scanning helper (Policy A), verifying `(bit, end_bit, crc)` asynchronously when the helper's scan finishes; waiting synchronously behind an in-flight helper (Policy B) re-introduces `#770`'s chain-head stall (`+1.2–2.3 ms`, doubling helper steals).

---

## Primary evidence verified

- **i9-10900K `character-stage1` (`run-20261125T001135Z`, SHA-256 `bb72107b…2204`):** `tsv/per_k.tsv` (`k = 1, 6, 7, 8, 9, 10`) and 4 full per-block nanosecond traces (`blocks-r00.tsv`–`blocks-r03.tsv`, 1,724 blocks each across `k = 6..10`).
  - Measured `k = 8` 4-round means: `S_core = 174.83 ms` (`A2 + A3 + issue + resid`), `S_maint = 10.16 ms` (`S_total = 184.99 ms`), `W/8 = 185.68 ms`.
  - Per-class block service (`r0`–`r3`, `k = 8`): 932 full blocks/run (`n = 899,809–900,000`) have scan mean `0.1849 ms`, median `0.1679 ms`, p90 `0.2579 ms`, worker exec mean `1.5912 ms`, median `1.4469 ms`; 792 runts/run have scan mean `0.0032 ms` (median `0.0021 ms`), worker exec mean `0.0031 ms`.
  - Measured `mibz2-read` feeder utilization (`per_k.tsv`): `k = 6`: `11.02 ms` active / `167.04 ms` `feed_wait` (93.8% idle); `k = 8`: `12.34 ms` active / `134.10 ms` `feed_wait` (91.6% idle); `k = 9`: `13.31 / 138.62 ms` (91.2% idle); `k = 10`: `14.04 / 136.02 ms` (90.6% idle).
- **i9-10900K `pooled-scan` (`run-20261126T133034Z`, `c71c4a56`, SHA-256 `97e397ad…8867`):**
  - K4 standalone AVX2 `Finder::find` over 47,419,732 bytes (pinned CPU 4): **median `4.82 ms` `[4.81, 5.00]` (`9.842 GB/s`)**, locating all 1,724 block magics and 48 stream-end magics.
  - Prototype in-decode chain plumbing (`report.txt` lines 72–88): `pool_find` = `4.97 / 5.16 / 6.11 / 6.60 / 7.35 ms` and `pool_copy` (45.2 MiB copied into `Arc<[u8]>`) = `4.32 / 4.92 / 5.27 / 6.17 / 7.24 ms` at `k = 6, 7, 8, 9, 10` — both executed on the serial chain thread (`research/pooled-scan:src/parallel/pooled.rs:195–257`).
  - Measured `P / none`: `1.105` (`k = 6`), `1.100` (`k = 7`), `1.052` (`k = 8`), `1.000` (`k = 9`), `0.936` (`k = 10`).
- **Validated unoverlapped handoff residual (`#1035` §3):** `Real − Sim = +1.15 ms` mean (`0.94–1.36 ms` across `r0`–`r3`, matching `pooled-scan` `S` `1.16 ms` and `SC` `1.12 ms`), added explicitly to every simulated wall time below.

---

## Q1. Idle-slack micro-structure across the `(S', W'/8)` grid **(derived)**

Using the exact `src/parallel.rs` serial-scanner discrete-event simulator validated in `#1035` (`slots = 32`, `READY_BYTES = 12 MiB`, real `t_scan_start[0] + s_in_wait[0]` startup `2.62 ms` mean, `s_in_maint` removed for post-`#1031` rows), we logged every mid-stream worker `job_wait` burst across all 4 `k = 8` rounds:

| Regime `(S' / W'/8)` (`k = 8`, 4-round mean) | Sim wall (ms, excl. 1.15 ms resid) | Mid-stream idle (`ms/w`) | Idle bursts / run | Burst p25 / p50 / p75 / p90 / max (ms) | Idle-ms in bursts `>= 1×` / `>= 2×` full scan | Bursts woken by runt (% bursts / % idle-ms) |
|---|---:|---:|---:|---|---:|---:|
| Today shipped (with `in_maint`, `185 / 186`) | 194.49 | 4.69 | 156 | `0.079 / 0.156 / 0.216 / 0.501 / 3.383` | 78.3% / 52.4% | — |
| Post-`#1031` (`174 / 186`, worker-bound) | 190.42 | 0.69 | 32 | `0.069 / 0.148 / 0.193 / 0.263 / 1.567` | 65.8% / 32.5% | — |
| Post-`#1031+#1034` (`150 / 186`, worker-bound) | 189.97 | 0.35 | 15 | `0.118 / 0.143 / 0.174 / 0.302 / 1.049` | 73.3% / 40.5% | — |
| Post-`#1031+#1034+A6b` (`150 / 165`) | 169.38 | 0.44 | 21 | `0.051 / 0.133 / 0.170 / 0.223 / 1.382` | 69.8% / 38.2% | — |
| Exact tie (`150 / 150`) | 156.47 | 2.61 | 141 | `0.040 / 0.107 / 0.179 / 0.283 / 1.471` | 72.5% / 38.8% | 19.1% / 35.8% |
| No-fold stack (`150 / 140`) | 154.86 | **10.99** | 545 | `0.041 / 0.094 / 0.152 / 0.372 / 2.077` | **70.2% / 51.0%** | 21.9% / 43.6% |
| No-fold tight (`145 / 135`) | 149.77 | **10.94** | 568 | `0.039 / 0.091 / 0.144 / 0.347 / 2.008` | **69.6% / 50.9%** | 21.9% / 43.0% |
| Fold-redesign (`145 / 125`) | 149.29 | **20.38** | 1,093 | `0.049 / 0.108 / 0.140 / 0.249 / 2.527` | **63.1% / 41.0%** | 32.7% / 49.8% |
| Deep worker (`140 / 120`) | 144.21 | **20.33** | 1,125 | `0.048 / 0.107 / 0.136 / 0.243 / 2.711` | **63.2% / 40.5%** | 32.7% / 49.8% |

**Key structural conclusions from Q1:**
- **No slack exists while `W'/8 > S'`:** At `(150, 165)` mid-stream idle is only `0.44 ms/w` (21 bursts). Genuine worker slack appears abruptly as `W'/8` crosses `S'`, jumping from `2.61 ms/w` at tie to `10.94 ms/w` (`87.5` worker-ms total) at `S' − W'/8 = 10 ms` and `20.38 ms/w` (`163.0` worker-ms total) at `S' − W'/8 = 20 ms`.
- **Bursts easily absorb whole-block helper scans:** At `S' = 145–150 ms`, a median full-block scan scales to `0.139–0.144 ms`. Roughly **two-thirds of all idle time** lies in bursts `>= 1×` full scan (p90 = `0.25–0.37 ms`, max `2.0–2.7 ms`), and once an idle worker claims a full block, it scans AND immediately decodes it (`~1.2–1.4 ms` total busy interval) — converting a fragmented string of `0.09–0.15 ms` idle waits into one clean `1.3+ ms` continuous work quantum!
- **Runt wakeups explain why `full_only` beats `single`:** Under serial scanning, `22%–33%` of idle bursts end only because the scanner emits a `3 µs` runt, immediately bouncing the worker back into `job_wait`. Skipping tiny candidates (`(next_bit - cand_bit) / 8 < 512 B`) keeps the serial scanner handling all runts at `2–3 µs` each while every helper activation steals a full `~140–185 µs` scan + `~1.1–1.4 ms` decode.

---

## Q2. Realized wall vs `(k·W'/k + S')/(k+1)` work bound, `k = 6..10` safety, and the `140 ms` contour **(derived)**

### 2a. Policy comparison at `k = 8` (including `+1.15 ms` handoff residual)

In every hybrid policy below, workers ALWAYS pop `job_queue` first (Priority 1) and enter helper scanning **only when `job_queue` is empty** and the admission window (`in_flight < slots`, `ready_bytes < 12 MiB`) has room. Helpers scan and immediately decode their claimed block on the same core (`0.002 ms` coordination tax per helper block; `0.41 ms` synchronous chunk-0 `Finder` wait unless async `OnceLock` is used).

| Policy (`k = 8`, 4-round mean, `+1.15 ms` resid) | `(150 / 140)` `wb = 141.11` | `(145 / 135)` `wb = 136.11` | `(145 / 125)` `wb = 127.22` | Helper blocks / ms at `145/125` |
|---|---:|---:|---:|---:|
| Shipped serial (post-`#1031`, no `in_maint`) | 156.01 ms | 150.92 ms | 150.44 ms | 0 / 0.0 ms |
| **H1 `jump` + `full_only` (async chunk-0 `OnceLock`, 0-copy)** | **145.98 ms** (`−10.03`) | **141.06 ms** (`−9.86`) | **131.82 ms** (`−18.62`) | 116 / 20.1 ms |
| H1 `jump` + `full_only` (sync chunk-0 `+0.41 ms`) | 146.39 ms (`−9.62`) | 141.47 ms (`−9.45`) | 132.23 ms (`−18.21`) | 116 / 20.1 ms |
| H1 `jump` + `full_only` (no `t0` steal, sync chunk-0) | 146.82 ms (`−9.19`) | 141.71 ms (`−9.21`) | 132.94 ms (`−17.51`) | 114 / 19.8 ms |
| H1 `jump` + `single` (steals runts too, sync chunk-0) | 146.45 ms (`−9.56`) | 141.42 ms (`−9.50`) | 132.25 ms (`−18.20`) | 195 / 20.2 ms |
| H1 `jump` + `bundle` (runts + full, sync chunk-0) | 146.41 ms (`−9.60`) | 141.49 ms (`−9.43`) | 132.44 ms (`−18.00`) | 196 / 20.3 ms |
| H1 `jump` + `full_only` (`max_helpers = 2`) | 146.79 ms (`−9.22`) | 141.77 ms (`−9.15`) | 132.62 ms (`−17.83`) | 115 / 19.8 ms |
| H1 `jump` + `full_only` (`max_lookahead = 8`) | 146.55 ms (`−9.46`) | 141.41 ms (`−9.51`) | 132.49 ms (`−17.96`) | 112 / 19.9 ms |
| H1 `jump` + `full_only` (small chunk-0 `in_wait = 0.05 ms`) | **144.63 ms** (`−11.38`) | **139.71 ms** (`−11.21`) | **130.47 ms** (`−19.97`) | 116 / 20.1 ms |
| H1 `jump` + `full_only` (helper `0.97×` local-`syms` / `1.05×` slow) | 146.40 / 146.49 ms | 141.43 / 141.47 ms | 132.29 / 132.30 ms | 116 / 19.7–20.8 ms |
| H1 `jump` + `full_only` (`+5.0 ms` `find` on scanner) | 146.47 ms (`−9.54`) | 141.46 ms (`−9.45`) | 132.52 ms (`−17.92`) | 141 / 24.0 ms |
| H2 `jump` + `full_only` (`+11.4 ms` `find+copy` on scanner) | 147.08 ms (`−8.93`) | 142.05 ms (`−8.86`) | 133.14 ms (`−17.31`) | 175 / 28.8 ms |
| **Policy B `serial_wait` (`#770` chain-wait trap)** | 147.79 ms (`−8.22`) | 142.70 ms (`−8.21`) | 134.53 ms (`−15.92`) | **222 / 37.7 ms** |
| **Pooled-all (killed `#942` `P` model, 0-copy)** | **163.94 ms** (`+7.93`) | **158.30 ms** (`+7.38`) | 148.26 ms (`−2.18`) | 1,724 / 148.4 ms |

**Exact gap decomposition of `H1` above `wb = (8·W'/8 + S')/9`:**
- At `(145, 135)` (`wb = 136.11 ms`, `H1 = 141.06 ms`): total gap `+4.94 ms` = **`2.62 ms` startup** (`1.22 ms` thread launch + `1.40 ms` 4 MiB chunk-0 read) + **`1.15 ms` handoff residual** + **`1.18 ms` scheduling & drain tail** (`0.60 ms` at `slots = ∞`, `0.58 ms` from the 32-slot window).
- At `(145, 125)` (`wb = 127.22 ms`, `H1 = 131.82 ms`): total gap `+4.60 ms` = `2.62 ms` startup + `1.15 ms` residual + **`0.83 ms` scheduling & drain tail** (`0.57 ms` at `slots = ∞`).
- Because workers absorb scan bursts seamlessly, **the 9-core hybrid coupling gap (`+4.6–5.0 ms` above `wb`) is actually SMALLER than shipped serial's `+5.4–6.0 ms` gap above `max(S', W'/8)`** — slack-only stealing destroys the tandem-queue bottleneck!
- **Why Policy B (`serial_wait`) degrades:** Waiting synchronously when the scanner hits an in-flight helper block causes the scanner to idle for up to a full block scan (`~0.18 ms`), draining `job_queue`, triggering a cascade of extra helper steals (`116 → 222` blocks at `145/125`) and adding `+1.2–2.3 ms` to wall. Policy A (`jump` to next unclaimed candidate; verify chain order asynchronously) is mandatory.

### 2b. Thread-count sweep `k = 6, 7, 8, 9, 10`: completely avoids `#942`'s `k = 6` regression

Simulated on the real per-`k` i9-10900K traces (`blocks-r00..03.tsv` at each `k`, post-`#1031` without `in_maint`, `+1.15 ms` residual, async chunk-0 `OnceLock` `H1`):

| Scenario & `k` | `S` (ms) | `W/k` (ms) | Work bound `wb` | Shipped serial | `H1` (`t0` / `no-t0`) | `H1 / shipped` | Killed `P_all` (`P/ship`) | Helper blocks (`no-t0`) |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| **Today workers (post-`#1031`)** | | | | | | | | |
| `k = 6` | 174.4 | 235.9 | 227.1 | 240.77 | **240.21 / 240.52** | **0.998 / 0.999** | 270.06 (**1.122**) | 3 (`0.6 ms`) |
| `k = 7` | 174.0 | 203.9 | 200.2 | 208.98 | **208.32 / 208.67** | **0.997 / 0.999** | 233.96 (**1.120**) | 4 (`0.8 ms`) |
| `k = 8` | 174.8 | 185.7 | 184.5 | 191.64 | **190.58 / 191.01** | **0.994 / 0.997** | 212.88 (**1.111**) | 13 (`2.6 ms`) |
| `k = 9` | 181.2 | 172.9 | 173.7 | 187.93 | **178.87 / 179.45** | **0.952 / 0.955** | 198.34 (1.055) | 59 (`13.7 ms`) |
| `k = 10` | 181.7 | 180.0 | 180.1 | 195.61 | **187.14 / 187.49** | **0.957 / 0.958** | 203.97 (1.043) | 72 (`15.9 ms`) |
| **Projected stack (`145 / 135` @ 8W)** | | | | | | | | |
| `k = 6` | 144.7 | 171.5 | 167.7 | 176.11 | **175.64 / 175.91** | **0.997 / 0.999** | 200.57 (**1.139**) | 3 (`0.5 ms`) |
| `k = 7` | 144.3 | 148.3 | 147.8 | 153.67 | **152.67 / 153.01** | **0.993 / 0.996** | 173.84 (**1.131**) | 14 (`2.6 ms`) |
| `k = 8` | 145.0 | 135.0 | 136.1 | 151.02 | **141.05 / 141.51** | **0.934 / 0.937** | 158.30 (1.048) | 70 (`12.8 ms`) |
| `k = 9` | 150.3 | 125.7 | 128.2 | 155.25 | **132.69 / 133.13** | **0.855 / 0.858** | 147.50 (0.950) | 135 (`24.1 ms`) |
| `k = 10` | 150.7 | 130.9 | 132.7 | 157.15 | **138.59 / 139.15** | **0.882 / 0.885** | 151.46 (0.964) | 136 (`24.5 ms`) |
| **Projected stack + fold (`145 / 125` @ 8W)** | | | | | | | | |
| `k = 6` | 144.7 | 158.8 | 156.8 | 163.51 | **162.96 / 163.21** | **0.997 / 0.998** | 187.84 (**1.149**) | 4 (`0.7 ms`) |
| `k = 7` | 144.3 | 137.3 | 138.2 | 149.13 | **142.62 / 143.00** | **0.956 / 0.959** | 162.82 (1.092) | 51 (`9.6 ms`) |
| `k = 8` | 145.0 | 125.0 | 127.2 | 150.51 | **132.00 / 132.29** | **0.877 / 0.879** | 148.26 (0.985) | 113 (`19.8 ms`) |
| `k = 9` | 150.3 | 116.4 | 119.8 | 155.13 | **124.19 / 124.60** | **0.801 / 0.803** | 138.15 (0.891) | 184 (`31.9 ms`) |
| `k = 10` | 150.7 | 121.2 | 123.8 | 156.09 | **129.53 / 130.07** | **0.830 / 0.833** | 141.72 (0.908) | 178 (`31.3 ms`) |

*Note:* Even without a thread-count cutoff, slack-only stealing is **strictly non-regressing (`<= 0.999×`) at every `k = 6..10`** once chunk-0 `Finder` runs asynchronously via `OnceLock`. (At `k = 1..5`, keeping the shipped serial path avoids spawning the finder entirely, just as `#942` kept `k = 1` shipped.) Notice also that at `k = 9` on today's i9 workers, `H1` already wins `−8.5..−9.1 ms` (`0.952×`), and after worker improvements (`145/135` @ 8W) `k = 9` hits **`132.7 ms` (`6.32 GB/s`)** — **−22.6 ms (`−14.5%`)** faster than shipped serial!

### 2c. Exact `6 GB/s` (`140.0 ms`) contour & overhead budget at `k = 8`

With async chunk-0 `OnceLock` `Finder` (`+1.15 ms` handoff residual included):

| `S'` (ms) | `W'/8` (ms) | Work bound `wb` | Shipped serial | `H1` (4 MiB chunk 0) | `H1` (small chunk 0) | `H1 − wb` | Wall margin to `140.0 ms` (`4M / small`) |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 155 | 135 | 137.22 | 160.61 | 141.95 | 140.60 | +4.73 | `−1.95 / −0.60 ms` |
| 155 | 133 | 135.44 | 160.53 | 140.18 | **138.83** | +4.73 | `−0.18 / +1.17 ms` |
| 155 | 130 | 132.78 | 160.42 | **137.60** | **136.25** | +4.82 | **`+2.40 / +3.75 ms`** |
| 150 | 135 | 136.67 | 155.72 | 141.54 | 140.19 | +4.87 | `−1.54 / −0.19 ms` |
| 150 | 133 | 134.89 | 155.64 | **139.62** | **138.27** | +4.73 | **`+0.38 / +1.73 ms`** |
| 150 | 130 | 132.22 | 155.53 | **136.89** | **135.55** | +4.67 | **`+3.11 / +4.45 ms`** |
| 150 | 125 | 127.78 | 155.35 | **132.59** | **131.24** | +4.81 | **`+7.41 / +8.76 ms`** |
| 145 | 135 | 136.11 | 150.92 | 141.06 | **139.71** | +4.94 | `−1.06 / +0.29 ms` |
| 145 | 133 | 134.33 | 150.80 | **139.11** | **137.77** | +4.78 | **`+0.89 / +2.23 ms`** |
| 145 | 130 | 131.67 | 150.64 | **136.40** | **135.05** | +4.73 | **`+3.60 / +4.95 ms`** |
| 145 | 125 | 127.22 | 150.44 | **131.82** | **130.47** | +4.60 | **`+8.18 / +9.53 ms`** |
| 140 | 135 | 135.56 | 146.19 | 140.49 | **139.14** | +4.93 | `−0.49 / +0.86 ms` |
| 140 | 133 | 133.78 | 146.04 | **138.70** | **137.35** | +4.92 | **`+1.30 / +2.65 ms`** |

**Overhead budget rule:**
- **Linear closed form:** `wall_8W ≈ (8·W'/8 + S')/9 + 4.85 ms` (4 MiB chunk 0) or `+ 3.50 ms` (small chunk 0).
- **Direct 1:1 budget** (startup delay, lock contention, or reader stalls): at `(145, 130)` there is `+3.6 ms` (`+5.0 ms` small chunk 0) before hitting `140.0 ms`; at `(145, 125)` there is **`+8.2 ms` (`+9.5 ms`)**.
- **Scanner-side `÷9` budget:** Serial-scanner-only overhead (such as `find_blocks`) is attenuated by `9×` (`+5 ms` on scanner costs only `+0.4–0.7 ms` wall), so even a fallback design that kept `Finder` on the scanner would stay well inside budget at `W'/8 <= 130 ms` — though offloading `Finder` to the idle feeder thread is strictly superior at every `k`.

---

## Q3. Architectural hazard check & composition with `#1031` and `#1034`

### 3a. Offloading `find_blocks` onto `mibz2-read` with zero startup penalty

- In `src/read.rs:245–278`, `mibz2-read` reads 4 MiB chunks into recycled `Vec<u8>` buffers (`d.spare()`) and blocks on `d.write(buf)` for **`134.10 ms` out of `146.44 ms` (91.6% idle)** at `k = 8`.
- Wrapping each chunk in an immutable `Arc<Chunk>`:
  ```rust
  struct Chunk {
      base: u64,
      bytes: Vec<u8>,
      cands: OnceLock<Vec<Cand>>, // bit offsets + magic-kind within this chunk
  }
  ```
  - **Chunk 0 (zero startup tax):** `mibz2-read` pushes `Arc::clone(&chunk0)` to `in_tx` **before** running `Finder::find(&chunk0.bytes)` (`0.41 ms` for 4 MiB at `9.84 GB/s`), then populates `chunk0.cands` via `OnceLock::set` immediately afterward. The serial scanner starts block 0 with **`0.00 ms` added `in_wait`**; by `t0 + 0.41 ms` (~2 full blocks scanned, before workers ever run dry), `chunk0.cands` is ready for helpers.
  - **Chunks 1..N:** `mibz2-read` carries the last 7 bytes of chunk `j − 1` as overlap (matching `pooled_scan.rs:191`'s `len - 7` rule so zero cross-chunk magics are missed), runs `Finder::find` while the 4 MiB buffer is L2/L3-hot from `read_full`, sets `cands`, and pushes `Arc<Chunk>`. Total feeder active time rises from `12.34 ms` to `~17.2 ms` — still **>85% idle** against a `130–140 ms` decode!
  - **Buffer recycling (`#1031` compatibility):** When the serial scanner retires a chunk (`ChannelSource::retire` in `#1031`) or the last helper drops its `Arc<Chunk>`, `Arc::try_unwrap(chunk)` recovers `chunk.bytes` (`Vec<u8>`) and pushes it back to `st.free_in` — preserving `#1031`'s zero-reallocation recycling without modifying `ChannelSource`'s hot path.

### 3b. Zero-copy helper slicing + `#1031` seam window (`pool_copy`: `5.27 ms → 0.00 ms`)

- In `research/pooled-scan:src/parallel/pooled.rs:236`, the chain thread copied `45.2 MiB` into per-block `Arc<[u8]>` buffers (`pool_copy = 5.27 ms` at `k = 8`, `7.24 ms` at `k = 10`).
- With `Arc<Chunk>`, any candidate `i` whose next magic `i + 1` (plus `MARGIN = 16 B`) lies inside the **same** 4 MiB chunk is sliced directly as `&chunk.bytes[start..end]` with **0 bytes copied and 0 allocations**.
- Only the **~11 blocks out of 1,724 (0.64%)** that straddle a 4 MiB chunk boundary cross chunks. Helpers simply **skip seam-straddling candidates** (`last > chunk.bytes.len()`), leaving those ~11 blocks to the serial scanner's existing `#1031` `seam` buffer! That eliminates `100%` of helper copy logic and `100%` of `pool_copy` (`5.27 ms → 0.00 ms`), while costing `< 0.05 ms` of missed helper opportunity across the entire stream.

### 3c. Composition with `#1034` (`scanner-coalesced-u8-decode`) and `Syms` pool

- **Zero cross-core `free_syms` ping-pong:** In the shipped pipeline, `#1035` §2(c) measured `+4.46 ms` (`+3.0%` cycles) on the serial scanner from cross-core `free_syms` coherence handoff. Every helper-claimed block runs `block::read_item_with_scratch` (using the exact same coalesced/BMI2 decoder and `Kernels` from `#1034`) into a **worker-local `ScanScratch` + `Vec<u16>`** (`~200 KiB` L2-hot), then immediately executes `runs::front` / `mtf` + `OpenCells` + `RLE1+CRC` on the **same core**, never touching `Jobs` or `free_syms`.
- **Memory footprint (K3 safety):** Only workers that actually enter helper mode lazily allocate their worker-local `ScanScratch` + `Vec<u16>` (`~200 KiB/worker` → `<= 1.6 MiB` total at `k = 8`, `< 0.25%` of `668 MiB` RSS; `0 bytes` allocated below the knee when helpers never trigger).

### 3d. Exactness, non-level-9, and adversarial-input guards

1. **Level-9 gate:** Helpers remain dormant until the serial scanner parses the active stream's header and sets `stream_level == 9` (resetting at each stream end magic). Level 1–8 streams incur `0` speculative scans and `0` overhead.
2. **Exact chain acceptance (`Scanner::next_hinted`):** The serial scanner maintains the exact bit cursor `at` and accepts a helper's completed `Meta { bit, end_bit, crc }` ONLY when `meta.bit == at` and `stream_level == 9` and `meta.ok`. Any false magic, missed magic, or erroring candidate is ignored/discarded and decoded sequentially by the serial scanner, guaranteeing exact `Error { kind, block, bit }`.
3. **Bounded lookahead (`max_lookahead = 8..16`):** Simulation (§2a) shows `max_lookahead = 8` matches unbounded `slots = 32` within `0.00–0.26 ms`, while strictly bounding worst-case speculative work and memory on adversarial inputs with dense false magics.
4. **No `#770` serialization:** As proved in §2a, the serial scanner MUST skip any block currently in `HelperScanning` state and claim the next unclaimed candidate inside `[taken, taken + slots)` (Policy A), never blocking on a helper's in-flight scan.

---

## Implication for `pooled-scan-hybrid`

`frontier/claims/pooled-scan-hybrid.toml` remains cleanly gated behind worker improvements (`S'(8) <= 165 ms` per `DECODER.md`; `stop_test` untouched), with three concrete design constraints now quantified ahead of writing `pooled-scan/hybrid-registration.md`:

1. **Trigger refinement:** Against post-`#1031+#1034` serial scan (`S' ≈ 145–150 ms`), 8-worker gains turn on once `W'/8 < S'` (`~−10 ms` / `−6.5%` at `W'/8 = 135–140 ms`, breaking `140 ms` / `6 GB/s` at `W'/8 <= 133–135 ms`, and reaching `~132 ms` / `−18.6 ms` at `W'/8 = 125 ms`).
2. **Design architecture for `hybrid-registration.md`:** Specify (a) `mibz2-read` feeder-offloaded AVX2 `Finder` with async `OnceLock` on chunk 0 (`0 ms` scanner/startup tax), (b) zero-copy `Arc<Chunk>` slicing with seam-straddling blocks left to `#1031`'s seam window (`0 ms` `pool_copy`), (c) `full_only` candidate filter (`gap >= 512 B`), (d) Policy A jump-ahead scanner with asynchronous chain verification (never `#770`'s `serial_wait`), (e) bounded `max_lookahead = 8..16`, and (f) worker-local `ScanScratch` + `Vec<u16>` running `#1034`'s exact `read_item_with_scratch` path end-to-end on one core.
3. **Kill test alignment:** With zero `k = 6` regression (`0.997×–0.999×` simulated), `hybrid-registration.md` can enforce strict non-regression across `k = 1, 6, 7` alongside a direct wall-time gate at `k = 8` (and `k = 9, 10`) once `W'/8` crosses below `S'`.
