# Wormhole Cycle 6 (Round 2, Cycle 1 of 5) — Six-Dimensional Lower & Upper Bounds in $\mathsf{BMCP}(M, B, \mu)$, and the Exact Gap Under Dieted `Open`

**Date:** 2026-11-21  
**Branch:** `research/carbon-wormhole`  
**Latest `origin/master` SHA:** `d425d772c0a0e6b82e9626b36bbfa4b28a6c9613` (`mibz2 0.5.9`)  
**Admitted-BWT Corpus (`origin/data/admitted-bwt`):** `41c03a5b22d630adb171231f5225da348b8055af` (`6/6` blocks SHA-256 authenticated)  
**Cycle:** 6 (Round 2, Cycle 1 of 5)

---

## 1. Explicit Machine Model $\mathsf{BMCP}(M, B, \mu)$ and Its Limits

After the memory diet (`Node16` + Chaitin `Endpoint`-in-`edges` overlay + `OutQ16` queues in dead `out[0..n)` + `[u32; 3]` metadata coloring) brings per-worker memory inside the i9-10900K's `20 MiB` shared L3, Mechanisms' Cycle 16 isolated the primary remaining cost: **`dep_past_l2_loads` (`~40.4%` of cycles)**, serially chained L2 misses hitting L3 one at a time.

To bound this quantity from below and above without Carbon Depth's hand-chosen nanosecond stage weights (`docs/journal/2026-11-20-carbon-depth-audit.md`), we state the **$(M, B, \mu)$-Bounded-MLP Cache-Probe Model ($\mathsf{BMCP}(M, B, \mu)$)** (combining Aggarwal–Vitter 1988 external memory $(M, B)$ with Miltersen–Nisan–Safra–Wigderson 1998 $\mu$-probe rounds):

- **Hierarchy:** Fast local memory holds $M$ bytes ($M_1 = 32\text{ KiB}$ L1d; $M_2 = 256\text{ KiB} = 4{,}096$ lines private L2 per i9-10900K core), backed by shared L3 of per-worker share $M_3(P) = \lfloor 20\text{ MiB} / P \rfloor$ (`2,048 KiB` at $P = 10$ workers; `2,925 KiB` at $P = 7$ workers), transferred in cache lines of $B = 64\text{ bytes}$.
- **Memory-Level Parallelism ($\mu$):** Computation proceeds in serial rounds $t = 1, \dots, D$. Between rounds, the core performs arbitrary computation on the $M$ bytes currently in fast memory. In round $t$, it may issue **at most $\mu \ge 1$ address-dependent cache-line loads** whose addresses are functions of the current fast-memory state ($\mu_{\mathrm{hw}} \approx 10\text{–}12$ L1d fill buffers / $16$ L2 MSHRs on Comet Lake).
- **Six Strictly Separated Cost Coordinates (never pooled or converted into one scalar):**
  1. **Preprocessing $(W_{\mathrm{prep}}, D_{\mathrm{prep}})$:** Operations and serial pointer-chase depth before byte emission.
  2. **Memory footprint $S_{\mathrm{mem}}$:** Per-worker scratch bytes (peak & persistent, tracked separately from `out[0..n)`).
  3. **Random-access count $Q_{\mathrm{rand}}$:** Total non-sequential address-dependent probes during emission.
  4. **Cache misses $(C_{\mathrm{miss}}(M_2), C_{\mathrm{miss}}(M_3))$:** Cache-line transfers past $M_2 = 256\text{ KiB}$ (L2$\to$L3) and past $M_3 = 2{,}048\text{ KiB}$ (L3$\to$DRAM).
  5. **Serial dependency depth $(D_{\mathrm{serial}}, D_{\mathrm{past-}M_2})$:** Longest chain of address-dependent loads ($D_{\mathrm{serial}}$), and maximum number of L2-missing loads along any single dependent chain ($D_{\mathrm{past-}M_2}$, the exact formal counterpart of `dep_past_l2_loads`).
  6. **Parallel work $W_{\mathrm{par}}$:** Total operations across all concurrent lanes/chains.
