# Carbon Cycle 14 — Algebraic Discrete Morse Reduction in the Fox Free-Monoid Ring $\mathbb{Z}\langle\Sigma^*\rangle$, `L1Cap-Fox64` Variable-Slot Dictionary, and `ValRet-YMM-BP4K` (`6/6 <= 0.546× Open-qi16`, `C14-LowMem` Beats `C13-Speed`)

*20 November 2026 — Carbon (`research/carbon-word-transport`)*

## 1. Executive Summary & Cycle 14 Scorecard

At the Cycle 14 boundary, `origin/master` (`804736e` / `ad65b3b`) introduced a major mathematical lead: **Forman / Sköldberg / Mischaikow–Nanda Discrete Morse reduction** (prompted by Vidit Nanda's 2026 Perseus 6 techniques over $\mathbb{Z}$ and $\mathbb{Z}[t, t^{-1}]$, Fox free-monoid rings $\mathbb{Z}\langle\Sigma^*\rangle$, Mrozek–Batko reductions vs coreductions, apparent pairs, totally unimodular $\det = \pm 1$ graph-incidence pivots, 256-entry 1-byte classification, 2-level prefetch, and index-order pointer-chain contraction), alongside Cycle 13's sole missed sub-bar (`G5.3a`: `<= 0.560× Open-qi16` on `6/6` Wikidata, where `block-0` and `block-2` stopped at `0.578×` and `0.572×`).

Cycle 14 investigates both the Discrete Morse / Fox-ring foundation and the exact microarchitectural bottlenecks of `BP4K` + `BipolarZQ`, yielding three model-free theorems (`verify_cycle14.py`, `6.40s`), two validated acquisition manifests (`fox64_u8fq_bp4k.toml`, `fox64_bzq_bp4k.toml`), and a frozen 25-round 6-arm confrontation (`g5_fox64_c14.c`):

- **Theorem 14.1 (`proved` + `verified` + `measured` — Algebraic Discrete Morse / Fox-Ring Equivalence, Unimodular Orbit-Time $\det = +1$, Mrozek–Batko Duality, & $1/3$ Combinatorial Peak Law):**
  1. **Sköldberg–Fox Equivalence & Unimodular Orbit-Time (`proved`, `verified` `1,620` cases):** On the cut BWT functional graph $G_{\text{cut}} = (V, E \setminus \{e_{\text{mark}}\})$, any cycle-free matched subset $S \subseteq V \setminus \{p\}$ defines an acyclic Forman matching whose Sköldberg reduced differential over the non-commutative Fox free-monoid ring $\mathbb{Z}\langle\Sigma^*\rangle$,
     $$M_{\text{crit}} = M_{CC} + M_{CS}\Big(\sum_{k \ge 0} M_{SS}^k\Big) M_{SC},$$
     is **algebraically identical** to Meyer/Open/R4 elimination (`1` outgoing gradient-path word $w(c) \in \Sigma^+$ per non-terminal critical cell $c \in C$). Specializing $\Sigma \mapsto t$ recovers Perseus 6's $\mathbb{Z}[t]$ reduction $t^{|w(c)|}$, while specializing $t \mapsto 1$ over $\mathbb{Z}$ yields an exact **totally unimodular reduced Laplacian** $\det(I - P_{\text{crit}}) = +1$ whose unique integer solution $(I - P_{\text{crit}})\boldsymbol{\tau}_C = \boldsymbol{\ell}_C$ computes the global output coordinate $\tau_C(c) = \mathrm{dist}_{\mathrm{LF}}(c, p) + \ell_C(p)$ (`verified` via exact Bareiss determinants on `1,120/1,120` Lyndon BWTs + `500/500` multi-alphabet BWTs). However, because $G_{\text{cut}}$ is a single directed path, forward/backward substitution *is* the $m$-step critical orbit walk (scoping Kitchen Sink's unimodular lead).
  2. **Mrozek–Batko Duality & Apparent Pairs (`proved`, `verified` `500` + `7` blocks):** Reduction matches thick $L$-strips $S_L$, whereas Coreduction matches thick $F$-strips $S_F = \mathrm{LF}(S_L)$ (`Backward Open` / Cycle 8 `CFSE`), preserving $|V \setminus S_L| = |V \setminus S_F| = m$. Right-endpoint Perseus apparent pairs (`bs == sb` in `R4-DGIM`) occur **`0` times** across all `6/6` Wikidata blocks and `runs`.
  3. **Zero-Bitset $1/3$ Peak Matching & L1D Dictionary Obstruction (`proved`, `verified` `5,912` permutations, `measured` census):** Strict index peaks $I_{\text{peak}} = \{v \ne \text{mark} : \pi^{-1}(v) < v > \pi(v)\}$ form a bitset-free independent Morse matching ($A_{\text{peak}} \cap I_{\text{peak}} = \emptyset$) with exact rational expectation $\mathbb{E}[|I_{\text{peak}}|]/m = \frac{1}{3} - O(1/m)$ (`29.5–30.0%` measured on `6/6` Wikidata) and admit single-pass index-order contraction. **Obstruction:** Contracting `m/3` thin peaks creates `35k–68k` distinct multi-byte Fox words (`560 KiB – 1.09 MiB >> 48 KiB` L1D), spilling `inline16[]` to L2, while an L1D-capped `512`-entry bigram cache captures only `2.8–13.3%` of $m$.

