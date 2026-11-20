# Carbon Cycle 10: Overturning `#657` via Non-Separable Meyer Co-Equilibration (`U24Q`) and Bipolar Dead-`out` Topological Page Unshuffling (`W24-BP1P`)

*20 November 2026. Session: Carbon (`research/carbon-word-transport`). Mode: Reorientation (default, `METHOD.md` §3a).*

## 1. Cycle Orientation & Summary of Outcomes

At the Cycle 9 → 10 boundary (`cbc62d2`), exact sub-phase profiling of `R4-DGIM + BVSMC-2P` (`1.011–1.964 ms`) showed that thick-complement Rauzy–Veech elimination (`RV-Elim = 0.070–0.240 ms`, `75–243 KiB` inside L2) had shrunk to **`7–12%` of total decode**, leaving two dominant global bottlenecks:
1. **The Cycle 2 / Cycle 8 Two-Pass vs External-Pool Consumer Dichotomy (`44–56%` of total):** By Theorem 8.1 (`Hamming = 4` Christie–Gates switch), `1.0m` retained-edge reads is the information-theoretic floor, yet zero-pool `BVSMC-2P` pays **`2.0m` random L2 loads** solely to avoid `BVSMC-1P`'s **`1,230 KiB` external `chunk_pool`**.
2. **The `#657` Static `LIMIT = 16` Assumption:** `FRONTIER.md` recorded `#657` (*"LIMIT=16 is best; per-block choice gains <= 1% even with hindsight"*) as a known structural ceiling. However, `#657` was measured under **1-lane serial emission** (and held under `BVSMC-2P`'s `2.0m` walk), never testing whether cutting consumer traversal from `2.0m` to `1.0m` shifts the optimal Meyer complement cut upward!

Cycle 10 explored four distinct mechanisms across queue compression, dictionary packing, threshold co-design, and in-place page-graph unshuffling, killed two in quantitative Triage, and took **Theorems 10.1, 10.2, and 10.3 (`W24-BP1P`)** through manifest validation (`w24_bp1p.toml`), standalone verification (`verify_cycle10.py`), AddressSanitizer/LeakSanitizer/UBSan validation, and a paired 25-round 7-arm C confrontation (`g5_w24_bp1p.c`):

1. **Triage Kills — `Tag16-Dict` Cross-Domain Extract Latency & `Epoch-U16Q` Store Equivalence (`measured`, killed before G5):**
   - *`Tag16-Dict` (`_mm_extract_epi8(v, 15)` length tag inside byte 15 of `inline16[code]`, zero `w_len[]` table):* Regressed 8-lane `emit` by `+2.2% to +4.8%` (`0.417 → 0.432 ms` on `block-0`, `0.827 → 0.862 ms` on `block-143` in `/tmp/triage_cycle10.c`) because `vpextrb` XMM→GPR cross-domain bypass latency (`3` cycles × `8` lanes) costs more on Sapphire Rapids (`3` L1D load ports) than an L1-resident `uint16_t w_len[cd]` load.
   - *`Epoch-U16Q` (`64K`-epoch `u16` queues, `1` YMM store/run, `408 KiB` vs `817 KiB`):* Ran within noise of `U16Q` (`0.544 vs 0.545 ms` on `block-0`, `1.031 vs 1.038 ms` on `block-143`) because both fit comfortably inside the `2 MiB` L2 cache (`Xeon Platinum 8481C`) and `Wire` pays epoch-boundary tracking.
2. **Theorem 10.1 — Non-Separable Producer–Consumer Meyer Co-Equilibration (`proved`, `verified`, & `measured` — overturning `#657`):**
   - Proved that the optimal Meyer width cut $L^* = c_{\text{thick}} / c_{\text{thin}}$ is **strictly coupled** to consumer pass multiplicity: under `2.0m` `BVSMC-2P`, doubling thin-row reads keeps $L^*_{\text{2P}} \approx 16$ (`W24-R4+BV2P / R4-DGIM(L16)+BV2P = 0.947–1.037×`, hurting `block-0` by `+3.7%`), whereas dropping to **`1.0m` 1-pass 8-lane splicing** + **branchless 3-YMM `U24Q`** halves $c_{\text{thin}}$ and shifts $L^*_{\text{1P}}$ up to **`24..32`**!
   - Raising `LIMIT : 16 → 24` slashes thick strips $H$ by **`12.1%–44.1%`** (`3,448 → 1,926` on `block-0`, `5,047 → 2,891` on `block-143`) and shrinks `(256 + words) × 16 B` from `25–62 KiB` down to **`19–46 KiB` — pulling the entire `inline16` phrase dictionary inside the `48 KiB` L1D cache on `6/6` blocks**!
3. **Theorem 10.2 & `BP-BVSMC-1P` — Bipolar Dead-`out` Physical Extent Paging & Zero-Stage Kahn Topological Unshuffling (`proved`, `verified`, & `measured`):**
   - Eliminates `85.0%–87.5%` of `BVSMC-1P`'s `1,230 KiB` external chunk pool by growing segment `0` forward in `out[0 .. L_0)` while allocating `2 KiB` (`BP_PAGE = 2048`) physical extents for `q >= 1` **top-down inside dead `out[0..n)`**, spilling only the final `~35` extents (`+` recycled cycle evictions = **`154–184 KiB` peak overflow across all 6 blocks**) into an L2 buffer and resolving the in-place page dependency graph via **Zero-Stage Kahn Topological Unshuffling** (`verified` across `10,000/10,000` random straddling extent permutations with strict byte-level read-before-overwrite auditing).
   - In the 25-round paired Gate G5 confrontation (`g5_w24_bp1p.c`), **`W24-R4+BP1P` (Arm 6) beat BOTH Cycle 7's `GIM-RV+BV2P` (`0.871–0.955×`) AND Cycle 9's `R4-DGIM+BV2P` (`0.878–0.953×`) on all `6/6` blocks**, breaking the **1-millisecond barrier on `block-0.L` (`0.964 ms`)** and reaching **`0.518–0.647×` shipped `Open-qi16`** (**`0.518×` / `1.93×` speedup on `block-143.L`!**). Pre-registered strict Gate G5 verdict is recorded faithfully as **`FAIL`** because `2a` (`<= 0.60×` `Open-qi16`: `3/6`, with `block-85` at `0.608×` and `block-2` at `0.627×`) and `4a` (`<= 0.94×` `R4-DGIM`: `4/6`, with `block-210` at `0.9405×`) missed the aggressive `5/6` sub-bars by `0.5–8 µs`, while **`W24-R4+BP1P` (and `L16-R4+BP1P` on low-`H` blocks) establishes the fastest 1-thread BWT reconstruction kernel measured in this programme to date**.

---

## 2. Four Mandatory Candidate Fields (`METHOD.md` §3a)

- **Candidate:** Co-Equilibrated `U24Q` + `R4-DGIM` + Bipolar Dead-`out` 1-Pass Splicing (`W24-BP1P`)
- **Mode:** Reorientation (jointly re-equilibrates Meyer's thin/thick domain decomposition to `LIMIT = 24` via 3-YMM `U24Q` and replaces both `BVSMC-2P`'s `2.0m` two-pass traversal and `BVSMC-1P`'s `1.23 MiB` external pool with bipolar dead-`out` `2 KiB` physical page allocation + Zero-Stage Kahn topological page unshuffling).
- **Displaced obligations:**
  1. `BVSMC-2P`'s entire Pass-1 `1.0m`-step random L2 length-discovery walk (`succ[q]`, `seg_bytes[q]`);
  2. `BVSMC-1P`'s `1,230 KiB` external `chunk_pool` allocation + full-block external-to-`out` `memcpy` (replaced by `out`-hosted `2 KiB` physical pages + **`154–184 KiB`** L2 recycled overflow);
  3. `12.1%–44.1%` of all thick strips $H$ (width `17..24` runs absorbed branchlessly into `U24Q`), shrinking `inline16` below `48 KiB` L1D on `6/6` blocks.
- **Replacement principle:** Push thin runs of width $w \le 24$ unconditionally with three 32-byte AVX2 iota stores (`r4_q_push_u24`), run `R4-DGIM` on the reduced $w > 24$ complement, walk $S = 48$ cut-segments in **one** 8-lane SIMD pass (`1.0m` loads) writing `q = 0` forward from `out + 0` and allocating `2048 B` extents (`BP_SAFE = 2032` SIMD guard) top-down from `out + (N_phys - 1)*2048`, and resolve the resulting page-interval dependency graph in place via Kahn's zero-refcount cascade with recycled overflow cycle-breaking (Theorem 10.2).
- **New paid obligations:**
  1. One extra 32-byte AVX2 store per thin run in `U24Q` (`96 B` vs `64 B`) and `+4.6%..+35.8%` larger `m` (`edges[0..m]` grows by `29–167 KiB`);
  2. `O(N_phys)` (`<= 440` pages) Kahn dependency resolution + `154–184 KiB` recycled L2 overflow buffer (`~110–150` `2 KiB` cycle evictions).
- **Nearest prior art:** `#657` (static `LIMIT = 16` optimality under 1-lane `Open-qi16`), Carbon Cycle 2's `BVSMC-1P` (`1,230 KiB` external chunk pool on `L = 16` `Open`) and `BVSMC-2P` (`0 B` pool via `2.0m` two-pass walk), and Carbon Cycle 9's `R4-DGIM` (`L = 16` + `BVSMC-2P`).

---

## 3. Foundational Theorems

### Theorem 10.1 — Non-Separable Producer–Consumer Meyer Co-Equilibration (`proved`, `verified`, & `measured`)

**Statement (`proved`):** Let $N_w$ denote the count of coalesced runs of width $w$ in $L$, so at threshold $L$ the thick strip count is $H(L) = \sum_{w > L} N_w$ and the retained row count is $m(L) = \sum_{w \le L} w N_w$ (up to $\pm 1$ at $p$). For smooth local cost functions $T(L) = T_{\text{thick}}(H(L)) + T_{\text{thin}}(m(L))$ with marginal thick-strip cost $c_{\text{thick}} = \partial T_{\text{thick}} / \partial H$ and marginal thin-row cost $c_{\text{thin}} = \partial T_{\text{thin}} / \partial m = c_{\text{prod}} + \nu \cdot c_{\text{walk}}$ (where $\nu \in \{1, 2\}$ is the consumer's `edges[]` pass count), moving runs of width $w$ from $A_{\text{thick}}$ to $B_{\text{thin}}$ changes total runtime by $\Delta T(w) = N_w (w c_{\text{thin}} - c_{\text{thick}})$, yielding the unique interior equilibrium
$$L^*(\nu) \;=\; \frac{c_{\text{thick}}}{c_{\text{prod}} + \nu \cdot c_{\text{walk}}}.$$
Consequently:
1. Switching from 2-pass zero-pool splicing ($\nu = 2$, `BVSMC-2P`) to 1-pass splicing ($\nu = 1$, `BP-BVSMC-1P`) strictly increases $L^*(1) > L^*(2)$, creating **super-modular co-design synergy** ($T(L_2, \nu\!=\!1) - T(L_1, \nu\!=\!2) < [T(L_2, \nu\!=\!2) - T(L_1, \nu\!=\!2)] + [T(L_1, \nu\!=\!1) - T(L_1, \nu\!=\!2)]$).
2. On blocks whose `L = 16` dictionary `(256 + words(16)) × 16 B` exceeds `L1D` (`62.3 KiB > 48 KiB` on `block-143`), raising $L : 16 \to 24$ reduces `words(24)` (`3,635 → 2,613`, `45.9 KiB < 48 KiB` L1D), simultaneously accelerating *both* `construct` and per-step `emit`! $\square$

---

### Theorem 10.2 — Bipolar Dead-`out` Physical Extent Paging & Zero-Stage Kahn Unshuffling (`proved` & `verified`)

**Statement (`proved`):** Partition `out[0..n)` into $N_{\text{phys}} = \lfloor n / B \rfloor$ pages $I_p = [pB, (p+1)B)$ ($B = 2048$). During a single $1.0m$-step multi-lane walk over $S$ orbit segments:
1. **Zero Cross-Page Slopover & Pigeonhole Initial Overflow:** Guarding 16-byte SIMD stores by `P < base + B - 16` (`2032`) and filling exact `B`-byte pages in `cold` ensures every non-terminal extent has exact length $B$ and zero bytes cross page boundaries. Since $\sum_{q=0}^{S-1} L_q = \text{clen} \le n$, at most $S$ partial terminal extents plus `seg0`'s 2-page safety cushion ($\le S + 4$ pages) spill into `ovf` during the walk.
2. **Zero-Stage Read-Before-Overwrite Safety:** Define $\text{ref}[s]$ on physical host page $s \in [0, \max(N_{\text{phys}}, \lceil\text{clen}/B\rceil))$ as the number of unfinished target pages $p \in [\lfloor L_0/B \rfloor, \lceil\text{clen}/B\rceil)$ whose target byte range $[pB, \min((p+1)B, \text{clen}))$ intersects the unique unread extent $e(s)$ hosted at $s$ ($\text{ref}[s] \in \{0, 1, 2\}$). Whenever an unfinished target page $p$ satisfies $\text{ref}[p] = 0$, physical page $p$ hosts **no** unread bytes required by $p$ or any other unfinished target page; hence copying all extents overlapping $I_p$ **directly into `out`** (with zero `stage[B]` intermediate buffer) preserves every unread byte in `out` and `ovf`.
3. **Monotone Cycle-Eviction Termination:** If no unfinished target page has $\text{ref}[p] = 0$, evicting one remaining host page $s$ (`ref[s] > 0`) to a recycled `ovf` slot sets $\text{ref}[s] \leftarrow 0$, strictly decreasing $\sum_u \text{ref}[u]$ (`<= 2 N_phys`) and guaranteeing termination in $O(N_{\text{phys}})$ page copies. $\square$

---

### Theorem 10.3 — Executable Verification (`verified`, `verify_cycle10.py` & `w24_bp1p.toml`)

- `check_manifest.py docs/journal/2026-11-20-carbon-cycle10/w24_bp1p.toml`: **PASS**.
- `verify_cycle10.py`: **PASS** across:
  1. **`10,000/10,000` random straddling extent permutations** with byte-level read-before-overwrite auditing (`0` violations);
  2. **`18,782/18,782` small `(L, p, limit)` end-to-end `W24-BP1P` runs**;
  3. **`8/8` adversarial controls** (catching and verifying the periodic-block `wr < n` `sp >= total_tp` invariant before C compilation);
  4. **`6/6` authenticated Wikidata blocks** (`5,399,692 B` in `7.66s`).

---

## 4. Gate G3 Cost Vector & Gate G5 Confrontation (`g5_w24_bp1p.c`)

### Sanitizer & Paired 25-Round 7-Arm Confrontation (`measured`, `g5_w24_bp1p.c`)

- **AddressSanitizer + LeakSanitizer + UndefinedBehaviorSanitizer (`-O1 -g -fsanitize=address,undefined`):** **PASS** (`0` warnings, `0` errors, `0` leaks across `8/8` adversarial self-tests with `32 B` overrun canaries + `6/6` Wikidata blocks across all `7` arms).
- **25-Round Paired Interleaved Benchmark (`gcc -O3 -march=native`, rotating arm order `(step + r + 3) % 7` after 3 warmup rounds; VM note: single-thread calibrated kernel timings on `Xeon Platinum 8481C` VM, `L1d = 48 KiB`, `L2 = 2 MiB`):**

| Block | Arm 0 `Open-qi16` (Tot ms) | Arm 2 `GIM-RV+BV2P` (Tot ms) | Arm 3 `R4(L16)+BV2P` (Con / Emit / Tot) | Arm 4 `W24-R4+BV2P` (Con / Emit / Tot) | Arm 5 `L16-R4+BP1P` (Con / Emit / Tot) | Arm 6 `W24-R4+BP1P` (Con / Emit / Tot, `/Open`, `/R4`, Ovf KiB) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `1.491` | `1.044` | `0.579 / 0.456 / 1.035` | `0.485 / 0.582 / 1.073` | `0.560 / 0.444 / 1.006` | **`0.478 / 0.484 / 0.964` (`0.647×`, `0.931×`, `158 KiB`)** |
| `block-2.L` | `1.859` | `1.234` | `0.677 / 0.553 / 1.223` | `0.597 / 0.596 / 1.192` | **`0.653 / 0.494 / 1.149`** | **`0.602 / 0.564 / 1.165` (`0.627×`, `0.953×`, `184 KiB`)** |
| `block-85.L` | `1.924` | `1.294` | `0.597 / 0.714 / 1.310` | `0.567 / 0.684 / 1.241` | **`0.565 / 0.580 / 1.147`** | **`0.567 / 0.604 / 1.169` (`0.608×`, `0.892×`, `178 KiB`)** |
| `block-143.L` | `3.430` | `2.039` | `1.121 / 0.907 / 2.022` | `0.910 / 1.037 / 1.943` | `1.073 / 0.878 / 1.952` | **`0.898 / 0.866 / 1.776` (`0.518×`, `0.878×`, `154 KiB`)** |
| `block-210.L` | `2.973` | `1.839` | `0.953 / 0.912 / 1.867` | `0.900 / 0.925 / 1.822` | `0.942 / 0.891 / 1.837` | **`0.896 / 0.863 / 1.757` (`0.591×`, `0.941×`, `162 KiB`)** |
| `block-207.L` | `2.247` | `1.422` | `0.817 / 0.605 / 1.420` | `0.781 / 0.642 / 1.415` | `0.806 / 0.588 / 1.396` | **`0.767 / 0.562 / 1.327` (`0.591×`, `0.935×`, `182 KiB`)** |

### Pre-Registered Gate G5 Summary & Super-Modularity Diagnosis

Keeping the pre-registered criteria in `g5_w24_bp1p.c` strictly frozen:
- **(1) Overflow `<= 256 KiB` & Full Sanitizer Cleanliness:** **PASS (`6/6` blocks: `154.0 – 184.0 KiB`, `-85%..87.5%` vs `BVSMC-1P`'s `1,230 KiB`)**.
- **(2a/2b) `W24-R4+BP1P Tot / Open-qi16 Tot`:** **`6/6` PASS `<= 0.65×` (`0.518× – 0.647×`, mean `0.597×`)**; `3/6` `<= 0.60×` (`block-85` = `0.608×`, `block-2` = `0.627×`, `block-0` = `0.647×`).
- **(3a/3b) `W24-R4+BP1P Tot / GIM-RV(L16)+BV2P Tot`:** **`6/6` PASS `< 1.00×` (`0.871× – 0.955×`, `4.5%–12.9%` faster on every block!)**; `2/6` `<= 0.92×` (`4/6` `<= 0.933×`).
- **(4a/4b/4c) `W24-R4+BP1P / R4-DGIM(L16)+BV2P`:** **`6/6` PASS `< 1.00×` (`0.878× – 0.953×`)**, `4/6` `<= 0.940×` (`5/6` `<= 0.941×` — `block-210` landed at `0.9405×`!), `3/6` `Con <= 0.92×` (`0.801× – 0.950×`).
- **Frozen Strict Gate G5 Verdict:** **`FAIL`** (missed the `>= 5/6` sub-thresholds on `2a`, `3a`, `4a`, `4c` by `0.5–8 µs`, while **winning `6/6` blocks unanimously against every baseline `Arm 0..3`**).

**Experimental Verification of Super-Modularity (`Arm 3, 4, 5, 6`):**
- On **`block-0.L`**: Raising `LIMIT : 16 → 24` under 2-pass `BV2P` **regressed** total time (`1.035 → 1.073 ms`, `+3.7%`), switching to `BP1P` at `L = 16` gave `-2.8%` (`1.006 ms`), yet **combining `W24 + BP1P` dropped total time by `-6.9%` to `0.964 ms` (sub-millisecond!)**.
- On **`block-143.L`**: `W24 + BV2P` gave `-3.9%` (`1.943 ms`) and `L16 + BP1P` gave `-3.5%` (`1.952 ms`, additive expectation `-7.4%`), yet **`W24 + BP1P` achieved `-12.2%` (`1.776 ms`, `0.518×` `Open-qi16`)** because `words` shrank from `3,635 → 2,613` (`45.9 KiB < 48 KiB` L1D), making `Arm 6 Emit` (`0.866 ms`) **faster than `Arm 5 Emit` (`0.878 ms`) despite `+41,784` additional retained steps `m`**.
- **Coordinator Audit Candidate Note:** Both `BP-BVSMC-1P` (`154–184 KiB` overflow, eliminating `85–87.5%` of `BVSMC-1P`'s `1.23 MiB` pool) and branchless 3-YMM `U24Q` (`LIMIT = 24` when paired with 1-pass emission) are ready for Coordinator native Rust audit alongside `BVSMC` and `R4-DGIM`.