- **Explicit Model Limits:**
  - *Counts:* exact pointer-dependency DAG depth, exact Belady (`1966`) offline optimal and Mattson (`1970`) LRU cache-line transfers at $M_1, M_2, M_3$, exact irregular probe count, and exact byte capacity.
  - *Ignores:* branch mispredictions, store-buffer forwarding stalls, hardware next-line sequential stream prefetchers, TLB page walks (`4 KiB` vs `2 MiB` pages), L3 slice/ring contention, and register ALU latency between loads.

---

## 2. Three Lower-Bound Theorems as Functions of $(n, r, \mathbf{w})$ and Preprocessing

### Theorem 6.1 — Unrestricted L2-Miss Zero Escape vs Pătraşcu–Viola Succinct Probe Multiplication (`proved`; `verified` on `6/6` blocks in `verify_cycle6.rs`)

Let $I_0 = (\text{records}, C, p, n)$ have maximal run count $r$, alphabet size $\sigma$, and run-width profile $\mathbf{w} = (w_1, \dots, w_r)$.

1. **Zero Random L2-Miss Escape in Unrestricted Cell-Probe (`proved`):** The Elias-$\gamma$ / order-0 RLBWT encoding size of $(L, p)$ satisfies
   $$H_{\mathrm{RLBWT}}(n, r, \mathbf{w}) \;=\; \frac{1}{8}\!\left(r \log_2 \sigma + \sum_{j=1}^r \big(1 + 2\lfloor \log_2 w_j \rfloor\big)\right) \;\in\; [56.1,\; 133.0]\text{ KiB} \;<\; M_2 = 256\text{ KiB}$$
   across **all 6 authenticated blocks** (`verify_cycle6.rs`, `cycle6_results.json`). Therefore, in an *unrestricted* cell-probe model (unlimited internal ALU/probe steps per output byte), a decoder can stream `records` once into $M_2 = 256\text{ KiB}$, build an L2-resident compressed RLBWT, and decode $T$ with **zero non-compulsory L2$\to$L3 random misses** ($C_{\mathrm{miss}}^{\mathrm{LB,unrestricted}}(M_2) = 0$).
