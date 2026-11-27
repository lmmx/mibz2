# Dissecting `#1034`'s 4-thread `pattern` CI regression & `scanner-per-block-fixed-cost` probe (`REFUTED`)

Explore / Smooth-zone probe (`scanner-per-block-fixed-cost`), 27 November 2026, comparing `#1034` (`5ed20c1d`, PR head `782f4feb`) against its first parent (`5ed20c1d^` / `f9848a48`).

## Executive verdict

1. **Frozen stop test (`frontier/claims/scanner-per-block-fixed-cost.toml`): `REFUTED`.**
   `#1034` did **not** introduce a per-block fixed scanner cost or first-block pipeline-fill penalty on `pattern`. On the contrary, `PAIR_GATE_SEL = 64` skips 2-symbol pair-table construction on all 22/22 blocks (`n_sel = 3`), cutting per-block `A2` setup by **−88.9%** (`3.93 → 0.44 µs/blk`, **9.0× faster**, disjoint ranges across 9 interleaved repeats on both baseline x86-64 and `-C target-cpu=native`), cutting total `S = A2 + A3` by **−77.1%** (`96.0 → 22.0 µs`, **4.36× faster**, disjoint), and cutting cold first-block scan latency (`open → read → blk0 → worker recv`) by **−13.4 µs** (`30.4 → 17.1 µs`). Consequently, `scanner-a2-fast-build` will **not** move 4T `pattern` wall time (its `A2` on `pattern` is already `0.006 ms` across 22 blocks).
