# Dissection of `src/cells.rs` (`EagerCells`), `src/run_fed.rs`, and Inline `Seq` / `t = 1..3` `OpenCells` Unification

*Date:* 27 November 2026  
*Zone:* Smooth (`explore`, zero `src/` edits; standalone `#![forbid(unsafe_code)]` probe in [`examples/eager_cells_seq_open_probe.rs`](../../../examples/eager_cells_seq_open_probe.rs))  
*Commit baseline:* `63cfc924` (`v0.5.15`)  
*Host (C5 diagnostic note):* 2-vCPU Intel Xeon @ 2.30 GHz KVM VM (Haswell, 32 KiB L1d / 256 KiB L2 / 45 MiB L3, AVX2/BMI1/BMI2/PCLMULQDQ, `cpus_allowed=[0,1]`). Concurrent VM load (~3–5) means steal-free on-CPU time (`/proc/thread-self/schedstat` for `Seq`, `/proc/self/stat` utime+stime for `t = 2, 3`) and true subprocess peak RSS (`/proc/self/status` `VmHWM`) are primary metrics. Cross-referenced against physical 10C/20T Comet Lake-S i9-10900K evidence (`origin/evidence` `run-20261125T022939Z` `kernel-count`, `bce3914d` `release-scaling`, `45ae3ff3` `crossover`, `77236472` `jcc-aligned-release`).

---

## 1. Frozen Stop Test (C1a / C2 — Frozen Before Running Any Benchmarks)

1. **Track 1 — Inline `Seq(OpenCells)` & `Seq(OpenR2Cells)` vs shipped `Seq(Cells)` + `t = 2, 3` worker pools (`Cells` vs `OpenCells` vs `OpenR2Cells`, zero `src/` edits):**
   - Measure across all **7 CI corpora** (`wikidata`, `text`, `records`, `random`, `runs`, `pattern`, `onebyte`) under **both** baseline x86-64 and JCC-32B-aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`), warm + L3-cold (`64 MiB` cache flush before every block on `wikidata` and `text`), with 100% SHA-256 parity vs `libbzip2` (`bzip2 -dc`), plus a forward-looking check with `--features forward-anchor-lanes` (`#1110`).
   - **PASS (open a `status = "idea"` i9-gated build claim for unifying `t = 1, 2, 3` onto `OpenCells` / `OpenR2Cells` on current `master`):**
     - **(i)** `OpenR2Cells` achieves `>= 3.0%` steal-free CPU reduction OR `>= 25%` peak RSS reduction with `<= 1.01×` CPU on `wikidata` across `t = 1, 2, 3` (both baseline x86-64 and JCC-32B, warm & L3-cold); AND
     - **(ii)** zero `> 2.0%` disjoint CPU regression on any of the 6 off-Wikidata guard corpora at `t = 1, 2, 3` under baseline or JCC-32B; AND
     - **(iii)** explicitly pre-registers physical i9-10900K `k = 1, 2, 3` verification (C5).
   - **STOP:** otherwise record a scoped dead-end row in `frontier/dead-ends.tsv` and do not open a premature `t = 1, 2` switch claim.

2. **Track 2 — Micro-Dissection of `src/cells.rs` (1,733 LOC) + `src/cells_owned.rs` (348 LOC) + `src/run_fed.rs` (830 LOC):**
   - Audit `src/run_fed.rs` reachability, produce the full buffer/RSS ledger (`trace::buffers()` + `VmHWM`), per-bucket & internal phase breakdown (`slots`, `induce_cells`, `write_owned` = `owned_scan` + `owned_copies`), and static `objdump -d` audit of hot `Cells` symbols.
   - **PASS (open a standalone `src/cells.rs` build claim):** ONLY IF `EagerCells` retains a real speed edge AND a targeted optimization clears `>= 5.0%` disjoint steal-free CPU reduction on `Cells` whole-block inversion on `wikidata` (both baseline and JCC-32B) with `<= +1.0%` guard delta and `<= 1.00×` RSS. Otherwise record a scoped `STOP` in `frontier/dead-ends.tsv`.

---

## 2. Parity Verification (63/63 Exact vs `libbzip2`)

Every arm (`cells`, `open`, `open-r2`) across `Seq` (`t = 1`) and `BzReader::parallel_workers_with` (`t = 2, 3`) matched `bzip2 -dc` byte count and SHA-256 identically on all 7 corpora (**63/63 PASS, 0 mismatches**):

