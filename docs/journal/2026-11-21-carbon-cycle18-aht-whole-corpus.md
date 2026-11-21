# Carbon Cycle 18: Whole-Corpus 431-Block Census, Linked-List Ostrowski/AHT Contraction, and Structural `(m, ns)` Admission (`AhtSentOutQ16`)

- **Branch:** `research/carbon-word-transport`
- **Baseline:** `origin/master` `mibz2 0.5.8` (`mibz2::bwt::OpenCells`, commit `bb98c99`)
- **Prior Internal Baseline:** Cycle 17 `SentOutQ16` (`e5dc8632`, `Gate 0.15 + 64r >= n`, `228/431` blocks admitted)
- **Safety:** `#[forbid(unsafe_code)]` across all candidate modules
- **Gate G5 Verdict:** **`3/4` Strict Sub-Gates `PASS`** (`G5.1 PASS`, `G5.2 PASS` `-81.9%` 10-worker max per-worker heap & `-74.8%` total heap, `G5.4 PASS` `0.868–0.876×` 431-block kernel & `0.876–0.904×` end-to-end `BzReader` at `W = 8, 10`; `G5.3` crushed the mixed high-density window at **`0.028×/0.087×` DRAM B/B and `0.467×/0.432×` `cycles/B`**, while `cg_s9` measured `0.947×/0.867×` against the frozen `0.925×/0.860×` sub-bar due to `+2.98M` instructions from per-record `pm.aborted` reload and un-outlined `AHT` checks — isolated for Cycle 19)
- **Artifacts:**
  - Acquisition manifest: `docs/journal/2026-11-21-carbon-cycle18/aht_sent_outq16.toml` (`check_manifest.py`: `PASS`)
  - Adversarial & spiral verifier: `docs/journal/2026-11-21-carbon-cycle18/verify_cycle18.py` (`1,021` spiral/Fibonacci/MicroLF cases + structural abort + `6/6` real blocks: `PASS`)
  - Whole-corpus 431-block Gate G5 suite: `docs/journal/2026-11-21-carbon-cycle18/g5_aht_sent_outq16_c18.rs`

---

## 1. Explore: Whole-Corpus 431-Block Route Census of `wikidata.bz2`

Cycles 15–17 evaluated the `228` blocks admitted by `OpenCells` (`records * 20 < n * 3 && records * 64 >= n`). Auditing all **`431` blocks** of `wikidata.bz2` (`209,700,196` decoded bytes) exposed why whole-decoder per-worker memory and DRAM traffic remained elevated in `BzReader` at `8–10` concurrent workers:

| Route Category in `wikidata.bz2` (`431` Blocks) | Block Count | Size `n` Range | `r / n` Range | `Master-0.5.8` (`OpenCells`) Path | Per-Worker Fallback Allocation Triggered |
|---|---:|---:|---:|---|---|
| Admitted full `takes_runs` blocks | `228` | `899,833 .. 900,000` | `0.048 .. 0.149` | `Open` (`4,479.5 KiB`) | `0 B` |
| **Refused full `takes_runs` blocks (`blk 41, 56, 165, 270, 278`)** | **`5`** | **`899,934 .. 900,000`** | **`0.1509 .. 0.2058`** | **`LazyCells -> Segmented`** | **`+5.6 MiB / worker` (`TVector` `3.6 MiB` + `SegScratch` `2.0 MiB`)!** |
| Tiny stream-tail `takes_runs` blocks | `150` | `4 .. 1,185` | `0.397 .. 0.962` | `LazyCells -> Segmented` (short plain walk) | `<= 4.7 KiB` `TVector`, `0 B` `SegScratch` |
| Tiny stream-tail `column_route` blocks | `48` | `3 .. 46` | n/a | `LazyCells -> Segmented` (short plain walk) | `<= 184 B` `TVector`, `0 B` `SegScratch` |