2. **Root cause of CI run `45947633032` (`+7.3%` `[+2.6%, +12.3%]` 4T `pattern` wall, `−2.2%` CPU, `+7.9%` peak RSS):**
   It is a **pre-existing bimodal head-of-line scheduling / `ready`-backlog spike** on 4-vCPU (2-core × 2-SMT) CI runners under 6-on-4 thread oversubscription (`main` + `mibz2-scan` + 4 workers, `slots = 16`), **not** caused by `#1034`.
   - Across **10 independent CI runs** (200 `pattern t=4` executions on 5 different Xeon/EPYC CPUs), `r(rss, wall)` is positive in **10/10** runs (`+0.33` to `+0.84`, median `+0.75`), `r(rss, runq)` is positive in **10/10** (`+0.41` to `+0.83`, median `+0.64`), and `r(rss, cpu)` is negative in **9/10** (`−0.19` to `−0.67`, median `−0.58`).
   - Across **4 true null-control CI runs** (80 executions with **zero** Rust decoder changes, including Python-only PR #1033 `45933661519` which touched only `src/python.rs`), the high-RSS (`> 38.5 MB`) spike rate is **`30/80` (`37.5%`)** (`base 14/40 = 35.0%`, `head 16/40 = 40.0%`). **Python-only PR #1033 (`45933661519`) independently triggered `+6.0% (+0.2%, +12.2%) slower` wall, `−0.2%` CPU, and `+5.9%` peak RSS (`36 → 39 MB`)** on `pattern t=4` from a `2/10` vs `6/10` draw — the exact same telemetry signature and `PERFORMANCE-POLICY.md` §2 rule 3 band as `#1034`'s `45947633032`.
   - Across **all 3 independent CI runs of `#1034`** (`45921211004` `2/10`, `45947633032` `6/10`, master push `45949114795` `3/10`), `#1034`'s pooled high-RSS rate is **`11/30` (`36.7%`)** — statistically indistinguishable from the null-control rate (`37.5%`, Fisher exact `p = 1.0`). Run `45947633032` fired `+7.3%` because `base` happened to draw `0/10` spikes while `head` drew `6/10`.

---

## 1. Deterministic census of `pattern.bz2` (CI `SCALE = 4`)

Generated via `ab gen DIR 4` (`tests/common/mod.rs`: `b"0123456".repeat(700_000 * 4)`, libbzip2 `-9`):

| Metric | Value | Notes |
|---|---:|---|
| Compressed / decoded size | `1,116 B → 19,600,000 B` | `17,562.7×` expansion; ~50 B/block |
| Blocks | `22` | `21 × 900,000` + `1 × 700,000` BWT rows |
| Inverse-BWT route (`route_census`) | `22/22` `runs_not_open` (`Segmented`) | `r/n = 0.008`; `0` Open, `0` Column |
| Alphabet / groups | `alpha = 9`, `n_groups = 2`, `used = 1` | `used = b"0123456"`; lengths `[4,4,3,3,3,3,3,3,3]` |
| Selectors / runs / symbols | `n_sel = 3/blk` (`66` tot), `sel_runs = 1/blk` | `121` syms/blk (`120` on blk 21); **`2,661` syms total** |
| Pair-gated blocks (`n_sel < 64`) | **`22/22` (100%)** | `#1034` skips pair-table build on every block |
| Single-thread stage split (`phases`) | Scanner `0.01 c/B` (**`0.1%`**) | `0.02 c/B` (`0.3%`) on `5ed20c1d^`; `99.7–99.9%` is worker (`walk 68–74%`, `build 18–21%`, `RLE1 3.8–5.0%`, `MTF 2.4–2.5%`, `CRC 2.1–3.1%`) |

`pattern` contains only `2,661` Huffman symbols across 22 blocks (`~22 µs` total serial scan time post-`#1034` vs `~20 ms` 4T wall on CI). Any change confined to the serial scanner kernel cannot account for a `+1,500 µs` (`+7.3%`) 4T wall loss unless it adds `> 70 µs/block` of fixed overhead.

---

## 2. `scanner-per-block-fixed-cost` probe (`examples/scanner_per_block_fixed_cost.rs`)

**Methodology (VM diagnostic, C5):** 2-vCPU Intel Xeon @ 2.30 GHz VM (BMI2/AVX2), `rustc 1.99.1`, `taskset -c 0` (1 thread pinned), `#![forbid(unsafe_code)]`, zero `src/` edits. Both `parent` (`5ed20c1d^`: unconditional pair build + `syms.resize(max_out, 0)` + `decode_group_win/slice`) and `#1034` (`5ed20c1d`: `PAIR_GATE_SEL = 64` + byte-aligned `u8` LUT + `syms.clear()` + coalesced chunked `decode_run` + `is_x86_feature_detected!("bmi2")`) are verified **100% symbol-for-symbol and end-bit exact** against shipped `mibz2::stream::Scanner` across all **455 blocks (`25,044,344` symbols)** of `pattern`, `runs`, `onebyte`, and `wikidata`, then timed **best of 9, interleaved** (alternating arm order each repeat).

### 2a. Per-block `A2` (fixed setup) and `A3` (symbol decode) — baseline x86-64

| Corpus (blocks, pair-gated) | Stage | `parent` best (med) [range] | `#1034` best (med) [range] | Per-block best (`parent → #1034`) | Delta best (med) | Ranges |
|---|---|---:|---:|---:|---:|---|
| `pattern` (`22`, `22/22`) | **A2 (setup)** | `86.56` (`87.88`) `[86.56..89.06]` µs | **`9.62`** (`9.96`) `[9.62..10.31]` µs | `3.934 → 0.437` µs/blk | **−88.9%** (`−88.7%`) | **FASTER (disjoint)** |
| `pattern` (`22`, `22/22`) | A3 (syms) | `7.58` (`7.77`) `[7.58..8.42]` µs | `11.08` (`11.36`) `[11.08..12.92]` µs | `0.344 → 0.504` µs/blk | +46.3% (+3.50 µs tot) | SLOWER (disjoint) |
| `pattern` (`22`, `22/22`) | **S (total)** | `96.03` (`96.47`) `[96.03..98.29]` µs | **`22.03`** (`22.53`) `[22.03..24.12]` µs | `4.365 → 1.002` µs/blk | **−77.1%** (`−76.7%`) | **FASTER (disjoint)** |
| `onebyte` (`1`, `1/1`) | **A2 (setup)** | `4.09` (`4.12`) `[4.09..4.45]` µs | **`0.36`** (`0.37`) `[0.36..0.39]` µs | `4.088 → 0.364` µs/blk | **−91.1%** (`−91.1%`) | **FASTER (disjoint)** |
| `onebyte` (`1`, `1/1`) | A3 (syms) | `0.18` (`0.18`) `[0.18..0.20]` µs | `0.22` (`0.23`) `[0.22..0.25]` µs | `0.178 → 0.223` µs/blk | +25.4% (+0.05 µs) | SLOWER (disjoint) |
| `onebyte` (`1`, `1/1`) | **S (total)** | `4.31` (`4.36`) `[4.31..4.68]` µs | **`0.63`** (`0.65`) `[0.63..0.66]` µs | `4.309 → 0.628` µs/blk | **−85.4%** (`−85.0%`) | **FASTER (disjoint)** |
| `runs` (`1`, `0/1`) | A2 (setup) | `44.29` (`46.41`) `[44.29..54.47]` µs | `44.31` (`45.44`) `[44.31..51.06]` µs | `44.286 → 44.311` µs/blk | +0.06% (−2.1%) | overlap |
| `runs` (`1`, `0/1`) | A3 (syms) | `259.17` (`262.04`) `[259.17..272.78]` µs | **`200.97`** (`207.75`) `[200.97..213.26]` µs | `259.17 → 200.97` µs/blk | **−22.5%** (`−20.7%`) | **FASTER (disjoint)** |
| `runs` (`1`, `0/1`) | **S (total)** | `305.61` (`308.79`) `[305.61..327.37]` µs | **`245.36`** (`253.24`) `[245.36..259.66]` µs | `305.61 → 245.36` µs/blk | **−19.7%** (`−18.0%`) | **FASTER (disjoint)** |
| `wikidata` (`431`, `198/431`) | A2 (setup) | `10,339` (`10,680`) `[10,339..11,516]` µs | `9,379` (`9,674`) `[9,379..11,393]` µs | `23.99 → 21.76` µs/blk | −9.3% (−9.4%) | overlap |
| `wikidata` (`431`, `198/431`) | A3 (syms) | `86,266` (`87,061`) `[86,266..89,417]` µs | **`68,314`** (`69,086`) `[68,314..83,625]` µs | `200.15 → 158.50` µs/blk | **−20.8%** (`−20.7%`) | **FASTER (disjoint)** |
| `wikidata` (`431`, `198/431`) | **S (total)** | `96,659` (`97,781`) `[96,659..100,217]` µs | **`78,008`** (`78,514`) `[78,008..95,073]` µs | `224.27 → 180.99` µs/blk | **−19.3%** (`−19.7%`) | **FASTER (disjoint)** |

`-C target-cpu=native` (`cfg(target_feature = "bmi2") = true`) confirms identical scaling: on `pattern`, `A2` drops `78.59 → 8.24 µs` (**−89.5%**, disjoint), `A3` shifts `7.42 → 10.40 µs` (`+2.98 µs` from pair-gating), and total `S` drops `87.71 → 20.16 µs` (**−77.0%**, disjoint); `onebyte` `S` drops `3.77 → 0.56 µs` (**−85.1%**, disjoint); `runs` `S` drops `299.3 → 229.7 µs` (**−23.2%**); `wikidata` `S` drops `98.20 → 73.08 ms` (**−25.6%**).

### 2b. First-block scan latency (`File::open → read → blk0 → worker recv`, best of 9)

| Corpus | `syms` state | Baseline `parent` best (med) | Baseline `#1034` best (med) | Native `parent` best (med) | Native `#1034` best (med) |
|---|---|---:|---:|---:|---:|
| `pattern` | cold (`Vec::new`) | `30.44` (`31.48`) µs | **`17.06`** (`28.06`) µs (−43.9%) | `27.62` (`31.74`) µs | **`24.77`** (`28.93`) µs (−10.3%) |
| `pattern` | warm (reserved) | `18.73` (`30.73`) µs | `25.04` (`29.57`) µs (within range) | `24.02` (`31.12`) µs | `24.82` (`29.90`) µs (within range) |
| `runs` | cold (`Vec::new`) | `335.64` (`345.75`) µs | **`291.64`** (`296.41`) µs (−13.1%) | `328.67` (`381.07`) µs | **`263.93`** (`272.56`) µs (−19.7%) |
| `onebyte` | cold (`Vec::new`) | `27.88` (`37.14`) µs | **`24.25`** (`26.84`) µs (−13.0%) | `27.07` (`30.94`) µs | **`16.52`** (`27.86`) µs (−39.0%) |
| `wikidata` | cold (`Vec::new`) | `434.33` (`466.66`) µs | **`404.89`** (`464.48`) µs (−6.8%) | `399.34` (`442.03`) µs | **`399.04`** (`445.35`) µs (−0.1%) |

**Stop-test evaluation:** On this VM, quiet 4T `pattern` wall is `~52 ms`; 50% of CI's `+7.3%` loss scaled to this machine is **`~1,900 µs`**. Post-`#1034` per-block `A2` setup is **−76.9 µs faster** (`−88.9%`, disjoint) and total `S` is **−74.0 µs faster** (`−77.1%`, disjoint), while cold first-block latency is **−13.4 µs faster** (and warm first-block median is `−1.16 µs`, well inside the parent's `[18.7..45.6] µs` range). **Verdict: `REFUTED`.**