- **Theorem 14.2 (`proved` + `verified` `1,506` cases — Slack-Budgeted Variable-Slot `L1Cap-Fox64` Dictionary):**
  - Word-length census at `LIMIT = 24` revealed why `16 B` inline slots left speed on the table: **`3.5–6.7%` of emitted steps (`4.5k–13.7k` steps/block) exceed `16 B`** (`1.2–2.8%` in `17..32 B`, `2.3–4.9%` in `33..64 B`, and `< 0.5%` in `> 64 B`).
  - Packing `|w| <= 16` into `1` slot (`16 B`), `17 <= |w| <= 32` into `2` contiguous slots (`32 B`), and `33 <= |w| <= 64` into `4` contiguous slots (`64 B`) inside `inline16[0..3072)` under the exact slack invariant
    $$\text{extra\_budget} = \max(0,\; 3072 - 256 - W_{\text{raw}})$$
    guarantees `(256 + slots) * 16 B <= 49,152 B = 48.0 KiB` L1D (`48.0 KiB` exact on `block-143`, `25.6–42.2 KiB` on the other 5 blocks) while keeping `slots < 2816 << (2^{14} - 256)` so **`wide == 0` (`0 B` `extra[]`) holds on `10/10` `r/n < 0.20` blocks**.

- **Theorem 14.3 (`proved` + `verified` `1,051,505` cases + `measured` — Power-of-Two Dummy Self-Loop, Disassembly Stall Elimination, `ValRet-YMM-BP4K`, & `DirectPtr-BZQ`):**
  - Disassembling Cycle 13's `bp4k_emit_pf8l_c13` (`objdump -d`) exposed three compounding microarchitectural stalls:
    1. **Address-escape lane spills:** Passing `&j_l`, `&p_l`, `&end_l`, `&rem_l` to `noinline` cold calls forced GCC to spill/reload lane state onto the stack every round;
    2. **Stack partial store-forwarding stalls (`40` stack stores/round):** `_mm_storeu_si128((__m128i*)(p_l - 16), v)` compiled via a 16-byte stack temporary `vmovdqa [rbp-0x46a0], xmm` whose stack slot GCC reused for spilled 8-byte `mov [rbp-0x46a0], rdi` — triggering a 16B/8B partial store-forwarding penalty;
    3. **Variable `shr cl` shift & `bits == 17` vs `18`:** `block-0` (`m = 122,028 < 2^{17}`) and `block-2` (`m = 126,615 < 2^{17}`) have `bits = 17` while `block-85..210` and `runs` have `bits = 18`; setting `bits = 32 - clz(m)` (instead of `clz(m - 1)`) guarantees $m < 2^{\text{bits}}$ so dummy index `m` is an exact self-loop `m & mask == m` even at exact powers of two $m = 2^k$ (`verified` for `1,049,999` values of $m$).
  - Replacing XMM inline stores with unconditional `32 B` `memcpy` (`vmovdqu ymm` direct to `[p_l - 32]`, covering **`95.1–100%` of steps in a single fast path**), inline `64 B` stores (`33 <= |w| <= 64`, `BP4K_SAFE64 = 4032`), `bits == 17 / 18` compile-time shift specialization, 2-register System V ABI `FoxColdRet` (`rax:rdx`, zero `&` escapes), and `DirectPtr-BZQ` (`cur_ptr[256] / cur_lim[256]`):
    - **Conquers Cycle 13's unfinished `G5.3a` (`<= 0.560× Open-qi16` on `6/6` Wikidata: `PASS 6/6` at `0.428× – 0.546×`, `0.827–1.474 ms`)!**
    - **`C14-LowMem` (`4.2 MiB` worker scratch, `0 B` queue, `0 B` `extra`, zero `Runs.per`) beats Cycle 13's `RunsMeta` *Speed* kernel on `5/6` Wikidata blocks (`G5.4b PASS`)** and reaches **`0.458× – 0.569× Open-qi16` (`G5.4c PASS 6/6`)**!
    - **Strict Frozen Gate G5 Verdict (`g5_fox64_c14.c`):** **Overall strict verdict `FAIL` (`11/13` sub-bars `PASS`)** solely on tightened `G5.3b` (`<= 0.540× Open`: `5/6`, `block-0` `0.546×`) and `G5.3c` (`<= 0.930× C13-Speed`: `4/7` vs `>= 6/7`, `7/7 < 1.00×`). Crucially, `probe_d32_opt.c` already isolated the exact `0.05–0.21 ms` root cause for Cycle 15 (§4.2): outlining `if (k->wide)` from `bp4k_emit_fox64` and bounding $W_{\text{raw}}$ in $O(1)$ without walking `tops[sa]` brings `block-0` to **`0.768 ms` (`0.522×`)** and `block-143` to **`1.262 ms` (`0.380× Open`, `0.835× C13`)**.