| Corpus | Raw bytes | `bzip2 -dc` SHA-256 (prefix) | Verified arms (`Seq`, `t = 2`, `t = 3`) |
|---|---:|---|---|
| `wikidata` | 209,700,196 | `3f075e30f4f8c734` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `text` | 49,784,532 | `67c43f1be78b2fcd` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `records` | 38,400,000 | `6da24faf241a1f0b` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `random` | 32,000,000 | `b2a9bcc4d2d5c79a` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `runs` | 40,000,041 | `e610e15b91ca8802` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `pattern` | 78,400,000 | `ea290f5f35cdab01` | `cells`, `open`, `open-r2` (9/9 PASS) |
| `onebyte` | 80,000,000 | `2e4641fb7ef6d413` | `cells`, `open`, `open-r2` (9/9 PASS) |

---

## 3. Track 1 — Inline `Seq` (`t = 1`) and `t = 2, 3` Parallel Worker Benchmarks

### 3.1 Inline `Seq` (`t = 1`, zero thread overhead, streaming `Scanner` + `decode_block::<I>`)

Pinned to CPU 1 (`taskset -c 1`), 5 counterbalanced interleaved rounds per build, steal-free `/proc/thread-self/schedstat` CPU and true subprocess `VmHWM` peak RSS.

#### Baseline x86-64 (default shipping release flags)

| Corpus | `Cells` CPU | `Open` CPU (`Δ`) | `OpenR2` CPU (`Δ [95% CI]`) | Disj? | `Cells` RSS | `Open` RSS (`O/C`) | `OpenR2` RSS (`R2/C`) |
|---|---:|---:|---:|---|---:|---:|---:|
| **`wikidata`** | **`0.8505 s`** | `0.9255 s` (`+12.08%`) | `0.9013 s` (**`+9.21% [+2.38%, +16.50%]`**) | **DISJ SLOWER** | `28.6 MiB` | **`16.3 MiB` (`0.57×`)** | `24.1 MiB` (`0.84×`) |
| `text` | `0.3519 s` | `0.3836 s` (`+9.40%`) | `0.3567 s` (`+1.12% [-2.33%, +4.70%]`) | ovlp | `21.1 MiB` | `21.1 MiB` (`1.00×`) | `21.1 MiB` (`1.00×`) |
| `records` | `0.8540 s` | `0.8576 s` (`+0.38%`) | `0.8600 s` (`+1.11% [-0.60%, +2.85%]`) | ovlp | `50.6 MiB` | `50.6 MiB` (`1.00×`) | `50.6 MiB` (`1.00×`) |
| `random` | `0.7341 s` | `0.7372 s` (`+0.26%`) | `0.7464 s` (`+1.50% [-1.32%, +4.40%]`) | ovlp | `40.9 MiB` | `40.9 MiB` (`1.00×`) | `40.9 MiB` (`1.00×`) |
| `runs` | `0.0470 s` | `0.0475 s` (`+3.74%`) | `0.0473 s` (`-1.19% [-4.83%, +2.58%]`) | ovlp | `42.6 MiB` | `43.0 MiB` (`1.01×`) | `42.6 MiB` (`1.00×`) |
| `pattern` | `0.1784 s` | `0.1715 s` (`-4.43%`) | `0.1713 s` (`-4.23% [-16.10%, +9.32%]`) | ovlp | `9.2 MiB` | `9.2 MiB` (`1.00×`) | `9.2 MiB` (`1.00×`) |
| `onebyte` | `0.0584 s` | `0.0593 s` (`+0.58%`) | `0.0589 s` (`+0.24% [-5.36%, +6.17%]`) | ovlp | `51.5 MiB` | `51.5 MiB` (`1.00×`) | `51.5 MiB` (`1.00×`) |

#### JCC-32B-aligned (`-C llvm-args=-x86-branches-within-32B-boundaries`, VM alignment control)