---

## 3. Root-cause dissection of CI run `45947633032` (`+7.3%` 4T `pattern` wall)

### 3a. The telemetry paradox in `45947633032`

Inspecting `runs.jsonl` of CI run `45947633032` (Intel Xeon Platinum 8573C, 2 cores × 2 SMT = 4 logical CPUs):

- **`t=1`:** wall `32.80 → 32.85 ms` (`+0.6%`), CPU `34.5 → 34.5 ms` (`+0.5%`), RSS `16.4 → 16.4 MB` (`−0.1%`). Flat.
- **`t=2`:** wall `20.25 → 19.75 ms` (`+0.9%` geomean), CPU `40.4 → 39.8 ms` (`+0.2%`), RSS `20.6 → 20.7 MB` (`+0.8%`). Flat.
- **`t=4`:** wall `19.75 → 21.10 ms` (**`+7.3%`** `[+2.6%, +12.3%]`), peak RSS `35.7 → 38.3 MB` (**`+7.9%`**, spiking to `43.6 MB`), run-queue wait `7.70 → 12.45 ms` (**`+62%`**, spiking to `20.2 ms`), PSI CPU stall `3.88 → 5.28 ms`, while **total CPU *fell* `72.1 → 70.8 ms` (`−2.2%`, dropping to `63.7–66.0 ms` on the slowest rounds)**.