---

## 2. Explore & Triage Kills (Recorded Honest Negatives)

| Investigated Candidate | Mechanism | Measured Outcome (`explore_cycle14.c`, `triage_c14.c`, `isolate_c14.c`, `probe_d32_opt.c`) | Why Killed |
|---|---|---|---|
| `OneBr16` / `OneBr32` (`ConjBP-Seg`) | Fuse `q >= SEG_START` + `row == p` into single `< sentinel` branch across 16 or 32 lanes | `OneBr16`: `2.078 -> 2.345 ms` (`+12.8%`); `OneBr32`: `2.078 -> 2.203 ms` (`+6.0%` on `random`) | `16` lanes starve L3 MLP (`900 KB` `3.6 MB` random loads need `>= 32` concurrent misses); buffering `q[32]` delays next-round prefetches |
| `SelfLoop-ConjBP32` (`marked |= (q>>31)<<K`) | Replace 32 per-lane `if (q & SEG_START)` branches with branchless mask accumulation + `tzcnt` | `2.008 -> 2.049 ms` (`+2.0%` on `random`), `1.547 -> 1.564 ms` (`text0`) | `q & SEG_START` is not-taken `99.95%` (`1 / 2000`), so branch prediction is zero-cycle while `marked |= (q>>31)<<K` adds 32 serial shift-OR ops per round |
| `LeafBurst-BP4K` (`8`-step single-lane burst on `<=16B` runs) | Advance 1 lane by 8 steps back-to-back before switching lanes (`BP4K_BURST_SAFE = 3936`) | `0.881 -> 1.076 ms` (`block-0`), `1.561 -> 1.908 ms` (`block-143`, `+22%` slower) | Round-abort amplification $1 - (1 - p_{>16})^8 \approx 28\%$ triggers slow re-loads across all 8 lanes while serializing 8 dependent L2 `edges[]` hops |
| `ThinPeak-Morse` (`m/3` bigram contraction) | Contract strict index peaks $I_{\text{peak}}$ (`29.8%` of $m$) in index order (Perseus technique) | `35k–68k` distinct words (`560 KiB – 1.09 MiB >> 48 KiB` L1D); `512`-slot L1D bigram cap hits only `2.8–13.3%` of $m$ | Either overflows `48 KiB` L1D dictionary by `12–23×` (turning every inline word fetch into an L2 miss) or saves `< 10%` of steps at the cost of a random `1.0m` producer pass |
| `1-Pass pack_edges_avx2` & `U16Gate` | Single-pass `inline16 + edges` write in Phase 3; `width <= 16` 32B stores in `RunQ` | `1-Pass`: `3–10%` slower (`1.262 -> 1.387 ms` on `block-143`); `U16Gate`: `2–3%` slower on `block-207/210/runs` | `2-Pass` lets GCC auto-vectorize `edst[s] = pack | qsrc[s]` with AVX-512 `vpord zmm` without `inline16` RFO contention; `U16Gate` crosses 64B cache lines more often on singletons |

---

## 3. Theorems & Proofs