2. **Explicit Table Size vs Succinct Serial Probe Lower Bound (`proved` via Pătraşcu–Viola *SICOMP* 2010 & Golynski *SODA* 2009):** Any first-return representation $(S, \pi_{\mathrm{ret}}, W)$ on $m = |S|$ retained rows encodes a permutation $\pi_{\mathrm{ret}} \in S_m$ plus emitted phrases, requiring $\ge \log_2(m!) \ge m \log_2(m/e)$ bits.
   - Any **$1$-probe word-aligned representation** ($t_{\mathrm{probe}} = 1$ load per retained row, packing $(\text{next}_j, \text{code})$ into $\ge \lceil (\log_2 m + \log_2 |\mathcal{C}|)/8 \rceil = 4$ bytes) requires $S_{\mathrm{tab}}(m) \ge 4m$ bytes (`351.1–798.0 KiB > 256 KiB` whenever $m > 65{,}536$, holding on `6/6` blocks).
   - Conversely, compressing the successor map into $S_{\mathrm{mem}} \le M_2 = 256\text{ KiB}$ via rank/select dictionaries incurs redundancy $\Delta$ bits/block and by Pătraşcu–Viola (2010) forces $t_{\mathrm{probe}} = \Omega(\log(m/\Delta))$ **serially dependent L1/L2 probes per step** ($t_{\mathrm{probe}} \ge 3\text{–}4$, multiplying $D_{\mathrm{serial}}$ from $m$ to $\ge 3m\text{–}4m$, matching `FRONTIER.md`'s measured `1.71–2.80×` loss on `BSO-512`).

---

### Theorem 6.2 — 4-Hamming Text-Splice Adversary & $\mu$-MLP Serial Dependency Lower Bound (`proved`; constructively `verified` across all $7! = 5{,}040$ permutations in `verify_cycle6.rs`)

Let $\mathbf{w} = (w_1, \dots, w_r)$ have $r_1 = |\{j : w_j = 1\}|$ singleton runs and $m_\Lambda = 1 + \sum_{w_j \le \Lambda} w_j$.

1. **4-Hamming Splice Indistinguishability Lemma (`proved` + `verified` on `5,040` permutations):** For any $K \ge 3$, let $u = 0^k 1$ ($k = \lceil 2\log_2 n \rceil$) be a marker absent from distinct blocks $X_1, \dots, X_{K-1}$ (where $X_q$ starts with byte $A_q$ and ends with byte $a_q$, all distinct), and define the $(K-1)!$ single-cycle texts
   $$T_\sigma \;=\; u\,X_{\sigma(1)}\,u\,X_{\sigma(2)}\,\cdots\,u\,X_{\sigma(K-1)}\,u\,Z\$$ \qquad (\sigma \in S_{K-1}).$$
   Then:
   - Every $T_\sigma$ has **identical length $n$, identical run count $r$, and identical sorted run-width multiset $\mathbf{w}$** (`verified` across all $7! = 5{,}040$ permutations in `verify_cycle6.rs`).
   - Swapping any two non-adjacent blocks $\sigma \mapsto \sigma \cdot (q_1, q_2)$ changes **exact Hamming distance $4$** out of $n$ characters in $L(T_\sigma)$ (`verified` on `5,040 / 5,040` swaps), while shifting the exact output position $\tau(x)$ of every row in the intervening blocks by $|X_{q_2}| - |X_{q_1}|$.
   - Every interior row of $X_q$ at distance $> k$ from $u$ has **identical $d$-run local $L$-context** across all $(K-1)!$ permutations (giving a constructive proof of `#687` Local-Factor Census's failure: local run shape in $L$ carries $0$ bits of $\sigma$).
2. **Serial Dependency Depth Lower Bound vs Preprocessing & $\mu$ (`proved` via Miltersen–Nisan–Safra–Wigderson 1998 & Verbin–Yu *SODA* 2011):**
   - **Under non-jumping preprocessing** (any preprocessing that eliminates at most $H(S)$ monochromatic intervals without composing $\pi_{\mathrm{ret}} \circ \pi_{\mathrm{ret}}$ across retained rows — including `Open`, `SwarOutQ16`, and `CPC-Micro`): determining the global block order of $\mathcal{F}_K$ (or computing $\tau(s_1)$ for any non-marked cut $s_1$ at cycle distance $d \asymp K$) requires chasing $K$ links of $\sigma$ from $p$. With at most $\mu$ concurrent probes per serial round:
     - At threshold $\Lambda = 16$ ($|S| = m_{16}$):
       $$Q_{\mathrm{rand}} \;\ge\; m_{16} \quad (\text{worst-case cycle}), \qquad D_{\mathrm{serial}}(\mu) \;\ge\; \left\lceil \frac{m_{16}}{\mu} \right\rceil.$$
     - Across *all* retained-set thresholds $\Lambda \ge 1$ with RV elimination budget $H(S) \le r - r_1$ (by Section–Complement Duality $|S| + H(S) \ge r$ and singleton irreducibility):
       $$Q_{\mathrm{rand}} + D_{\mathrm{prep}} \;\ge\; r, \qquad D_{\mathrm{serial}}(\mu) \;\ge\; \left\lceil \frac{r_1}{\mu} \right\rceil \quad \big(r_1 \in [34{,}262,\; 97{,}054]\text{ on the 6 blocks}\big).$$
   - **Under $J \ge 1$ pointer-jumping / doubling passes in preprocessing** (`Wyllie 1979`): halving cycle depth by $2^J$ requires $W_{\mathrm{prep}} \ge J \cdot m$ random probes (`J >= 1` already pays $\ge m$ extra random accesses before `emit` starts).

---

### Theorem 6.3 — Belady Offline Optimal ($C_{\mathrm{OPT}}$) & $\mu$-MLP Serial L2-Miss Floor on Explicit Return Tables (`proved` via Belady *IBM Syst. J.* 1966; `verified` on `6/6` blocks)

For any decoder traversing an explicit 4-byte-per-entry return table `edges[0..m)` with cache capacity $M_2 = 256\text{ KiB}$ ($L_2 = 4{,}096$ lines of $64\text{ B}$, $16$ entries/line):

1. **Compulsory Line Floor (`proved`):** Visiting all $m$ vertices touches $N_{\mathrm{lines}} = \lceil m / 16 \rceil \in [5{,}618,\; 12{,}768]$ distinct 64B cache lines.
2. **Belady Offline Replacement Floor (`proved`):** Let $C_{\mathrm{OPT}}(M_2)$ be Belady's farthest-in-future offline minimum miss count on the cycle's line sequence $(\lfloor j_t / 16 \rfloor)_{t=0}^{m-1}$. No hardware or software replacement policy on `edges[0..m)` (without changing vertex layout) can incur fewer than $C_{\mathrm{OPT}}(M_2)$ L2 misses, and no $\mu$-MLP schedule traversing those edges can achieve serial L2-miss depth below:
   $$D_{\mathrm{past-}M_2}^{\mathrm{LB}}(\mu) \;=\; \left\lceil \frac{C_{\mathrm{OPT}}(M_2)}{\mu} \right\rceil.$$

---

## 3. Exact Six-Dimensional Ledger: LOWER vs Dieted `Open` UPPER (`verify_cycle6.rs`)

Verified byte-exact on **`16,209` small/multi-cycle/multi-epoch cases + `5,040` splice permutations + `6/6` authenticated blocks** against compiled `origin/master` (`d425d77`) [`mibz2::bwt::OpenCells`](../../../src/open.rs):

| Block | $r$ | $r_1$ (Univ. $Q$ LB) | $m_{16}$ (`Diet` $Q_{\mathrm{rand}}$ & $D_{\mathrm{serial}}$) | Prep $D_{\mathrm{prep}}$ (`Unit` / `Zorich`) | `Diet` Scratch (`edges` KiB) | Compulsory Lines $\lceil m/16\rceil$ | **Belady $C_{\mathrm{OPT}}(256\text{K})$** ($\mu=1$ LB) | **`Diet` $D_{\mathrm{past-}M_2}$** (`LRU` $\mu=1$ Upper) | `Diet` L3 Misses (`2048K`, $W=10$) | **$\mathsf{BMCP}$ LB ($\mu=4$)** | **$\mathsf{BMCP}$ LB ($\mu=8$)** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `51,184` | `35,116` | `89,874` | `11,215` / `7,250` | `351.1K` | `5,618` | **`7,437`** | **`25,533`** (`0.028/B`) | **`809`** | **`1,860`** | **`930`** |
| `block-2.L` | `54,532` | `34,262` | `112,070` | `13,671` / `8,573` | `437.8K` | `7,005` | **`11,947`** | **`37,778`** (`0.042/B`) | **`746`** | **`2,987`** | **`1,494`** |
| `block-85.L` | `79,375` | `55,504` | `156,404` | `7,130` / `3,563` | `611.0K` | `9,776` | **`19,025`** | **`47,553`** (`0.053/B`) | **`479`** | **`4,757`** | **`2,379`** |
| `block-143.L` | `108,335` | `72,578` | `196,841` | `21,295` / `11,807` | `768.9K` | `12,303` | **`41,149`** | **`94,739`** (`0.105/B`) | **`831`** | **`10,288`** | **`5,144`** |
| `block-210.L` | `128,732` | `97,054` | `204,281` | `12,140` / `6,237` | `798.0K` | `12,768` | **`38,714`** | **`89,106`** (`0.099/B`) | **`563`** | **`9,679`** | **`4,840`** |
| `block-207.L` | `87,111` | `62,407` | `135,921` | `13,898` / `8,306` | `530.9K` | `8,496` | **`19,279`** | **`51,240`** (`0.057/B`) | **`815`** | **`4,820`** | **`2,410`** |

### Lever 2 (`2026-11-21-the-shape-of-the-rh-proof.md`): Phase Transition Across $n \in [100\text{k}, 900\text{k}]$ (`measured`, `verify_cycle6.rs`)

| Prefix Size $n$ | `block-0` $4m$ (`KiB`) | `block-0` $D_{\mathrm{past-}M_2}$ (`miss/B`) | `block-143` $4m$ (`KiB`) | `block-143` $D_{\mathrm{past-}M_2}$ (`miss/B`) |
|---:|---:|---:|---:|---:|
| `100,000` | `47.3 KiB` | **`0`** (`0.0000`) | `124.8 KiB` | **`0`** (`0.0000`) |
| `200,000` | `110.0 KiB` | **`65`** (`0.0003`) | `132.6 KiB` | **`118`** (`0.0006`) |
| `300,000` | `134.8 KiB` | **`303`** (`0.0010`) | `242.3 KiB` | **`8,076`** (`0.0269`) |
| `450,000` | `208.7 KiB` | **`5,876`** (`0.0131`) | `384.8 KiB` | **`32,035`** (`0.0712`) |
| `650,000` | `313.8 KiB` | **`19,575`** (`0.0301`) | `573.3 KiB` | **`62,959`** (`0.0969`) |
| `~900,000` | `351.1 KiB` | **`25,533`** (`0.0284`) | `768.9 KiB` | **`94,739`** (`0.1053`) |

`dep_past_l2_loads` exhibits a **critical capacity phase transition** right where $4m(n) + S_{\mathrm{dict}}$ crosses $M_2 = 256\text{ KiB}$ ($n \approx 250\text{k–}400\text{k}$), jumping from `0` misses to `0.028–0.105` dependent misses/byte (`345,949` total across the 6 blocks).

---

## 4. The Exact Gap, and What Cycle 7 Must Test

Comparing LOWER and UPPER across all six separated dimensions:

1. **Where the bounds ALREADY MEET:**
   - **L3 / DRAM misses ($M_3 = 2{,}048\text{ KiB}$ at $W = 10$ workers):** Dieted `Open` incurs only **`479–831` L3 misses per block** (`< 0.06 B/B` during `emit`). The L3/DRAM problem is solved; no further memory diet is needed.
   - **Single-cursor random-access count $Q_{\mathrm{rand}}$ at $\Lambda = 16$:** Meets $m_{16}$ (`1.00×`).
2. **Where the bounds DO NOT MEET (The Gap):**
   - **LRU vs Offline Belady (`2.30×–3.43×`):** Even at $\mu = 1$, LRU (`25.5k–94.7k`) pays `2.3–3.4×` Belady `OPT_L2` (`7.4k–41.1k`) because 16 entries per 64B line are visited at pseudorandom cycle times (`> 4,096` steps apart), causing repeated eviction and reload of the same cache line.
   - **$\mu = 1$ Serial Walk vs $\mu$-MLP Hardware (`4×–8×`):** Dieted `Open` runs at $\mu = 1$, chaining `100%` of its `25.5k–94.7k` `edges` L2 misses serially (`9–15` of the core's MSHRs idle!), sitting **`4×–8×` above `DietL2 / \mu`** and **`9.2×–27.5×` above `OPT_L2 / \mu`**.
3. **Why Crossing the Gap Requires Solving a Three-Bound Squeeze:**
   - At $P = 10$ workers on `20 MiB` L3 (`2,048 KiB`/worker), `out` (`879 KiB`) + `edges` (`351–798 KiB`) + `dict` (`~100 KiB`) leaves **only `~270 KiB` of L3 headroom per worker** — ruling out `#675`'s `900 KiB` staging buffers!
   - By Theorem 6.2 (`4-Hamming Splice`), no local rule knows the output offsets $\tau(s_q)$ of $K$ parallel starts before walking!
   - **Cycle 7's Mandate (Move D of `2026-11-21-the-shape-of-the-rh-proof.md`):** Can an **alternative global-order construction** — specifically r-index **$\Phi$ / toehold sampling**, **bijective BWT / Lyndon factorization**, or **partial Rauzy–Veech / Zorich boundary evaluation** — build $K \ge 4$ exact anchor pairs $(s_q, \tau(s_q))$ in $O(r)$ without paying more serial L2 misses in preprocessing than `emit` saves?

---

## 5. Artefacts & Literature Cited

- `verify_cycle6.rs` (`#[forbid(unsafe_code)]` core, linked directly against `origin/master` `libmibz2.rlib`).
- `dieted_open_upper.toml` (`PASS` under `docs/team/acquisition/check_manifest.py`).
- `cycle6_results.json`, `cycle6_output.txt`.
- **Citations:** L. A. Belady (*IBM Syst. J.* **5**(2), 1966); R. L. Mattson, J. Gecsei, D. R. Slutz & I. L. Traiger (*IBM Syst. J.* **9**(2), 1970); A. Aggarwal & J. S. Vitter (*CACM* **31**(9), 1988); P. B. Miltersen, N. Nisan, S. Safra & A. Wigderson (*JCSS* **57**(1), 1998); A. Golynski (*SODA* 2009); M. Pătraşcu & E. Viola (*SICOMP* **39**(6), 2010); E. Verbin & W. Yu (*SODA* 2011).