Conditioning `45947633032`'s `t=4` rounds on peak RSS (`<= 38.5 MB` vs `> 38.5 MB`):

| Group in `45947633032` (`pattern t=4`) | Rounds | Wall median (mean) | CPU median | Peak RSS median | `runq` median |
|---|---:|---:|---:|---:|---:|
| `base` (`f9848a4`) — all `<= 38.5 MB` | `10/10` | `19.75` (`19.61`) ms | `72.1` ms | `35.7` MB | `7.70` ms |
| `head` (`782f4fe`) — low-RSS (`<= 38.5 MB`) | `4/10` | `20.20` (`20.20`) ms | `74.4` ms | `36.0` MB | `6.70` ms |
| `head` (`782f4fe`) — high-RSS (`> 38.5 MB`) | `6/10` | **`21.45` (`21.62`) ms** | **`68.2` ms** | **`40.9` MB** | **`15.00` ms** |

### 3b. Physical mechanism: 6-on-4 oversubscription → head-of-line `ready` pile-up → unrecycled `956 KiB` `Vec<u8>` output buffers

Confirmed via traced `occupancy --features trace` on `pattern.bz2` (`slots = 4, 8, 16`) and aligned with `#1094` (`docs/journal/2026-11-27-explore/expansive-block-memory.md`):