### Theorem 14.1 (Algebraic Discrete Morse / Fox-Ring Equivalence, Unimodular Orbit-Time, & $1/3$ Peak Law — `proved`, `verified` `8,032` cases)
1. **Fox-Ring Schur–Morse Identity (`proved`):** Give $G_{\text{cut}} = (V, E \setminus \{e_{\text{mark}}\})$ edge weights $M_{u, \mathrm{LF}(u)} = F[u] \in \mathbb{Z}\langle\Sigma^*\rangle$. For any matched subset $S \subseteq V \setminus \{p\}$ inducing no directed cycle, $M_{SS}^k = 0$ for all $k > |S|$, so $(I - M_{SS})^{-1} = \sum_{k=0}^{|S|} M_{SS}^k$ exists over $\mathbb{Z}\langle\Sigma^*\rangle$ and sums the exact non-commutative word labels of unique Forman gradient paths $c \to s_1 \to \dots \to s_k \to c'$, matching `Open`/`R4-DGIM` identically.
2. **Unimodular Orbit-Time $\det(I - P_{\text{crit}}) = +1$ (`proved`):** Under $\Sigma \mapsto 1 \in \mathbb{Z}$, $P_{\text{crit}}$ is nilpotent ($P_{\text{crit}}^{|C|} = 0$) because $G_{\text{cut}}$ is acyclic; ordering $C$ in topological orbit order makes $I - P_{\text{crit}}$ unitriangular with $\det(I - P_{\text{crit}}) = +1$, and $(I - P_{\text{crit}})\boldsymbol{\tau}_C = \boldsymbol{\ell}_C$ solves $\tau_C(c) = \mathrm{dist}_{\mathrm{LF}}(c, p) + \ell_C(p)$ over $\mathbb{Z}$.
3. **Combinatorial Peak Matching (`proved`):** On any single-cycle permutation $\pi$ of $\{0, \dots, m-1\}$ cut at `mark`, $I_{\text{peak}} = \{v \ne \text{mark} : \pi^{-1}(v) < v > \pi(v)\}$ has no adjacent vertices because $u \in I_{\text{peak}} \implies u > \pi(u)$, violating $\pi^{-1}(w) < w$ at $w = \pi(u)$. For uniform cyclic permutations ($m \ge 4$), every non-adjacent triple $(\pi^{-1}(v), v, \pi(v))$ on $V \setminus \{\text{mark}\}$ has probability $1/3$ of peaking at $v$, giving exact mean $\mathbb{E}[|I_{\text{peak}}|] = (m - 1)/3 - O(1)$ (`verified` `Fraction(1, 3)` on $m = 4\dots 8$, `5,912` cycles).

### Theorem 14.2 (Slack-Budgeted Variable-Slot `L1Cap-Fox64` Dictionary — `proved`, `verified` `1,506` cases)
Let $W_{\text{raw}}$ be an upper bound on raw Phase-3 band words (`words <= 2H + 255` by Theorem 13.2), and set $\text{extra\_budget} = \max(0,\, 3072 - 256 - W_{\text{raw}})$. Allocating `1` slot for $|w| \le 16$ (`0` extra), `2` slots for $17 \le |w| \le 32$ (`1` extra if `extra_used + 1 <= extra_budget` and `slots <= 2948`), and `4` slots for $33 \le |w| \le 64$ (`3` extra if `extra_used + 3 <= extra_budget` and `slots <= 2800`), falling back to `1` sentinel slot (`w_len = 0xFF`) otherwise, consumes at most $256 + W_{\text{raw}} + \text{extra\_budget} \le 3072$ slots $= 49,152\text{ B} = 48.0\text{ KiB}$ L1D and satisfies $256 + \text{slots} \le 3072 < 16,384$, preserving `wide == 0` whenever $m < 2^{18}$.

