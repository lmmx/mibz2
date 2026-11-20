# Carbon Cycle 11: Bijection In-Degree Conservation, Tight Radix-Slab Queueing from `Runs.{counts, per}`, and `4 KiB` Bipolar Dead-`out` Splicing (`FlatQ-BP4K` — Gate G5 `PASS`)

*20 November 2026. Session: Carbon (`research/carbon-word-transport`). Mode: Reorientation (default, `METHOD.md` §3a).*

## 1. Cycle Orientation & Summary of Outcomes

At the Cycle 10 → 11 boundary (`04b68a7`), read-only inspection of `origin/master` (`4ab98cd`, Coordinator's first-pass audit of `research/carbon-depth`) reinforced two core methodological rules:
1. Combinatorial lower bounds must be **pure model-free mathematical invariants** (never "millisecond floors" derived from asserted per-element cycle counts).
2. Read-only inspection of `src/open.rs:416` (`fn invert_runs(&mut self, r: &Runs, ...)`) and `src/runs.rs:91-92` revealed a decisive cross-stage structural leak: **`Runs::front` already computes exact per-byte row counts `counts: [u32; 256]` AND exact per-byte record counts `per: [u32; 256]` before `Open` is invoked**, yet `OpenCells::invert_runs` discards both arrays and passes only `r.records()`!

Cycle 11 proved a new model-free bijection conservation obstruction (Theorem 11.1), formulated **Tight Radix-Slab Queueing + Primary-Split Scanning (`FlatQ`, Theorem 11.2)** and **`4 KiB` Page-Granular Bipolar Dead-`out` Splicing (`BP4K-BVSMC-1P`, Theorem 11.3)**, validated them via `check_manifest.py` (`flatq_bp4k.toml`), `verify_cycle11.py`, and full AddressSanitizer/LeakSanitizer/UBSan, and achieved a **unanimous `6/6` pre-registered Gate G5 `PASS`** across all eight sub-criteria in `g5_flatq_bp4k.c`:

1. **Theorem 11.1 — Measure-Preserving Bijection In-Degree Conservation / No-Hub Law (`proved` & `verified`):**
   Proved that because any retained BWT first-return map $g : [0, m) \xrightarrow{\sim} [0, m)$ (in $L$-space or $F$-space) is a bijection ($\deg^-(v) = \deg^+(v) = 1$), counting measure $\mu(U) = |U|$ is strictly $g$-invariant: $|g^{-1}(U)| = |U|$ regardless of symbol skew $C[0..256)$. Consequently, **any** L1D-resident vertex-bypass table of capacity $K \le B_{\text{L1}}/4 = 12,288$ intercepts at most $K / m \in [5.15\%, 10.07\%]$ of orbit steps on `W24` Wikidata blocks (`verified` across `1,385` small columns and `6/6` blocks), ruling out hub-shortcutting on retained thin permutations without asserted cycle constants.
2. **Theorem 11.2 — Tight Pre-Allocated Radix-Slab Queueing from `Runs.{counts, per}` & Primary-Split Trichotomy (`FlatQ`, `proved`, `verified`, & `measured`):**
   - Proved the exact *a priori* envelope $\mathrm{thin}[a] \le \min(\mathrm{counts}[a],\; L \cdot \mathrm{per}[a]) + 1$ for every byte $a \in [0, 256)$. Partitioning a single contiguous slab `flat_q` by $\min(\mathrm{counts}[a], 24 \cdot \mathrm{per}[a]) + 36$ eliminates **256 separate `Vec<u32>` allocations**, **all per-run capacity checks**, and **four of the five per-byte memory accesses (`queues[c]`, `q_len[c]`, `q_cap[c]`, `all_c[c]`)** from `97%+` of runs (leaving only `q_cur[c]` in `2 KiB` L1D, while reconstructing $\mathrm{all\_c}[c] = (q_{\text{cur}}[c] - q_{\text{start}}[c]) + \mathrm{thick\_rows}[c]$ lazily on the `1.5–3%` thick strips).
   - Splitting the scan at `pos + 128 <= p` / straddle / `pos > p` eliminates primary-index comparisons from `>99.9%` of runs.
3. **Theorem 11.3 & Gate G5 Unanimous `PASS` (`FlatQ-BP4K`, `g5_flatq_bp4k.c`):**
   - Raising bipolar dead-`out` physical page size from `2 KiB` to OS page size **`4 KiB` (`BP4K_PAGE = 4096`)** halves `cold` page-boundary transitions and halves Kahn page-graph vertices (`<= 220` pages; `Kahn = 40–46 µs`) while keeping peak recycled overflow at **`188–204 KiB` (`47–51` pages `<= 256 KiB` on `6/6` blocks)**.
   - Across 25 paired interleaved rounds (`g5_flatq_bp4k.c`), **`RunsMeta-FlatQ+BP4K` (Arm 6) passed Gate G5 `6/6` unanimously**:
     - **`0.444× – 0.573×` shipped `Open-qi16` total** (`0.872 ms` on `block-0`, `1.001 ms` on `block-85`, **`1.540 ms` vs `3.472 ms` = `0.444×` / `2.25×` speedup on `block-143`!**), with `construct` down to **`0.374× – 0.468×` `Open-qi16`**;
     - **`0.554× – 0.680×` calibrated `Open+U16Q-1L` total**;
     - **`0.754× – 0.842×` Cycle 7's promoted `GIM-RV(L16)+BV2P` total** (`15.8%–24.6%` faster on `6/6` blocks);
     - **`0.859× – 0.885×` Cycle 10's `W24-R4+BP1P(2K)` total** (`0.785× – 0.834×` `construct` on `6/6` blocks);
     - Even **`PaidPre-FlatQ+BP4K` (Arm 5)** — which ignores `Runs.{counts, per}` and pays its own `records` histogram pre-pass inside `construct` — beat Cycle 10 on **`6/6` blocks (`0.939× – 0.973×`)**, reaching **`0.484× – 0.615×` `Open-qi16`**.

---

## 2. Four Mandatory Candidate Fields (`METHOD.md` §3a)

- **Candidate:** Tight Radix-Slab `FlatQ` (`Runs.{counts, per}`) + Primary-Split `RunQ` + `4 KiB` Bipolar Dead-`out` 1-Pass Splicing (`FlatQ-BP4K`, promoted for Coordinator safe-Rust audit).
- **Mode:** Reorientation (eliminates cross-stage information loss between `Runs::front` and `Open::construct`, collapses 256 dynamic queue vectors + 5 per-byte state lookups into a single pre-partitioned radix slab + 1 L1D write cursor `q_cur[256]`, splits `RunQ` around `p`, and coarsens bipolar dead-`out` page unshuffling to `4 KiB` OS pages).
- **Displaced obligations:**
  1. `256` separate `Vec<u32>` heap buffers, per-run `len + 24 > cap` capacity branches, and per-thin-run reads/writes of `q_len[c]`, `q_cap[c]`, `thin[c]`, and `all_c[c]`;
  2. Per-run `p >= pos && p < end` primary-index branching across the `>99.9%` of runs outside `[p - 127, p]`;
  3. Half of all `2 KiB` page-boundary `cold` transitions and half of all Kahn page-graph vertices (`439 → 219` pages).
- **Replacement principle:** Read `&Runs.{counts, per}` (`src/runs.rs:91-92`) directly in `OpenCells::invert_runs` (`src/open.rs:416`), partition one reusable `flat_q` buffer by $\min(\mathrm{counts}[a], 24 \cdot \mathrm{per}[a]) + 36$ (Theorem 11.2), advance only `q_cur[c]` on thin runs while reconstructing $\mathrm{all\_c}[c]$ lazily via `thick_rows[c]` on the `1.5–3%` thick strips, run `R4-DGIM` at `LIMIT = 24`, and emit in 1 pass over `4 KiB` bipolar pages inside `out[0..n)` + `188–204 KiB` recycled overflow (Theorem 11.3).
- **New paid obligations:**
  1. Reserved virtual/reusable slab envelope of $\sum_a (\min(\mathrm{counts}[a], 24\cdot\mathrm{per}[a]) + 36) \times 4\text{ B} = 1,967\text{--}3,085\text{ KiB}$ per worker (of which only `488–954 KiB` = `4m` bytes are touched/dirtied in cache);
  2. `+14 to +38 KiB` slightly larger peak recycled overflow at `4 KiB` page granularity (`188–204 KiB` vs `154–184 KiB` at `2 KiB`, still well inside `256 KiB`).
- **Nearest prior art:** Production `src/runs.rs` + `src/open.rs` (`#664`/`#666` `Vec16` `U16Q`, which still used 256 dynamic vectors, discarded `Runs.{counts, per}`, and checked `p` + updated `all_c[c]` on every run) and Carbon Cycle 10's `W24-BP1P(2K)` (`04b68a7`).

---

## 3. Foundational Theorems

### Theorem 11.1 — Measure-Preserving Bijection In-Degree Conservation (`proved` & `verified`)

**Statement (`proved`):** Let $g : [0, m) \xrightarrow{\sim} [0, m)$ be any retained first-return permutation of a BWT column. Because $g$ is bijective, $|g^{-1}(U)| = |U|$ for every $U \subseteq [0, m)$. Hence for any primitive orbit $(j_0, \dots, j_{m-1})$ and any fast-path vertex subset $U$ of capacity $|U| \le K$, the exact fraction of transitions landing in $U$ is $|U|/m \le K/m$, independent of symbol frequency skew. At $B_{\text{L1}} = 48\text{ KiB}$ ($K = 12,288$), any vertex-bypass subset intercepts at most $5.15\%\text{--}10.07\%$ of `W24` Wikidata transitions. $\square$

---

### Theorem 11.2 — Tight Radix-Slab Queue Envelope & Lazy Rank Conservation (`proved` & `verified`)

**Statement (`proved`):** Given `counts[a]` $=\sum_{r: b_r = a} w_r$ and `per[a]` $= |\{r : b_r = a\}|$ from `Runs::front` (`CAP = 127`), under threshold $L \in [1, 127]$:
1. Every thin maximal run of byte $a$ consumes $\ge 1$ record of byte $a$ and emits $\le L$ thin rows, while at most one thick maximal run across $[0, n)$ contains $p$ and emits $1$ singleton thin row. Therefore,
   $$\mathrm{thin}[a] \;\le\; \min\!\big(\mathrm{counts}[a],\; L \cdot \mathrm{per}[a]\big) + 1 \qquad \forall a \in [0, 256).$$
2. At the start of any maximal run at column offset `pos`, the total count of byte $a$ in $L[0..\text{pos})$ equals $(q_{\text{cur}}[a] - q_{\text{start}}[a]) + \mathrm{thick\_rows}[a]$, where $\mathrm{thick\_rows}[a]$ is updated solely on thick strips; at `pos == n`, $(q_{\text{cur}}[a] - q_{\text{start}}[a]) + \mathrm{thick\_rows}[a] = \mathrm{counts}[a]$.
3. Since every single-record run has width $\le 127$, no single-record run starting at $\text{pos} \le p - 128$ can cover $p$, and no run starting at $\text{pos} > p$ can cover $p$. $\square$

---

### Theorem 11.3 — Executable Verification (`verified`, `verify_cycle11.py` & `flatq_bp4k.toml`)

- `check_manifest.py docs/journal/2026-11-20-carbon-cycle11/flatq_bp4k.toml`: **PASS**.
- `verify_cycle11.py`: **PASS** across `1,385/1,385` No-Hub columns, **`18,782/18,782` small runs** (with `cap = 3` forced multi-record primary straddles verifying zero Theorem 11.2 envelope violations), **`8/8` adversarial controls**, and **`6/6` authenticated Wikidata blocks** (`5,399,692 B` in `7.30s`).

---

## 4. Gate G3 Cost Vector & Gate G5 Unanimous `PASS` (`g5_flatq_bp4k.c`)

### Producer–Consumer Memory & Cost Accounting (Gate G3)

| Metric | Shipped `Open-qi16` (Arm 0) | Cycle 7 `GIM-RV+BV2P` (Arm 2) | Cycle 10 `W24-BP1P(2K)` (Arm 4) | Cycle 11 `RunsMeta-FlatQ+BP4K` (Arm 6) |
|---|---|---|---|---|
| Queue representation | `256` `Vec<u32>` + scalar push | `256` `Vec<u32>` + `U16Q` | `256` `Vec<u32>` + `U24Q` | **1 `flat_q` slab + `q_cur[256]` (`0` cap branches)** |
| Per-thin-run state updates | `queues`, `q_len`, `q_cap`, `thin`, `all_c` + `p` check | Same (`5` arrays + `cap` + `p` branch) | Same (`5` arrays + `cap` + `p` branch) | **Only `q_cur[c]` (`1` L1D pointer; `0` `p` checks outside `p`)** |
| Consumer paging & walks | `1.0m` serial 1-lane | `2.0m` 8-lane (`0 B` pool) | `1.0m` 8-lane (`2 KiB` pages, `154–184 KiB` ovf) | **`1.0m` 8-lane (`4 KiB` pages, `188–204 KiB` ovf)** |
| Touched active working set | `1,279.2 – 2,503.7 KiB` | `1,181.2 – 2,328.4 KiB` | `1,405.9 – 2,449.9 KiB` | **`1,435.9 – 2,487.9 KiB`** *(reserved slab `1,967–3,085 KiB`)* |

### Sanitizer & Paired 25-Round 7-Arm Confrontation (`measured`, `g5_flatq_bp4k.c`)

- **AddressSanitizer + LeakSanitizer + UndefinedBehaviorSanitizer (`-O1 -g -fsanitize=address,undefined`):** **PASS** (`0` warnings, `0` errors, `0` leaks across `8/8` adversarial self-tests incl. `pos+128` straddle and `50k >4KiB EpRule` + `6/6` Wikidata blocks across all `7` arms).
- **25-Round Paired Interleaved Benchmark (`gcc -O3 -march=native`, rotating arm order `(step + r + 3) % 7` after 3 warmup rounds; VM note: single-thread calibrated kernel timings on `Xeon Platinum 8481C` VM, not 20-thread native proof):**

| Block | Arm 0 `Open-qi16` (Con / Emit / Tot) | Arm 1 `Open+U16Q-1L` (Tot ms) | Arm 2 `GIM-RV+BV2P` (Tot ms) | Arm 4 `C10 W24-BP1P` (Con / Emit / Tot) | Arm 5 `PaidPre-FlatQ+BP4K` (Con / Emit / Tot) | Arm 6 `RunsMeta-FlatQ+BP4K` (Con / Emit / Tot, `/Open`, `/C10`, Ovf) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.991 / 0.536 / 1.523` | `1.295` | `1.050` | `0.498 / 0.483 / 0.987` | `0.471 / 0.462 / 0.936` | **`0.411 / 0.463 / 0.872` (`0.573×`, `0.883×`, `188 KiB`)** |
| `block-2.L` | `1.182 / 0.730 / 1.905` | `1.554` | `1.256` | `0.619 / 0.577 / 1.195` | `0.576 / 0.552 / 1.124` | **`0.512 / 0.552 / 1.057` (`0.555×`, `0.885×`, `196 KiB`)** |
| `block-85.L` | `1.030 / 0.919 / 1.952` | `1.572` | `1.288` | `0.546 / 0.604 / 1.154` | `0.525 / 0.584 / 1.102` | **`0.437 / 0.570 / 1.001` (`0.513×`, `0.867×`, `200 KiB`)** |
| `block-143.L` | `1.940 / 1.527 / 3.472` | `2.778` | `2.042` | `0.913 / 0.872 / 1.788` | `0.851 / 0.827 / 1.679` | **`0.725 / 0.815 / 1.540` (`0.444×`, `0.861×`, `192 KiB`)** |
| `block-210.L` | `1.534 / 1.438 / 2.973` | `2.462` | `1.844` | `0.883 / 0.850 / 1.745` | `0.823 / 0.820 / 1.644` | **`0.693 / 0.808 / 1.499` (`0.504×`, `0.859×`, `192 KiB`)** |
| `block-207.L` | `1.360 / 0.925 / 2.284` | `1.890` | `1.418` | `0.763 / 0.568 / 1.331` | `0.741 / 0.549 / 1.295` | **`0.636 / 0.530 / 1.164` (`0.510×`, `0.875×`, `204 KiB`)** |

### Pre-Registered Gate G5 Verdict: **`PASS` (`6/6` across all 8 sub-criteria)**

- **(1) Peak Overflow `<= 256 KiB` & Full Sanitizer Cleanliness:** **PASS (`6/6`: `188–204 KiB`)**.
- **(2) `RunsMeta-FlatQ+BP4K Tot <= 0.58× Open-qi16 Tot`:** **PASS (`6/6`: `0.444×, 0.504×, 0.510×, 0.513×, 0.555×, 0.573×`)**.
- **(3a/3b) `RunsMeta-FlatQ+BP4K Tot <= 0.85× GIM-RV(L16)+BV2P Tot`:** **PASS (`6/6`: `0.754× – 0.842×`)**.
- **(4a/4b) `RunsMeta-FlatQ+BP4K Tot <= 0.92× Cycle 10 W24-BP1P Tot`:** **PASS (`6/6`: `0.859× – 0.885×`)**.
- **(4c) `RunsMeta-FlatQ+BP4K Con <= 0.88× Cycle 10 W24-BP1P Con`:** **PASS (`6/6`: `0.785× – 0.834×`)**.
- **(4d) `PaidPre-FlatQ+BP4K Tot < 1.00× Cycle 10 W24-BP1P Tot`:** **PASS (`6/6`: `0.939× – 0.973×`)**.

---

## 5. Coordinator Audit Handoff Notes

`FlatQ-BP4K` unifies the discoveries of Cycles 2, 7, 9, 10, and 11 into a single coherent architecture ready for Coordinator safe-Rust verification:
1. **Minimal Producer Port (`FlatQ` + `p`-Split in `src/open.rs` + `src/runs.rs`):** Expose `pub(crate) fn counts(&self) -> &[u32; 256]` and `pub(crate) fn per(&self) -> &[u32; 256]` on `Runs` (`src/runs.rs:91-92`) and pass them into `Open::Kernel` at `src/open.rs:416`. Even before changing `emit`, `FlatQ` + `p`-split + `R4-DGIM` cuts `construct` to **`0.374× – 0.468×` of shipped `Open` (`0.53× – 0.63×` of `Open+U16Q`)** using `100%` safe slice indexing!
2. **Full Producer+Consumer Port (`U24Q` + `R4-DGIM` + `BP4K-BVSMC-1P`):** Pairing `LIMIT = 24` (`U24Q`) with 1-pass `4 KiB` bipolar dead-`out` splicing + Zero-Stage Kahn unshuffling reaches **`0.444× – 0.573×` of shipped `Open`** (`0.554× – 0.680×` of `Open+U16Q-1L`) while needing only `188–204 KiB` of overflow scratch instead of `1.23 MiB`.