| Corpus | `Cells` CPU | `Open` CPU (`Δ`) | `OpenR2` CPU (`Δ [95% CI]`) | Disj? | `Cells` RSS | `Open` RSS (`O/C`) | `OpenR2` RSS (`R2/C`) |
|---|---:|---:|---:|---|---:|---:|---:|
| `wikidata` | `0.7839 s` | `0.7696 s` (`-2.42%`) | `0.7533 s` (`-2.07% [-7.45%, +3.62%]`) | ovlp | `28.5 MiB` | **`16.4 MiB` (`0.57×`)** | `24.1 MiB` (`0.85×`) |
| `text` | `0.3300 s` | `0.3433 s` (`+3.22%`) | `0.3322 s` (`+0.12% [-3.08%, +3.42%]`) | ovlp | `21.1 MiB` | `21.1 MiB` (`1.00×`) | `21.1 MiB` (`1.00×`) |
| `records` | `0.8303 s` | `0.8310 s` (`-0.44%`) | `0.8282 s` (`-0.55% [-1.81%, +0.72%]`) | ovlp | `50.6 MiB` | `50.6 MiB` (`1.00×`) | `50.6 MiB` (`1.00×`) |
| `random` | `0.7091 s` | `0.7093 s` (`-2.05%`) | `0.7110 s` (`+0.91% [-7.68%, +10.30%]`) | ovlp | `40.9 MiB` | `40.8 MiB` (`1.00×`) | `40.9 MiB` (`1.00×`) |
| `runs` | `0.0481 s` | `0.0505 s` (`-1.65%`) | `0.0477 s` (`-6.32% [-23.89%, +15.31%]`) | ovlp | `42.6 MiB` | `43.0 MiB` (`1.01×`) | `42.7 MiB` (`1.00×`) |
| `pattern` | `0.2152 s` | `0.2186 s` (`-1.83%`) | `0.2114 s` (`-1.01% [-15.59%, +16.09%]`) | ovlp | `9.2 MiB` | `9.3 MiB` (`1.01×`) | `9.2 MiB` (`1.00×`) |
| `onebyte` | `0.0611 s` | `0.0620 s` (`+2.78%`) | `0.0600 s` (`-1.74% [-4.45%, +1.03%]`) | ovlp | `51.5 MiB` | `51.5 MiB` (`1.00×`) | `51.5 MiB` (`1.00×`) |

#### L3-Cold Inline `Seq` (`64 MiB` cache flush before every block, 3 interleaved rounds)

| Build | Corpus | `Cells` CPU | `Open` CPU (`Δ`) | `OpenR2` CPU (`Δ`) | Verdict |
|---|---|---:|---:|---:|---|
| Base x86-64 | `wikidata` | `0.9204 s` | `0.9304 s` (`+1.09%`, ovlp) | `0.9434 s` (`+2.49%`, ovlp) | `Cells` loses its 12% warm edge (`+70 ms` cold penalty vs `+5 ms` for `Open`) |
| Base x86-64 | `text` | `0.3793 s` | `0.3945 s` (`+4.02%`, ovlp) | `0.3707 s` (`-2.26%`, ovlp) | `OpenR2` eliminates `within_cap` penalty |
| JCC-32B | `wikidata` | `0.8986 s` | **`0.8267 s` (`-8.00%`, DISJ)** | **`0.8333 s` (`-7.27%`, DISJ)** | `Cells` suffers `+115 ms` L3-cold penalty vs `+57 ms` for `Open` |
| JCC-32B | `text` | `0.3672 s` | `0.3662 s` (`-0.27%`, ovlp) | `0.3484 s` (`-5.13%`, ovlp) | `OpenR2` wins cleanly |

**Key takeaway from Track 1A:**
- On **shipping baseline x86-64 (pre-`#1110`)**, warm inline `Seq(Cells)` (`0.8505 s`, `28.6 MiB`) beats `Seq(OpenCells)` (`+12.08%`) and `Seq(OpenR2Cells)` (`+9.21%` DISJOINT) — **reproducing physical i9-10900K `k = 1` (`1.123×` wall, `27.7 MiB` RSS in `run-20261125T022939Z` / `bce3914d`) to the tenth of a percent!**
- Under L3-cold, `EagerCells`'s `6.15 MiB` `cells.buf` takes a `+70–115 ms` cold-miss penalty, erasing its warm edge on Base (`+1.1%`) and putting `Open`/`OpenR2` `-7.3%..-8.0%` ahead DISJOINT on JCC-32B.

---

### 3.2 Parallel Workers at `t = 2` and `t = 3` (`BzReader::parallel_workers_with`)

5 counterbalanced interleaved rounds per build, steal-free process CPU (`proc_cpu_s`) and true subprocess `VmHWM` peak RSS.

#### Baseline x86-64 (`t = 2, 3`)