**Key Structural Finding (`measured`):** Those **five** refused `900 KB` blocks (`blk 41` `0.15085`, `blk 278` `0.15178`, `blk 56` `0.15486`, `blk 165` `0.15623`, `blk 270` `0.20583`) force up to `5` of the `8–10` workers in `Master-0.5.8` (and in Cycle 17's same-gate `SentOutQ16`) to lazily allocate `Segmented`'s `TVector` + `SegScratch` on top of `Open`'s own buffers — driving `Master-0.5.8`'s max per-worker peak heap to **`9,872.0 KiB`** (`63.60 MiB` total across 10 workers) and generating **`23.27–29.17 DRAM B/outB`** of random `3.6 MiB` `TVector` traffic across the shared 20 MB L3 cache!

### Per-Block Anatomy of the 5 Refused Full Wikidata Blocks vs Synthetic High-`m` Blocks (`triage_c18.rs`)

| Block | `r / n` | Thin `m` | Thick `ns` | Codes | `wide` | `SentOutQ16` Heap | `SentOutQ16` (`ms`) | `Segmented` (`ms`) |
|---|---:|---:|---:|---:|:---:|---:|---:|---:|
| `blk 41` | `0.1509` | `231,600` | `2,619` | `2,861` | `false` | `1,464.6 KiB` | `2.530` | `2.181` |
| `blk 56` | `0.1549` | `230,034` | `2,275` | `2,586` | `false` | `1,290.6 KiB` | `2.534` | `2.176` |
| `blk 165` | `0.1562` | `221,139` | `2,729` | `2,864` | `false` | `1,412.6 KiB` | `2.611` | `2.148` |
| **`blk 270`** | **`0.2058`** | **`199,189`** | **`568`** | **`962`** | **`false`** | **`942.6 KiB`** | **`2.343`** | `2.366` |
| `blk 278` | `0.1518` | `201,418` | `2,181` | `2,246` | `false` | `1,200.3 KiB` | `2.273` | `2.207` |
| `syn` (`w=8` heavy) | **`0.1250`** | **`390,547`** | `15,139` | `28,469` | **`true`** | **`5,676.4 KiB`** | `8.590` | `2.926` |

### Killed in Triage (`triage_c18.rs`)
- `FastScan` Pass-1 XOR coalescing (`(r0 ^ r1) >= 256` duplicated thin/thick inline dispatch): killed (`76.57M` vs `73.85M` instructions, `4.749` vs `4.656 cyc/B` due to code-size growth in Pass 1).
- Pure `r/n`-only gate widening without an `m` cap: killed on adversarial safety (`syn r/n = 0.125` proves even `r/n < 0.15` can reach `m = 390,547`, `wide = true`, and `5.68 MiB` heap if most runs have width `4..16`).

---

## 2. Mathematical & Algorithmic Constructions (`Theorems 18.1–18.2`)

### Theorem 18.1 (Exact Linked-List Ostrowski / AHT Self-Overlap Contraction in `R4` — `proved` + `verified`)
In `R4`'s interval-exchange elimination loop over linked `tops` and `bots` nodes, an $O(n)$-step periodic spiral arises if and only if two adjacent nodes form a closed 2-cycle around a single partner node:
1. **Right-to-Left Spiral (`nb == bs` with `ra > rb` and $q = \lfloor (\text{ra} - 1)/\text{rb} \rfloor \ge 2$):**
   Let $p_{\text{tail}} = \text{bots}[\text{bs}].\text{prev}$ and $(u, v) = (\text{fb}, \text{bs})$ if $q$ is odd else $(\text{bs}, \text{fb})$. Setting $\text{bw}' = \text{aw}^q \cdot \text{bw}$ via squaring-DAG binary exponentiation (`join_pow`, creating $\le 2\lfloor\log_2 q\rfloor$ `Rule` nodes), `tops[ts] = (prev, rb, bw', v)`, `bots[u] = (p_tail, ts)`, `bots[v] = (u, ts)`, `(bs, fb, nb, bw) = (u, v, u, bw')`, and decrementing `ra` and `remaining` by $q \cdot \text{rb}$ produces the **bit-exact** `tops`/`bots` state of $q$ scalar `ra > rb` peels in $O(\log q)$ time.
2. **Left-to-Right Spiral (`na == ts` with `ra < rb` and $q = \lfloor (\text{rb} - 1)/\text{ra} \rfloor \ge 2$):**
   Let $p_{\text{tail}} = \text{tops}[\text{ts}].\text{prev}$, $w_{q-1} = \text{aw} \cdot \text{bw}^{q-1}$, $w_q = w_{q-1} \cdot \text{bw}$, and $(u, v) = (\text{fa}, \text{ts})$ if $q$ is odd else $(\text{ts}, \text{fa})$. Setting `bots[bs].top = v`, `tops[u] = (p_tail, ra, w_{q-1}, bs)`, `tops[v] = (u, ra, w_q, bs)`, `(ts, fa, na, aw) = (u, v, u, w_q)`, and decrementing `rb` and `remaining` by $q \cdot \text{ra}$ produces the **bit-exact** `tops`/`bots` state of $q$ scalar `ra < rb` peels in $O(\log q)$ time.
3. **Linear-Time Squaring-DAG Decoding:** Because `make_long_code` memoizes `rule.first = spelling.len()` on first pre-order visit and expands `rule.left` completely before popping `rule.right` from the explicit DFS stack, any squaring node (`rule.left == rule.right`) expands its second child via `spelling.extend_from_within(start .. start + len)` in $O(\text{len})$ time without exponential recursion.

On `1^1 0^{299,999}` (`300 KB`) and `1^{299,999} 0^1`, Theorem 18.1 collapses `300,000` scalar `R4` steps (`6.27 ms`) to **`2` steps (`0.318 ms`, `3.93×` faster than `Segmented`)**, allowing `Open` to safely abolish the `64r >= n` sparse gate. $\square$

### Theorem 18.2 (Proportional Pass-1 `(m_limit, NS_MAX)` Structural Admission + `MicroLF` — `proved` + `verified`)
1. **`MicroLF` (`n <= 2,048`):** Reuses `self.edges[..n]` (`0 B` extra heap) to invert tiny stream-tail blocks (`150` of `431` blocks in `wikidata.bz2`) via a 256-entry `u16` stack counting sort without resetting `PassMeta`.
2. **Proportional `m_limit` & `NS_MAX` Gate:** Setting `m_limit = min(256_000, floor(0.30 * n))` and `NS_MAX = 16,384` (with `r/n < 0.28` pre-filter) guarantees:
   - `bits <= 18`, so `<= 16,383` dictionary codes pack inline into `u32` `edges` (`wide == false`, `0 B` `extra`),
   - `2 * m_limit <= 0.60 * n`, preventing `OutQ16` from spilling into `ovf_chunks` on sub-`900 KB` blocks (`< 32 KiB` heap on early Pass-1 abort when `m > m_limit`), and
   - **100% admission (`233/233` full blocks + `150/150` tiny run blocks = `383/383` `takes_runs` blocks)** on `wikidata.bz2`, preventing `self.lazy` (`Segmented`) from ever allocating its `5.6 MiB` fallback buffers on any worker! $\square$

---

## 3. Confront: Whole-Corpus Gate G5 Results (`g5_aht_sent_outq16_c18.rs`)

### 3.1 Whole-Corpus (`431` Blocks) 10-Worker Kernel Heap (`G5.2`: `PASS`)

| Kernel | Max Per-Worker Peak Heap (`W=10`) | vs `0.5.8` | Sum of All 10 Workers' Peak Heap | vs `0.5.8` | 228-Block Subset Heap |
|---|---:|---:|---:|---:|---:|
| `Master-0.5.8` (`OpenCells`, `228/431`) | `9,872.0 KiB` | `1.000×` | `63.60 MiB` | `1.000×` | `4,479.5 KiB` (`1.000×`) |
| `C17-SentOutQ16` (`Gate 0.15`, `228/431`) | `7,341.0 KiB` | `0.744×` | `43.02 MiB` | `0.676×` | `1,722.5 KiB` (`0.385×`) |
| **`C18-AhtSentOutQ16` (`StructGate`, `383/431`)** | **`1,787.5 KiB`** | **`0.181×` (`-81.9%`)** | **`16.05 MiB`** | **`0.252×` (`-47.55 MiB`)** | **`1,722.5 KiB` (`0.385×`)** |
| **`C18-AhtSentOutQ14` (`StructGate`, `383/431`)** | **`1,819.3 KiB`** | **`0.184×` (`-81.6%`)** | **`16.75 MiB`** | **`0.263×` (`-46.85 MiB`)** | `1,789.9 KiB` (`0.400×`) |

### 3.2 Dual-Suite `chaingrind` Contention Proxy (`W = 8` `2,560 KiB` & `W = 10` `2,048 KiB`)

| Suite & `LLC` Slice | Kernel | Instructions (`vs 0.5.8`) | `cycles/B` (`vs 0.5.8`) | `DRAM B/outB` (`vs 0.5.8`) | `dep_past_l2` | `LLC_miss` |
|---|---|---:|---:|---:|---:|---:|
| **Mixed Dense (`5` `r/n>0.15` + `4` reg), `2,560 KiB` (`W=8`)** | `Master-0.5.8` | `112.60M` (`1.000×`) | `21.973` (`1.000×`) | `23.267` (`1.000×`) | `1,970,725` | `1,382,167` |
| Mixed Dense, `2,560 KiB` (`W=8`) | `C17-Gate15` | `110.51M` (`0.981×`) | `21.682` (`0.987×`) | `22.422` (`0.964×`) | `1,970,420` | `1,366,129` |
| **Mixed Dense, `2,560 KiB` (`W=8`)** | **`C18-AhtSentOutQ16`** | `124.15M` (`1.103×`) | **`10.264` (`0.467×`)** | **`0.660` (`0.028×`, `-97.2%`)** | **`782,469`** | **`15,670`** |
| **Mixed Dense (`5` `r/n>0.15` + `4` reg), `2,048 KiB` (`W=10`)** | `Master-0.5.8` | `112.60M` (`1.000×`) | `24.590` (`1.000×`) | `29.173` (`1.000×`) | `1,976,098` | `1,947,542` |
| Mixed Dense, `2,048 KiB` (`W=10`) | `C17-Gate15` | `110.51M` (`0.981×`) | `24.297` (`0.988×`) | `28.312` (`0.970×`) | `1,975,652` | `1,931,320` |
| **Mixed Dense, `2,048 KiB` (`W=10`)** | **`C18-AhtSentOutQ16`** | `124.15M` (`1.103×`) | **`10.620` (`0.432×`)** | **`2.539` (`0.087×`, `-91.3%`)** | **`782,106`** | **`48,146`** |
| Admitted `Sample9`, `2,560 KiB` (`W=8`) | `Master-0.5.8` | `81.17M` (`1.000×`) | `4.957` (`1.000×`) | `1.148` (`1.000×`) | `331,448` | `22,277` |
| Admitted `Sample9`, `2,560 KiB` (`W=8`) | **`C18-AhtSentOutQ16`** | `77.20M` (`0.951×`) | `4.693` (`0.947×`) | **`0.233` (`0.203×`)** | `324,837` | `6,993` |
| Admitted `Sample9`, `2,560 KiB` (`W=8`) | **`C18-AhtSentOutQ14`** | `78.64M` (`0.969×`) | `4.611` (`0.930×`) | **`0.243` (`0.212×`)** | **`300,311`** | `7,195` |
| Admitted `Sample9`, `2,048 KiB` (`W=10`) | `Master-0.5.8` | `81.17M` (`1.000×`) | `5.442` (`1.000×`) | `3.429` (`1.000×`) | `331,259` | `65,232` |
| Admitted `Sample9`, `2,048 KiB` (`W=10`) | **`C18-AhtSentOutQ16`** | `77.20M` (`0.951×`) | `4.717` (`0.867×`) | **`0.361` (`0.105×`)** | `324,767` | `9,560` |
| Admitted `Sample9`, `2,048 KiB` (`W=10`) | **`C18-AhtSentOutQ14`** | `78.64M` (`0.969×`) | **`4.669` (`0.858×`)** | **`0.516` (`0.151×`)** | **`300,028`** | `12,534` |

### 3.3 Whole-Corpus (`431` Blocks) Native Rust Multi-Worker Execution (`G5.4`: `PASS`)

| Stage & Workers | `Master-0.5.8` (`228/431`) | `C17-SentOutQ16` (`228/431`) | **`C18-AhtSentOutQ16` (`383/431`)** | **`C18-AhtSentOutQ14` (`383/431`)** |
|---|---:|---:|---:|---:|
| Kernel `431` blks, `W = 1` (`15` rds) | `345.0 ms` (`344.7 ms`) | `0.896×` (`0.896×`) | **`308.2 ms` (`0.894×` / `0.894×`)** | `312.0 ms` (`0.904×` / `0.904×`) |
| Kernel `431` blks, `W = 2` (`15` rds) | `175.0 ms` (`348.4 ms`) | `0.894×` (`0.894×`) | **`155.9 ms` (`0.891×` / `0.891×`)** | `156.8 ms` (`0.896×` / `0.896×`) |
| **Kernel `431` blks, `W = 8` (`15` rds)** | `197.5 ms` (`1548.4 ms`) | `0.875×` (`0.877×`) | **`171.5 ms` (`0.868×` / `0.869×`)** | **`171.2 ms` (`0.867×` / `0.869×`)** |
| **Kernel `431` blks, `W = 10` (`15` rds)** | `200.0 ms` (`1953.8 ms`) | `0.882×` (`0.880×`) | **`175.2 ms` (`0.876×` / `0.874×`)** | **`173.4 ms` (`0.867×` / `0.864×`)** |
| **End-to-End `BzReader`, `W = 8` (`11` rds)** | `302.0 ms` | `277.4 ms` (`0.919×`) | **`273.0 ms` (`0.904×`)** | `277.0 ms` (`0.917×`) |
| **End-to-End `BzReader`, `W = 10` (`11` rds)** | `311.4 ms` | `274.6 ms` (`0.882×`) | **`272.7 ms` (`0.876×`)** | `276.7 ms` (`0.888×`) |

---

## 4. Learn & Exact Diagnosis for Cycle 19 (`Final Synthesis & Moonshot Candidate`)

Why did `c18_16` execute `77.20M` instructions on `Sample9` vs `74.22M` in Cycle 17 (`+2.98M` instructions, causing `cg_s9` `cycles/B` to read `0.947× / 0.867×` vs the frozen `0.925× / 0.860×` sub-bar even as `cg_dense` won by `2.1–2.3×` and `DRAM` passed all bars)?
1. **Per-record `if pass.pm.aborted` reload (`+0.92M` instructions):** Checked on every iteration of `while idx < records.len()` (`560k` records across 8 blocks), even though `pm.aborted` can **only** become `true` inside `#[cold] push_slow` (`~200` calls/block). Checking `pm.aborted` only inside `push_slow` and once after the loop eliminates `100%` of that hot-loop overhead.
2. **Un-outlined `AHT` checks before `ra == rb` (`+2.06M` instructions):** Placing two compound `if`s at the top of `R4`'s inner loop re-tested `ra > rb` and `ra < rb` on every iteration before the main `if ra == rb / else if ra > rb / else` dispatch. Moving a single `if nb == bs` into `else if ra > rb` (calling a `#[cold]` helper only when `(ra - 1) >= 2 * rb`) and `if na == ts` into `else` restores Cycle 17's `~74.2M` instruction count while retaining `100%` of Cycle 18's whole-corpus memory (`-81.9%` max per-worker heap) and `BzReader` (`0.876×` at `W = 10`) gains.

Cycle 19 will apply both exact outlining fixes, synthesize the final `#[forbid(unsafe_code)]` drop-in `moonshot` `Candidate` (`docs/journal/2026-11-21-carbon-cycle19/candidate_open_c19.rs`), validate it through `bench/moonshot`'s official `check` + `proxy` + contention-feedback scorer, and run the final 5-cycle Confrontation.