### Theorem 14.3 (`bits = 32 - clz(m)` Dummy Self-Loop & `BP4K_SAFE64` — `proved`, `verified` `1,051,505` cases)
1. **Exact Power-of-Two Self-Loop (`proved`):** For every integer $m \ge 1$, $b = 32 - \mathrm{clz}_{32}(m) = \lfloor\log_2 m\rfloor + 1$ satisfies $m < 2^b$, so dummy slot `edges[m] = m` (`w_len = 0`) unpacks `next_j = edges[m] & ((1u << b) - 1) == m` with `0` bytes written — remaining a fixed point even when $m = 2^k$ (where `32 - clz(m - 1) = k` would alias `m & (2^k - 1) == 0`!).
2. **64B-Safe Bipolar Paging (`proved`):** Setting `BP4K_SAFE64 = 4096 - 64 = 4032` (`end_l = page_base + 64`) ensures both unconditional `32 B` stores (`[p_l - 32 .. p_l)`) and inline `64 B` stores (`[p_l - 64 .. p_l)`) stay strictly within `page_base[0..4096)` whenever `p_l >= end_l`, with universal Pigeonhole overflow $\le (S + 2) \cdot 4\text{ KiB} = 200\text{ KiB} \le 256\text{ KiB}$ ($S = 48$).

---

## 4. Controlled 25-Round 6-Arm Confrontation (`g5_fox64_c14.c`, `-O3 -march=native`)

*Note (`METHOD.md` §5): Single-thread Sapphire Rapids VM C timings (`Xeon Platinum 8481C`, `48 KiB` L1d, `2 MiB` L2); native 1–20 thread Rust verification belongs to the Coordinator.*

### 4.1 Frozen Gate G5 Confrontation Table (`task-1286`, `25` Interleaved Rounds)

| Block | `nr/n` | `Open16/Col` (ms) | `SegShp` (ms) | `C13-LowM` (ms) | `C13-Spd` (ms) | **`C14-LowMem`** (ms) | **`C14-Speed`** (ms) | **`14Spd / Open`** | **`14Low / Open`** | **`14Spd / C13Spd`** | `DictKB` | `WS_KB` | `OvfKB` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `0.0627` | `1.514` | `2.379` | `0.945` | `0.872` | **`0.861`** | **`0.827`** | **`0.546x`** | **`0.569x`** | **`0.948x`** | `31.5 K` | `1,221 K` | `188 K` |
| `block-2.L` | `0.0661` | `1.903` | `2.452` | `1.139` | `1.068` | **`1.022`** | **`0.979`** | **`0.514x`** | **`0.537x`** | **`0.917x`** | `37.2 K` | `1,243 K` | `196 K` |
| `block-85.L` | `0.0941` | `1.950` | `2.258` | `1.091` | `1.004` | **`0.971`** | **`0.918`** | **`0.471x`** | **`0.498x`** | **`0.914x`** | `25.6 K` | `1,580 K` | `200 K` |
| `block-143.L` | `0.1248` | `3.444` | `2.390` | `1.653` | `1.548` | **`1.579`** | **`1.474`** | **`0.428x`** | **`0.458x`** | **`0.952x`** | **`48.0 K`** | `2,156 K` | `192 K` |
| `block-210.L` | `0.1480` | `2.971` | `2.311` | `1.601` | `1.453` | **`1.403`** | **`1.337`** | **`0.450x`** | **`0.472x`** | **`0.920x`** | `39.1 K` | `2,008 K` | `192 K` |
| `block-207.L` | `0.1016` | `2.268` | `2.314` | `1.263` | `1.164` | **`1.126`** | **`1.073`** | **`0.473x`** | **`0.496x`** | **`0.922x`** | `42.2 K` | `1,415 K` | `204 K` |
| `onebyte` | `0.0079` | `0.026` | `0.261` | `0.028` | `0.026` | **`0.028`** | **`0.028`** | `1.087x` (`0.107x` Shp) | `1.072x` (`0.106x` Shp) | `1.071x` | `4.0 K` | `4 K` | `0 K` |
| `pattern` | `0.0079` | `1.896` | `1.200` | `0.104` | `0.084` | **`0.105`** | **`0.085`** | **`0.045x`** (`0.071x` Shp) | **`0.056x`** (`0.088x` Shp) | `1.010x` | `4.0 K` | `4 K` | `16 K` |
| `rle_spiral` | `0.0204` | `2.487` | `1.223` | `0.209` | `0.179` | **`0.202`** | **`0.183`** | **`0.074x`** | **`0.081x`** | `1.023x` | `4.8 K` | `1,144 K` | `152 K` |
| `runs` | `0.1825` | `4.063` | `2.024` | `1.297` | `1.149` | **`1.208`** | **`1.149`** | **`0.283x`** (`0.567x` Shp) | **`0.297x`** (`0.597x` Shp) | **`1.000x`** | `10.0 K` | `2,770 K` | `216 K` |
| `text0` | `0.2982` | `10.426` | `2.186` | `2.118` | `2.111` | **`2.121`** | **`2.116`** | **`0.968x` Shp** | **`0.970x` Shp** | `1.002x` | `0.0 K` | `3,515 K` | `205 K` |
| `random` | `0.9922` | `3.728` | `3.744` | `2.622` | `2.584` | **`2.571`** | **`2.575`** | **`0.688x` Shp** | **`0.687x` Shp** | **`0.997x`** | `0.0 K` | `3,515 K` | `207 K` |
| `records` | `0.9969` | `3.783` | `3.763` | `2.623` | `2.615` | **`2.595`** | **`2.607`** | **`0.693x` Shp** | **`0.690x` Shp** | **`0.997x`** | `0.0 K` | `3,515 K` | `209 K` |