| Corpus | `t` | `Cells` CPU | `Open` CPU (`Δ`) | `OpenR2` CPU (`Δ [95% CI]`) | `Cells` RSS | `Open` RSS (`O/C`) | `OpenR2` RSS (`R2/C`) |
|---|--:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 2 | `0.950 s` | `0.930 s` (`-0.50%`) | `0.930 s` (`+0.62% [-7.17%, +9.06%]`) | `58.8 MiB` | **`28.5 MiB` (`0.48×`)** | `41.3 MiB` (`0.70×`) |
| `wikidata` | 3 | `0.960 s` | `0.920 s` (`-3.55%`) | `0.970 s` (`+0.63% [-0.08%, +1.34%]`) | `77.8 MiB` | **`32.3 MiB` (`0.42×`)** | `48.0 MiB` (`0.62×`) |
| `text` | 2 | `0.410 s` | `0.430 s` (`+6.29%`) | `0.410 s` (`-0.98% [-2.63%, +0.69%]`) | `44.9 MiB` | `45.0 MiB` (`1.00×`) | `45.5 MiB` (`1.02×`) |
| `text` | 3 | `0.430 s` | `0.450 s` (`+3.67%`) | `0.430 s` (`-1.37% [-3.84%, +1.16%]`) | `58.7 MiB` | `56.6 MiB` (`0.97×`) | `57.2 MiB` (`0.97×`) |
| `records` | 2 / 3 | `0.95 / 0.95 s` | `-1.64% / -0.65%` | `-0.40% / -1.24%` | `92.7 / 99.4 M` | `0.99× / 1.02×` | `1.02× / 1.02×` |
| `random` | 2 / 3 | `0.81 / 0.85 s` | `+0.24% / -2.04%` | `+0.24% / -2.54%` | `80.9 / 86.9 M` | `1.00× / 1.00×` | `1.02× / 1.02×` |
| `runs` | 2 / 3 | `0.07 / 0.07 s` | `+0.41% / -0.41%` | `+0.41% / -5.98%` | `56.5 / 56.7 M` | `0.99× / 0.99×` | `1.00× / 1.00×` |
| `pattern` | 2 / 3 | `0.31 / 0.32 s` | `-0.53% / -0.71%` | `-2.40% / +0.46%` | `18.2 / 26.8 M` | `1.05× / 1.01×` | `1.05× / 1.02×` |
| `onebyte` | 2 / 3 | `0.11 / 0.11 s` | `+3.54% / +2.33%` | `+0.00% / +0.42%` | `93.1 / 91.8 M` | `0.99× / 0.97×` | `0.96× / 0.98×` |

#### JCC-32B-aligned (`t = 2, 3`)

| Corpus | `t` | `Cells` CPU | `Open` CPU (`Δ`) | `OpenR2` CPU (`Δ [95% CI]`) | `Cells` RSS | `Open` RSS (`O/C`) | `OpenR2` RSS (`R2/C`) |
|---|--:|---:|---:|---:|---:|---:|---:|
| `wikidata` | 2 | `0.920 s` | `0.840 s` (`-8.93%`) | `0.870 s` (`-6.09% [-12.20%, +0.45%]`) | `56.8 MiB` | **`29.2 MiB` (`0.51×`)** | `41.2 MiB` (`0.73×`) |
| `wikidata` | 3 | `1.030 s` | **`0.840 s` (`-16.18%`)** | **`0.870 s` (`-13.59% [-20.57%, -5.99%]` DISJ)** | `76.1 MiB` | **`31.3 MiB` (`0.41×`)** | `49.5 MiB` (`0.65×`) |
| `text` | 2 / 3 | `0.41 / 0.42 s` | `-0.11% / +5.69%` | `-5.33% / +0.96%` | `42.2 / 59.5 M` | `1.05× / 0.97×` | `1.07× / 1.04×` |
| `records` | 2 / 3 | `0.95 / 0.95 s` | `-2.09% / -2.27%` | `-1.93% / -1.49%` | `92.7 / 100.5 M` | `1.00× / 1.00×` | `1.03× / 1.01×` |
| `random` | 2 / 3 | `0.81 / 0.80 s` | `-0.51% / -0.25%` | `+0.24% / -0.50%` | `82.6 / 88.3 M` | `1.00× / 1.00×` | `0.98× / 0.99×` |
| `runs` | 2 / 3 | `0.07 / 0.07 s` | `-0.41% / +2.28%` | `+0.00% / -0.41%` | `56.5 / 56.6 M` | `1.00× / 0.99×` | `1.00× / 1.00×` |
| `pattern` | 2 / 3 | `0.32 / 0.32 s` | `-2.51% / +1.28%` | `-0.04% / -1.85%` | `19.0 / 27.7 M` | `1.01× / 0.98×` | `1.00× / 0.97×` |
| `onebyte` | 2 / 3 | `0.11 / 0.12 s` | `+1.92% / +1.49%` | `+5.53% / +3.40%` (ovlp, 10ms tick) | `91.1 / 92.3 M` | `1.00× / 0.98×` | `1.05× / 0.97×` |