1. At `t = 4`, `BzReader::parallel` runs `main` (reader + hash), `mibz2-read`, `mibz2-scan`, and 4 `Segmented` workers (`7` threads at startup, `6` sustained) on 4 logical CPUs (2 physical cores), with `slots = 16`.
2. `mibz2-scan` scans 16 blocks in `< 0.1 ms` and fills the entire 16-slot window immediately. `held: symbols mean 0.0 MB` (`syms` is only 121 `u16`s = 1 faulted 4 KiB page per slot — zero RSS effect).
3. Each worker holds `6.15 MB` of scratch (`tt 3.60 MB + seg.bufs 2.51 MB`, × 4 workers = `24.6 MB`), and allocates a **`956 KiB` (`0.96 MB`)** `Vec<u8>` output buffer (`Vec::with_capacity(n + n/16 + 64)`) whenever `st.free_out` is empty.
4. Buffers only return to `st.free_out` when `main` pops blocks from `st.ready` **in strict order** (`taken = 0, 1, 2, …`). Whenever 6-on-4 runqueue preemption delays `main` or the head-of-line worker (`occupancy`: `ready→taken` spikes up to `11.1–24.9 ms`, `ready, not taken` reaches `5+` blocks = `4.5+ MB`), the remaining workers keep finishing later blocks and allocating fresh `956 KiB` `Vec<u8>` buffers (`+3 to +8 MB` RSS, `+700–1,900` extra minor page faults).
5. Because workers finish early or stall on slots while `main` drains and hashes the piled-up tail, active SMT concurrency drops on the 2-core runner → higher IPC per active thread → **total CPU *drops*** (`r(rss, cpu) < 0`) while **wall time increases `+1.5–3.1 ms` (`+8–14%`)**.
6. Bounding `slots` directly bounds this accumulation in `occupancy` (`t=4`, 6 rounds): `slots = 4` → peak RSS `33.3 MB` (`ready` max 4 blks, `3.6 MB`); `slots = 8` → `35.8 MB`; `slots = 16` → `35.9 MB` median (spiking to `40–44 MB` whenever head-of-line blocks pile up).

### 3c. Cross-run proof across 10 paired CI runs + 3 `#1034` CI runs (210 `pattern t=4` executions)

Every downloaded paired CI run from `master` exhibits the exact same bimodal correlation:

| CI run | Change (scope vs `pattern`) | CPU (4 vCPU) | `hi-RSS (>38.5M)` `base` vs `head` | `r(rss,wall)` | `r(rss,runq)` | `r(rss,cpu)` | CI `pattern t=4` wall [95% CI] |
|---|---|---|---:|---:|---:|---:|---|
| **`45933661519`** | **PR #1033 (`src/python.rs` ONLY — true NULL)** | EPYC 9V74 | `2/10` vs `6/10` | `+0.83` | `+0.42` | `−0.31` | **`+6.0% [+0.2%, +12.2%]` slower**, RSS `+5.9%` |
| `45947374790` | A6b feature-gated (true NULL) | EPYC 9V45 | `3/10` vs `6/10` | `+0.55` | `+0.70` | `−0.19` | `+5.4% [−6.4%, +18.6%]` |
| `45944904862` | A6b feature-gated (true NULL) | Xeon 6975P-C | `4/10` vs `2/10` | `+0.33` | `+0.58` | `−0.61` | `−0.2% [−6.7%, +6.8%]` |
| `45928216036` | A6b feature-gated (true NULL) | EPYC 7763 | `5/10` vs `2/10` | `+0.68` | `+0.83` | `−0.67` | `−1.9% [−6.1%, +2.3%]` |
| `45923510226` | Open precheck (inactive on `pattern`) | EPYC 9V45 | `5/10` vs `7/10` | `+0.77` | `+0.75` | `−0.56` | `+4.1% [−3.7%, +12.6%]` |
| `45924113840` | `mtf-fold` bench (feature-gated) | EPYC 7763 | `2/10` vs `5/10` | `+0.73` | `+0.69` | `−0.67` | `+1.9% [−2.7%, +6.8%]` |
| `45926987285` | `mtf-fold` bench (feature-gated) | EPYC 9V74 | `2/10` vs `5/10` | `+0.84` | `+0.56` | `−0.58` | `+0.8% [−5.4%, +7.4%]` |
| `45948941328` | `#1080` Segmented build | Xeon 6975P-C | `3/10` vs `3/10` | `+0.59` | `+0.54` | `+0.07` | `+2.9% [−1.6%, +7.6%]` |
| `45921211004` | `#1034` earlier PR run | Xeon 6975P-C | `1/10` vs `2/10` | `+0.84` | `+0.41` | `−0.53` | `+1.6% [−1.7%, +5.0%]` |
| **`45947633032`** | **`#1034` exact-head PR run (flagged)** | Xeon 8573C | **`0/10` vs `6/10`** | **`+0.82`** | **`+0.78`** | **`−0.63`** | **`+7.3% [+2.6%, +12.3%]` slower**, RSS `+7.9%` |
| `45949114795` | `#1034` master push (`5ed20c1d`, single arm) | EPYC 9V74 | — vs `3/10` | — | — | — | `lo 18.9 ms` / `hi 21.0–22.7 ms` |