- **Passed Sub-Bars (`11/13`):** `G5.1` (12/12 adversarial + 13/13 blocks + canaries + `out == ll`), `G5.2a` (`10/10` zero BZQ overflow chunks), `G5.2b` (`10/10 <= 48.0 KiB` L1D dict & `wide == 0`), `G5.2c` (`6/6` WS `<= 2,250 KB` & `13/13` Ovf `<= 256 KB`), **`G5.3a` (`6/6 <= 0.560× Open-qi16`)**, **`G5.4a` (`6/7` `C14-LowMem <= 0.940× C13-LowMem`)**, **`G5.4b` (`5/6` `C14-LowMem <= 1.000× C13-Speed`)**, **`G5.4c` (`6/6` `C14-LowMem <= 0.575× Open-qi16`)**, `G5.5a` (`3/3` dense/col win), `G5.5b` (`13/13 < 1.00×`, `12/13 <= 0.80×` Shipped), `G5.5c` (`13/13 < 1.00×`, `12/13 <= 0.80×` Shipped).
- **Missed Tightened Sub-Bars (`2/13` -> Honest Overall Gate G5 `FAIL`):** `G5.3b` (`<= 0.540× Open`: `5/6`, `block-0` at `0.546×`) and `G5.3c` (`<= 0.930× C13-Speed`: `4/7` vs `>= 6/7`, `7/7 <= 1.000×`).

### 4.2 Exact Root-Cause Diagnosis for Cycle 15 (`probe_d32_opt.c` vs `g5_fox64_c14.c`)

Comparing `g5_fox64_c14.c` against `probe_d32_opt.c` (`task-1209`, where `Fox64` ran **`0.768 ms` (`0.522×`) on `block-0`, `1.262 ms` (`0.380× Open`, `0.835× C13`) on `block-143`, and `1.002 ms` (`0.847× C13`) on `runs`**) pinpoints two exact engineering defects introduced when adding the general `wide == 1` path into `g5_fox64_c14.c`:
1. **Inlined `F64_WIDE(0..7)` enlarged `bp4k_emit_fox64`'s stack frame & register pressure** even though `k->wide == 0` on `10/10` blocks (`probe_d32_opt.c` delegated `if (k->wide)` to a `noinline` cold function, keeping `bp4k_emit_fox64` tight in GPRs);
2. **`count_raw_fox_words` walked `tops[sa]` (`R4Top` linked list) a second time** before Phase 3 (`+0.02–0.05 ms`), whereas Theorem 13.2's $O(1)$ counter `w_raw_ub = n_active_tops + 255` bounds $W_{\text{raw}}$ in **zero memory accesses**!

---

## 5. Reproducibility Commands

```bash
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-carbon-cycle14/fox64_u8fq_bp4k.toml
python3 docs/team/acquisition/check_manifest.py docs/journal/2026-11-20-carbon-cycle14/fox64_bzq_bp4k.toml
python3 -u docs/journal/2026-11-20-carbon-cycle14/verify_cycle14.py
gcc -O1 -g -mavx2 -mbmi -mbmi2 -fsanitize=address,undefined -Werror=sequence-point -Wall -Wextra -Werror \
    docs/journal/2026-11-20-carbon-cycle14/g5_fox64_c14.c -o /tmp/g5_fox64_c14_asan && /tmp/g5_fox64_c14_asan /tmp/admitted-bwt/admitted-bwt 1
gcc -O3 -march=native -Werror=sequence-point -Wall -Wextra -Werror \
    docs/journal/2026-11-20-carbon-cycle14/g5_fox64_c14.c -o /tmp/g5_fox64_c14 && /tmp/g5_fox64_c14 /tmp/admitted-bwt/admitted-bwt 25
```