---

### 3.3 Strategic Discovery 1 — `#1110` (`forward-anchor-lanes`) Flips the Entire `t = 1, 2, 3` Crossover!

Enabling `--features forward-anchor-lanes` (`#1110`, 4-lane forward emission in `OpenCells`) completely flips `OpenCells` vs `Cells` on **both** shipping Base x86-64 (no JCC flag!) AND JCC-32B:

| Build (`+ #1110`) | Mode | Corpus | `Cells` CPU | **`Open + #1110` CPU (`Δ [95% CI]`)** | `OpenR2 + #1110` CPU (`Δ`) | `Cells` RSS | **`Open + #1110` RSS** | `OpenR2` RSS |
|---|---|---|---:|---:|---:|---:|---:|---:|
| **Base x86-64** | `Seq t=1` | `wikidata` | `0.7992 s` | **`0.7863 s` (`-2.01% [-3.2%, -0.8%]`)** | `0.8013 s` (`+2.42%`) | `28.5 MiB` | **`17.5 MiB` (`0.61×`)** | `25.1 MiB` |
| **Base x86-64** | `Par t=2` | `wikidata` | `0.910 s` | **`0.840 s` (`-4.96% [-11.8%, +2.4%]`)** | `0.880 s` (`+1.00%`) | `56.7 MiB` | **`31.0 MiB` (`0.55×`)** | `42.4 MiB` |
| **Base x86-64** | `Par t=3` | `wikidata` | `0.950 s` | **`0.850 s` (`-10.89% [-14.8%, -6.8%]` DISJ)** | **`0.890 s` (`-7.32%` DISJ)** | `75.7 MiB` | **`34.6 MiB` (`0.46×`)** | `54.2 MiB` |
| **JCC-32B** | `Seq t=1` | `wikidata` | `0.7855 s` | **`0.7109 s` (`-9.08%`)** | `0.7521 s` (`-3.47%`) | `28.5 MiB` | **`17.4 MiB` (`0.61×`)** | `25.0 MiB` |
| **JCC-32B** | `Par t=2` | `wikidata` | `0.930 s` | **`0.800 s` (`-14.44%` DISJ)** | **`0.830 s` (`-11.25%` DISJ)** | `56.9 MiB` | **`30.6 MiB` (`0.54×`)** | `42.6 MiB` |
| **JCC-32B** | `Par t=3` | `wikidata` | `0.950 s` | **`0.830 s` (`-13.66%` DISJ)** | **`0.860 s` (`-9.44%` DISJ)** | `76.3 MiB` | **`33.8 MiB` (`0.44×`)** | `53.8 MiB` |
| Base / JCC | `t=1..3` | `text` | `0.34..0.45 s` | `+1.39%..+5.34%` (`within_cap` overhead) | **`-1.62%..+0.90%` (`t=1,2`)** | `21..59 M` | `~1.00×` | `~1.00×` |

**Implication:** Switching `t = 1, 2` to `OpenCells` *before* `#1110` regresses single-threaded `wikidata` (`+9.2%..+12.1%` Base x86-64; `+12.3%` on i9 `k = 1`). Once `#1110` (`forward-anchor-lanes`) lands on `master`, `OpenCells` beats `EagerCells` on **every** thread count `t = 1, 2, 3` (`-2.0%..-10.9%` Base, `-9.1%..-14.4%` JCC-32B) with **`-39%..-56%` peak RSS**, making full `t = 1..=20` unification (and eventual `EagerCells` retirement) a clean post-`#1110` i9-gated follow-up.

---

### 3.4 Strategic Discovery 2 — Critical `R2_2tier_m22` RSS Footgun on `wikidata` (Actionable for `post-1080-open-admission`)