Key summary statistics:
- **4 true null controls (`80` executions, 0 Rust decoder lines changed):** `base 14/40 (35.0%)`, `head 16/40 (40.0%)`, pooled **`30/80 (37.5%)`** high-RSS spike rate; high-RSS wall penalty `+2.4%` to `+14.5%`.
- **All 3 independent CI runs of `#1034` (`30` executions):** `2/10 + 6/10 + 3/10 =` **`11/30 (36.7%)`** high-RSS spike rate — identical to the null-control rate (`37.5%`).
- **Bootstrap over the 4 true null controls (50,000 simulated 10-round paired runs):** one-sided false-positive rate on `pattern t=4` alone is **`2.20%`** for `95% CI > 0%`, **`1.91%`** for `>= +5%` with `95% CI > 0%` (triggering `PERFORMANCE-POLICY.md` §2 rule 3), and **`1.20%`** for `>= +7%` with `95% CI > 0%`. PR #1033 (`45933661519`) is a live realization of this exact false positive on zero Rust changes.

### 3d. Local VM paired reproduction (`5ed20c1d` vs `5ed20c1d^`, baseline / native / JCC-32B)

- Static JCC 32B boundary audit (`objdump -d`): `#1034` does **not** worsen JCC alignment of `pattern`'s worker functions (`unrle_in_place`: `12/60 → 10/60` crossings; `CellsKernel::<false>::invert_runs`: `1/8 → 0/8` crossings), and Xeon 8573C / AMD EPYC CI runners are not affected by the Skylake JCC erratum.
- Local paired `ab child` runs on `pattern.bz2` (VM diagnostic, 2 vCPU; quiet `N = 30` at `t=4` + `N = 20` across `t=1, 2, 4`):
  - Quiet `N = 30`, `t=4`, **baseline x86-64**: wall `64.60 → 63.60 ms` (`−5.63%` geomean), CPU `103.7 → 102.1 ms`, RSS `36.6 → 36.9 MB` (`+0.74%`), `hi>38.5M`: `8/30` vs `13/30`.
  - Quiet `N = 30`, `t=4`, **JCC-32B aligned**: wall `52.05 → 52.00 ms` (`+0.84%` geomean), CPU `101.0 → 100.7 ms`, RSS `37.0 → 37.4 MB` (`+0.45%`), `hi>38.5M`: `13/30` vs `14/30`.
  - `N = 20`, **`-C target-cpu=native`**: `t=1` wall `+0.20%`, CPU `−0.67%`; `t=4` wall `+1.50%` `[−7.85%, +11.80%]`, CPU `−1.53%`, RSS `+3.0%`.
  - No reproducible wall or CPU regression in any build mode (`base`, `native`, `jcc`); both `5ed20c1d^` and `5ed20c1d` exhibit the same bimodal `36 MB` vs `40+ MB` RSS spikes locally.

---

## 4. Recommendations

1. **Flag `2026-11-27-scanner-coalesced-ci-acceptance.md`:** This probe refutes the scanner fixed-cost hypothesis (`scanner-per-block-fixed-cost`: **REFUTED**) and establishes that `#1034`'s `+7.3%` 4T `pattern` CI result is a stochastic draw (`0/10` vs `6/10`) of a pre-existing 6-on-4 `ready`-backlog scheduling regime that also fired `+6.0% [+0.2%, +12.2%]` on Python-only PR #1033 (`45933661519`). No `#1034` scanner revert or `pattern`-targeted scanner tweak is warranted; `scanner-a2-fast-build` should proceed on its own `wikidata`/`text` merit, not as a `pattern` fix. Per the claim's scope, the flag remains `status: open` with this evidence appended for policy disposition by an independent reviewer.
2. **Structural fix (`ready-cap-worker-gate`, `#1094`):** Gating worker output-buffer issuance on ready bytes / ready backlog (rather than only gating scanner admission) directly addresses the root cause of both `pattern t=4`'s bimodal `+3..8 MB` / `+8..14%` wall spikes and `#1094`'s run-only memory amplification, without altering benchmark corpora or policy thresholds (`PERFORMANCE-POLICY.md` §2).