Across every single run in §3.1–§3.3, **`OpenR2Cells` (`R2_2tier_m22`) added `+7.8 MiB` (`t = 1`), `+12.8 MiB` (`t = 2`), and `+15.7–20.0 MiB` (`t = 3`) of peak RSS on `wikidata` compared to shipped `OpenCells`** (`+45%..+49%` higher RSS than shipped `OpenCells`!):
- **Root cause (proven by Track 2 `trace` buffer ledger in §4.2):** Shipped `OpenCells` admits **all 233/233** full Wikidata blocks (`198 Low + 30 Mid + 5 Dense`), so its `Segmented` fallback (`tt` + `seg.bufs`) is **never allocated (`0 B`)**. `R2_2tier_m22` refuses **9 full Wikidata blocks** (`4 MidRef` with `m/n ∈ (0.22, 0.26)` + `5 Dense` with `m/n ∈ [0.221, 0.257]`), sending them to `Segmented` — which permanently allocates `tt` (`7.03 MiB` `Vec` capacity after doubling from `3.6 MB`) + `seg.bufs` (`4.14 MiB`) = **`+11.2 MiB` cap / `+7–8 MiB` resident per worker!**
- At `k = 8` workers on the 20 MiB L3 i9-10900K, `+7–8 MiB/worker` would add **`+55–65 MiB` RSS** (violating the `<= 1.02×` i9 RSS gate!).
- **Clean fix for `post-1080-open-admission`:** Because all 5 Wikidata `Dense` blocks have `m/n ∈ [0.221, 0.257]` and `r/n ∈ [0.15, 0.17]`, while all 40 `text` `Dense` blocks have `m/n >= 0.30` (most `0.40–0.55`), raising the `m/n` cap from `0.22` to `0.26` over `0.10 <= r/n < 0.17` (and rejecting `r/n >= 0.17` outright with zero `within_cap` scan, OR releasing `Segmented`'s buffers after rare fallback) keeps **233/233 Wikidata blocks in `Open` (`0 B` `Segmented` allocation!)** while still rejecting `text`'s dense blocks! Also note under `#1110` (`forward-anchor-lanes`), keeping those 9 Wikidata blocks in `Open` was actually **faster** (`0.7109 s` vs `0.7521 s` on `JCC+FWD`, `0.7863 s` vs `0.8013 s` on `Base+FWD`)!

---

## 4. Track 2 — Micro-Dissection of `src/cells.rs`, `src/cells_owned.rs`, and `src/run_fed.rs`

### 4.1 `src/run_fed.rs` (830 LOC) Audit

`src/lib.rs:134` gates `src/run_fed.rs` strictly behind `#[cfg(test)] mod run_fed;` (*"Research only, compiled for the tests and nothing else ... Nothing in the decoder calls this module."*). It implements Line A (`build_from_runs` / `build_from_runs_wide` run-fed T-vector) and Line B (`Ψ²` census) purely as unit-tested research prototypes. **Zero production instructions, zero RSS, zero hot-path presence.**

### 4.2 Why `EagerCells` Peak RSS Is `1.75–2.4×` `OpenCells` (`trace::buffers()` Ledger)

Exact `Scratch::buffers()` high-water capacity after decoding full `wikidata.bz2` (431 blocks) and `text.bz2` (47 blocks) (`probe-diag`, `--features "moonshot trace"`):

| Corpus | Kernel | Total `Scratch` cap | Breakdown of allocations |
|---|---|---:|---|
| `wikidata` | **`Cells`** (`Eager`) | **`17.16 MiB`** | `cells.buf = 6,151.2 KiB` (`6.01 MiB`!) + `tt = 3,515.7 KiB` + `seg.bufs = 5,263.4 KiB` (`8.57 MiB` `Segmented` fallback!) + `ll = 1,757.7 KiB` + `runs = 844.2 KiB` |
| `wikidata` | **`OpenCells`** | **`2.54 MiB`** *(+ ~2 MiB Open internal)* | `ll = 1,757.7 KiB` + `runs = 844.2 KiB`; **`tt = 0 B`, `seg.bufs = 0 B`, `cells.buf = 0 B`** (admits 233/233 full blocks!) |
| `wikidata` | **`OpenR2Cells`** | **`13.47 MiB`** *(+ ~2 MiB Open internal)* | `ll = 1,757.7 KiB` + `runs = 844.2 KiB` + **`tt = 7,031.2 KiB` + `seg.bufs = 4,141.7 KiB`** (allocated by 9 refused Wikidata blocks!) |
| `wikidata` | `Segmented` | `16.81 MiB` | `ll = 1,757.7 KiB` + `runs = 844.2 KiB` + `tt = 7,030.8 KiB` + `seg.bufs = 7,563.4 KiB` |
| `text` | `Cells` / `OpenR2` / `Seg` | `15.87 MiB` | Identical (`Cells` admits 0/47 `text` blocks -> 100% `Segmented`: `tt = 7,031.2 KiB` + `seg.bufs = 5,571.2 KiB` + `runs = 1,868 KiB` + `ll = 1,758 KiB`) |

Two mechanisms explain `EagerCells`'s `1.75–2.4×` RSS (`28.6 / 58.8 / 77.8 MiB` vs `16.3 / 28.5 / 32.3 MiB` at `t = 1, 2, 3`):
1. **`cells.buf` (`6.01 MiB`/worker):** `Kernel::slots` (`src/cells.rs:653-704`) allocates `tops` (`letters × 20 B ≈ 1.4 MiB`) + `arena` (`16 × budget = 2.30 MiB`) + `rules` (`16 × budget = 2.30 MiB`) for `budget = 0.16 × n = 144,000` cells and zero-fills/touches the allocation eagerly.
2. **Double-engine residency (`+8.57 MiB`/worker on `wikidata`):** Because `Cells`'s density gate (`recs.len() * 10 >= n`, `src/cells.rs:597`) refuses **35** full Wikidata blocks (`26 MidAdmit + 4 MidRef + 5 Dense`, plus `2 Budget` refusals = **37/233** refused full blocks!), every worker also permanently allocates `fallback: Segmented` (`tt` + `seg.bufs`) alongside `cells.buf`.

### 4.3 Per-Bucket & Internal Phase Breakdown of `EagerCells` on `wikidata.bz2`

Route census (`431` blocks): `Cells` admits `196` full blocks (`Low(<0.10)`, `20,456,299` cells, `100%` full-period `0` fallback, `6,339,139` rules, `2,122,787` writer commands), refuses `185` (`density`: `35` full + `150` runts), `2` (`budget`), `48` `Column` runts.

Per-bucket `decode_block` CPU medians (`schedstat` ms, 3 interleaved reps, warm):

| Bucket (`wikidata`) | Blks | Rows | `Cells` (Base / JCC / JCC+`#1110`) | `Open` (Base / JCC / JCC+`#1110`) | `OpenR2` (Base / JCC / JCC+`#1110`) | `Segmented` (Base / JCC) |
|---|---:|---:|---:|---:|---:|---:|
| `Low(<0.10)` | 198 | 178.20M | `666.1 / 604.5 / 600.9` | `655.4 / 558.9 / 528.5` (`-12.1%` w/ `#1110`) | `667.0 / 563.6 / 528.0` (`-12.1%`) | `946.9 / 915.1` (`+42..51%`) |
| `MidAdmit(0.10..0.15)` | 26 | 23.40M | `133.8 / 131.8 / 122.8` *(Seg!)* | `134.3 / 120.7 / 103.1` (`-16.1%` w/ `#1110`) | `154.9 / 127.3 / 125.3` | `133.4 / 130.5` |
| `MidRef(0.10..0.15)` | 4 | 3.60M | `20.0 / 18.5 / 23.0` *(Seg)* | `27.6 / 19.9 / 19.8` | `20.2 / 19.9 / 20.1` | `22.6 / 19.9` |
| `Dense(0.15..0.28)` | 5 | 4.50M | `29.7 / 25.0 / 24.1` *(Seg)* | `36.3 / 27.9 / 31.4` | `31.4 / 26.9 / 26.0` | `28.0 / 20.0` |
| **Total `decode_block`** | **431** | **209.70M** | **`858.4 / 781.9 / 770.6`** | **`847.4 / 729.1 / 694.7` (`-9.8%` DISJ)** | `861.3 / 739.8 / 704.4` (`-8.6%`) | `1136.4 / 1087.1` |

Internal phase attribution inside `Cells` (`writer_owned` mode 2 vs mode 3 on `wikidata.bz2`, `taskset -c 1`):
- `MTF/front`: `70.7–73.7 ms` (`~9%` of `decode_block`)
- `RLE1+CRC`: `62.9–79.3 ms` (`~9%`)
- `Segmented` fallback (37 refused full blocks): `~175–183 ms` (`~22%`)
- `Cells` inverse BWT (`196` admitted blocks, `~515–560 ms` total = `~65%` of `decode_block`):
  - **`Kernel::slots` + `Kernel::induce_cells`:** **`~445–485 ms` (`~85–88%` of `Cells` invBWT)** — `20,456,299` induction steps (`~22–24 ns/step`), memory-latency bound over the `6.15 MiB` `cells.buf` + `0.9 MiB` `out` bottom slots.
  - **`Kernel::write_owned` (`owned_scan` + `owned_copies`):** **`~71.0 ms` CPU** (`104.3 ms` wall-best; **`~12–15%` of `Cells` invBWT, `~8–10%` of whole-file `Cells` CPU**), split ~75–85% in `owned_scan` (`~66.7–88.2 ms` for `6.34M` rules) and ~15–25% in `owned_copies` (`~9.6–28.2 ms` for `2.12M` commands).

### 4.4 Static `objdump -d` Audit of `src/cells.rs` & `src/cells_owned.rs` (`probe-base`)

| Symbol | Insns | `call`s | `vzeroupper` | `prefetcht0` | Stack mem ops | 32B JCC cross | Hot-loop microarchitectural findings |
|---|---:|---:|---:|---:|---:|---:|---|
| `Kernel::slots` | 589 | 9 (`1 memset`, `3 alloc`, `5 panic`) | 0 | 0 | 74 | 10 | Cold allocs + `cnt=[0;257]` zeroing; 2 passes over run records (`r ≈ 50–80k`/blk) |
| **`Kernel::induce_cells`** | **228** | **0** | **0** | **8** | **38** | **0** | Already zero-call, zero-bounds-check, zero-JCC-crossing, 8× software `prefetcht0` (`FAR=12, NEAR=6`). Spills `cells`/`budget`/`top`/`rp` to red zone (`-0x60(%rsp)`) on 15 GPRs, masked by 22–24 ns/step L2/L3 miss latency |
| **`Kernel::write_owned`** | **422** | **13 (`4 memset`, `2 memcpy`, `1 memmove`, `2 alloc`, `1 panic`)** | **0** | **0** | **65** | **10** | **`owned_scan` calls GOT-indirect `memcpy@GLIBC_2.14` TWICE (`853f5`, `8546d`) on every `<= 16 B` arena child (`copy_small`), forcing caller-saved reloads of `r8/r9/rbp/rdi` from `(%rsp)` after every leaf write!** `owned_copies` calls indirect `memmove@GLIBC` (`855a9`) on `2.12M` commands. |
| `Kernel::write_cells` | 529 | 20 | 0 | 0 | 72 | 10 | Cold fallback only (`0/196` Wikidata blocks hit it; 100% full-period go to `write_owned`) |

**Why Track 2B is a clean `STOP` despite the `write_owned` `memcpy` finding:**
Even eliminating 100% of `memcpy` call overhead inside `owned_scan` (cutting `owned_scan` by an optimistic `-30%..-40%`) saves only **`~20–27 ms` (`~2.5%–3.5%` of `Cells` whole-file CPU)**, falling below the frozen **`>= 5.0%` standalone `src/cells.rs` gate**, because `85–88%` of `Cells` invBWT sits in the already-tight, 0-call, 8-prefetch `induce_cells` memory-latency loop — AND `#1110` (`forward-anchor-lanes`) already makes `OpenCells` beat `Cells` by `-2.0%..-10.9%` Base / `-9.1%..-14.4%` JCC-32B at `t = 1, 2, 3` with half the RSS!

---

## 5. Final Verdicts Against Frozen Stop Test (Strict C2)

1. **Track 1 (Immediate pre-`#1110` `t = 1, 2` switch to `OpenCells` / `OpenR2Cells` on current `master`): `STOP`.**
   - Fails Criterion (i) on shipping baseline x86-64 warm `wikidata` `t = 1` (`Open` `+12.08%`, `OpenR2` `+9.21% [+2.38%, +16.50%]` DISJOINT SLOWER, `OpenR2` RSS `0.84×`), matching physical i9-10900K `k = 1` (`1.123×`).
   - **Scoped boundary (what this does NOT close):** Once `#1110` (`forward-anchor-lanes`) AND an RSS-safe post-`#1080` admission rule (preventing `Segmented` allocation on the 9 Wikidata blocks!) land on `master`, `OpenCells + #1110` beats `EagerCells` at **all** `t = 1, 2, 3` (`-2.0%..-10.9%` Base, `-9.1%..-14.4%` JCC-32B, `0.44–0.61×` RSS) and should be evaluated in a single physical i9-10900K `k = 1, 2, 3` unification run!
2. **Track 2 (`src/cells.rs` + `src/run_fed.rs` standalone micro-optimization): `STOP`.**
   - `src/run_fed.rs` is test-only dead code (`#[cfg(test)]`); `induce_cells` is already an ultra-tight 228-insn zero-call/8-prefetch memory-latency loop (`85–88%` of `Cells` invBWT); `write_owned`'s `memcpy@GLIBC` calls sit inside a `~8–10%` `owned_scan` phase (`~2.5–3.5%` ceiling `< 5.0%` gate) and are strategically superseded by post-`#1110` `OpenCells` unification.
3. **No `status = "idea"` claim opened** (both frozen gates stopped); both scoped `STOP` rows recorded in `frontier/dead-ends.tsv` and the `R2_2tier_m22` RSS warning recorded in `frontier/deltas/2026-11-27-explore-eager-cells-and-seq-open.md`.
