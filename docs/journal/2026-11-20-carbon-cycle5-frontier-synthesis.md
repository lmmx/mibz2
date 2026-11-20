# Carbon Cycle 5 · Optimal Transport & Learned-Coordinate Anti-Smoothness, the Scoped GUID External-Memory Floor, and Closing the Pipeline Boundaries (Obligation #5)

*20 November 2026 · Carbon (`research/carbon-word-transport`, Cycle 5).*

## 0. Cycle-Boundary Check (`origin/master` Fetch — `#664`)

Fetching `origin/master` (`c2bd8c7a`) at the Cycle 4 → Cycle 5 boundary surfaced [`2026-11-20-open-u16q.md`](https://github.com/lmmx/mibz2/commit/c2bd8c7) (`#664`):
- Safe-Rust `U16Q` (`16`-wide queue fill + `[u8; 16]` 1-lane emit) passed `200,000` random-walk checks and Miri, and measured **`−0.1%` (`0.400 → 0.399 s` at 1 thread) / `+0.0%` (`0.189 → 0.190 s` at 4 threads)** across all `431` Wikidata blocks on `Xeon 6975P-C` (`not adopted`), adding a sixth row to `CONTEXT.md` §5a's fidelity ledger (`LLVM`'s `Range<u32>` `Vec::extend` codegen did not suffer GCC's scalar loop penalty, and safe-Rust slice bounds checks absorbed the 1-lane `[u8; 16]` copy).
- Crucially, `#664` anchors the whole-pipeline scale: `0.400 s` across `431` blocks is **`~0.93 ms/block` for the entire bzip2 decode (`Huffman + MTF/RLE2 + Inverse BWT + RLE1 + CRC-32`)**. Cycle 5 therefore targets the remaining unexamined foundational questions in `FRONTIER.md` and `2026-11-20-kitchen-sink.md`: learned piecewise-linear global coordinates (`PGM-index`), optimal transport, random-permutation / external-memory bounds on the GUID wall, and **Obligation #5 / Foundational Question #4** (*"Starting from the decoded column / Is the inverse BWT the right problem?"* across both the pre-BWT MTF/RLE2 boundary and the post-BWT RLE1/CRC-32 boundary).

---

## 1. Explore: Four Foundational Formulations Across Kitchen-Sink & Pipeline Boundaries

Mode: **Reorientation** (`METHOD.md` §1 & §3a):

| # | Candidate / Framework | Mathematical setting | Mechanism & information needed | Likely mathematical or economic trap |
|---|---|---|---|---|
| 1 | **`OT-PGM-Coord`** | 1D Monge–Kantorovich optimal transport & learned piecewise-linear indexes (`kitchen-sink.md` lines 134–140, `PGM-index`) | Approximate $\theta(x) = \mathrm{SA}[x]$ (or `LF`) by $\varepsilon$-bounded linear segments (`practitioner's version of kα`) + exact local correction | Right-context sorting mixes occurrences from across $[0, n)$, making $\mathrm{SA}[x]$ vertically anti-smooth ($\Theta(n)$ segments even at $\varepsilon = 1024$) |
| 2 | **`GUID-EM-Dirichlet`** | Aggarwal–Vitter external-memory permutation lower bounds & Poisson–Dirichlet / $\mathrm{Beta}(1, S-1)$ order statistics (`kitchen-sink.md` Row 6; `working-backwards.md` Q1) | Establish the exact scoped I/O & representation floor of the $M$-UUID family $\mathcal{T}_M$ and the exact segment-imbalance law of $S$-cut cycles | Bounds cache-line transfers in $(K, Z, B_{\text{line}})$, not wall-clock cycles directly (`METHOD.md` §6) |
| 3 | **`PreBWT-MTF-MaxRun`** | BWT pipeline front-end (`FRONTIER.md` Obligation #5 & FQ #4; `src/runs.rs`) | Use MTF literal bijection (`s >= 2`) and `runs.rs`'s uncapped maximal runs + `counts[256]` directly, bypassing `CAP = 127` record splitting | Pre-`EOB` `C[c]` non-causality forbids inversion before `EOB`; removing `CAP = 127` re-coalescing may have `< 7%` G4 headroom |
| 4 | **`L1-Fused-PostBWT`** | Stateful transducer composition (`Open::emit ∘ RLE1 ∘ CRC-32/BZIP2` across `src/open.rs → src/rle.rs → src/crc.rs`) | Exploit forward emission order of `Open::emit` via a `4 KiB` L1 window feeding stateful AVX2 `unrle` + 64B `PCLMULQDQ` `crc32`, eliminating `s.ll` (`900 KB/worker`) | `unrle_avx2 + clmul_crc32` already run at `~8.5 GB/s` (`~0.11 ms`), so staging into a `4 KiB` buffer may lose on 1-thread fusion overhead (`METHOD.md` §7) |

---

## 2. Triage & Foundational Theorems (`verify_cycle5.py`)

All verifications and censuses below are reproduced by [`verify_cycle5.py`](2026-11-20-carbon-cycle5/verify_cycle5.py).

### Result 2.1 · Monge–Kantorovich Factorisation & Vertical Anti-Smoothness of $\mathrm{SA}[x]$ (`proved` + `measured`, closing `kitchen-sink.md` Learned-Coordinate Lead)

**Theorem 5.1 (1D Optimal-Transport Factorisation & $\mathrm{PGM}$ Anti-Smoothness — `proved` + `measured`).**
1. *Per-symbol optimal transport:* For every symbol $c \in \Sigma$ and every convex cost $\Phi(|y - x|)$ ($p \ge 1$), stable $\mathrm{LF}|_{L^{-1}(c)} : L^{-1}(c) \to F^{-1}(c)$ is the **unique** Monge–Kantorovich optimal transport bijection between the empirical support $L^{-1}(c)$ and the contiguous block $F^{-1}(c)$ (by Hardy–Littlewood–Pólya / 1D monotone rearrangement). Global $\mathrm{LF}$ is a $\sigma$-colour multi-marginal gluing of 1D optimal transports, explaining why intra-symbol rank is $Q = 0$ sequential (`Open`'s `queues[256]`) while cross-symbol composition is non-monotone.
2. *Anti-smoothness of $\theta(x) = \mathrm{SA}[x]$:* Unlike `LF(x)` ($r$ exact linear pieces, where `Open` already evaluates the eliminated map in 1 `u32` load), the global text coordinate $\theta(x) = \mathrm{SA}[x]$ is **maximally anti-smooth** in $x$: even inside a single wide run (`L[x] = L[x+1]`), rows $x$ and $x+1$ represent distinct occurrences of the same right-context scattered across $[0, n)$. Using the exact $O(n)$ shrinking-cone algorithm (Ferragina–Vinciguerra `PGM-index` optimal piecewise-linear segmentation for $|\mathrm{SA}[x] - (a x + b)| \le \varepsilon$):

| Block | Mean jump $\mathbb{E}_x[\|\mathrm{SA}[x+1] - \mathrm{SA}[x]\|] / n$ | Optimal `PGM` segments at $\varepsilon = 64$ | Optimal `PGM` segments at $\varepsilon = 1,024$ |
|---|---:|---:|---:|
| `block-0.L` | `0.145n` (`130,490` B) | `445,421` (**`0.495n`**) | `434,587` (**`0.483n`**) |
| `block-2.L` | `0.161n` (`144,900` B) | `445,482` (**`0.495n`**) | `431,829` (**`0.480n`**) |
| `block-85.L` | `0.076n` (`68,395` B) | `427,633` (**`0.475n`**) | `318,864` (**`0.354n`**) |
| `block-143.L` | `0.153n` (`137,680` B) | `439,405` (**`0.488n`**) | `409,572` (**`0.455n`**) |
| `block-207.L` | `0.155n` (`139,500` B) | `440,218` (**`0.489n`**) | `402,431` (**`0.447n`**) |
| `block-210.L` | `0.094n` (`84,596` B) | `371,884` (**`0.413n`**) | `315,595` (**`0.351n`**) |

Even at a coarse error tolerance of $\pm 1,024$ text bytes, a learned piecewise-linear model of $\mathrm{SA}[x]$ requires **`315,595`–`434,587` linear segments (`0.35n–0.48n` — `3.5–4.8×` MORE entries than `Open`'s entire `edges[0..m]` table!)**, plus an $O(n)$ inversion to fit the model in the first place. **Killed.**

### Result 2.2 · The Scoped GUID Wall Floor & Random-Cycle Segment Law (`proved` + `measured`, addressing `working-backwards.md` Q1 & `kitchen-sink.md` Row 6)

**Theorem 5.2 (Scoped External-Memory Lower Bound & Dirichlet Segment Law — `proved` + `measured`).**
1. *Scoped I/O & representation floor on $\mathcal{T}_M$:* For the family $\mathcal{T}_M$ of $2^{122 M}$ blocks embedding $M$ independent 122-bit v4 UUIDs into a fixed template (`Cycle 3 Proposition 2.1`), (a) any intermediate state determining `text` requires $\ge 122 M$ bits; and (b) in the Aggarwal–Vitter external-memory model with cache capacity $Z$ words and cache-line width $B_{\text{line}}$ words (`B_line = 16` `u32`s = `64 B`), realising the induced permutation on the $K = \Theta(M)$ interior singleton hex rows (which realizes $\ge 2^{122 M}$ distinct ordering/symbol outcomes across fixed template slots) requires
   $$\text{Transfers}(K, Z, B_{\text{line}}) \ge \Omega\!\left(\min\!\left(K,\; \frac{K}{B_{\text{line}}} \log_{Z / B_{\text{line}}} \frac{K}{B_{\text{line}}}\right)\right)$$
   cache-line transfers (Aggarwal–Vitter 1988). For $K \in [90\text{k}, 204\text{k}]$, $B_{\text{line}} = 16$, $Z_{\text{L1}} / B_{\text{line}} = 512$, $\log_{512}(K/16) \in [1.25, 1.38] < 2$: `Open`'s 256-bucket sequential queue fill (`4 B/row`, `16` entries/line) is already within a small constant of the radix-coalescing transfer floor on the producer side, while 1-lane `emit` pays the uncoalesced $K$ branch of the $\min$ (escaped in latency only by $K$-way MLP overlap or multi-pass radix bucketing whose `> 40 B/row` constant killed `BSSE`). *(Scope: binds external-memory cache-line transfers on unstructured singleton permutations; does not convert transfers to CPU cycles — `METHOD.md` §6.)*
2. *Uniform spacing / Dirichlet segment law on $g^{-1}$ (`measured`, `verify_cycle5.py`):* Under $S = 48$ equidistributed cuts on a random $m$-cycle, normalised segment lengths converge to $\mathrm{Dirichlet}(1, \dots, 1)$ with expected maximum $\mathbb{E}[\max_q |P_q| / (m/S)] = H_{48} = 4.46$. Across the 6 Wikidata blocks, $\max_q |P_q| / (m/48) = \mathbf{3.11,\; 3.58,\; 4.39,\; 4.74,\; 5.21,\; 5.58}$ (median **`4.57` vs theoretical `4.46`**!), confirming quantitatively that Open's retained cycle behaves macroscopically like a uniform random cycle and why $S \ge 6 K$ (`S = 48` for `K = 8`) is both necessary and sufficient to load-balance $K$ lanes (`D_{8,48} = 0.14–0.17m` vs ideal `0.125m`).

### Result 2.3 · Obligation #5 / Foundational Question #4: Exact Pre-BWT MTF Bijection & Post-BWT RLE1 Streamability (`proved` + `verified`)

**Theorem 5.3 (`proved` + `verified` across `873/873` small columns + `6/6` blocks in `verify_cycle5.py`).**
1. *Pre-BWT (`syms → runs`):* In standard bzip2 (`src/runs.rs`), while global cumulative counts $C[c]$ are non-causal prior to `EOB` (forbidding exact `LF` steps before `EOB`) and blocks share zero inter-block BWT state, non-`RUNA`/`RUNB` MTF literals (`s >= 2`, excluding `EOB`) stand in exact bijection with all maximal runs after run $0$ (and run $0$ too whenever $L[0] \neq \min(\Sigma_{\text{used}})$), with width-1 runs after run $0$ characterised identically by `s_k >= 2 && s_{k+1} >= 2` (`lit[k] & lit[k+1]`). Meanwhile, `src/runs.rs` already computes exact `counts[256]`, and its `CAP = 127` record split inflates $R / r = 1.035\text{--}1.102$ (`+3,990` to `+5,284` split records on the 6 blocks).
2. *Post-BWT (`Open::emit → rle::unrle → crc::crc32`):* On the 6 authenticated `900 KB` blocks, RLE1 4-equal-byte escapes occur only **`23`–`83` times per block** (`34, 62, 23, 83, 24, 24`). Because `Open::emit` outputs bytes in strict forward order (unlike `Segmented`'s reverse walk), composing `Open::emit` with a 2-register `(prev, streak)` stateful AVX2 `unrle` + 64B-block `PCLMULQDQ` `crc32` over a `4 KiB` L1 staging buffer is byte- and CRC-exact while eliminating the `900 KB/worker` `s.ll` intermediate buffer! $\square$

---

## 3. Deepen & Confront: Gate G5 Pipeline-Boundary Falsifier (`g5_l1pipe.c` — **FAIL on Wall Time; PASS on `-900 KB/worker` Memory**)

### Mandatory `METHOD.md` §3a Candidate Declaration
- **Mode:** Reorientation of pipeline boundaries (testing `FRONTIER.md` Obligation #5 & FQ #4 across both `runs::front → Open` and `Open → rle::unrle → crc::crc32`; classification deferred to the Coordinator and MONODROMY).
- **Displaced obligation:** Obligation #5 (`CAP = 127` record split/re-coalesce + `all[256]` recount on the pre-BWT side; `900 KB` `s.ll` allocation, write, `unrle` re-read, and cold `out` `crc32` re-read on the post-BWT side).
- **Replacement principle:** Direct `MaxRun + pre_counts` entry on the pre-BWT boundary (`Arm 1`), `4 KiB` L1-windowed stateful `AVX2 unrle + PCLMULQDQ crc32` fusion (`Arm 2`, `l1pipe.toml` **`PASS`** in `check_manifest.py`), and both combined (`Arm 3`).
- **New paid obligations:** In `Arm 2/3`, writing words into `stage[4 KiB]` in L1 before `stream_rle_feed` copies `32 B` AVX2 chunks to `out` (adding an L1 `memcpy` + `slen >= 4096` check inside `Open::emit`).
- **Nearest prior art:** Shipped `src/lib.rs::decode_block` (`s.ll` 900 KB intermediate buffer shared with `Segmented`); `Scalar-cap` (`Astra E`, fused front + Cells).

### Measured 4-Arm Results ([`g5_l1pipe.c`](2026-11-20-carbon-cycle5/g5_l1pipe.c), 25 paired interleaved rounds — `measured`)

| Block | `SepPost` (`Emit+RLE+CRC`) | **`PostOnl` (`RLE+CRC` only!)** | `L1Post` (ms) | `L1/Sep` `post` | `SepTot` (ms) | **`MaxTot` (ms)** | `L1Tot` (ms) | `L1MxTot` (ms) | **`Max/Sep` `tot`** | `L1/Sep` `tot` | `Both/Sep` `tot` |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | 0.639 | **0.107** | 0.667 | `1.045×` | 1.585 | **1.495** | 1.609 | 1.509 | **`0.943×`** | `1.015×` | `0.952×` |
| `block-2.L` | 0.824 | **0.105** | 0.876 | `1.062×` | 1.949 | **1.854** | 2.006 | 1.912 | **`0.952×`** | `1.029×` | `0.981×` |
| `block-85.L` | 1.053 | **0.105** | 1.084 | `1.030×` | 2.026 | **1.896** | 2.055 | 1.939 | **`0.936×`** | `1.014×` | `0.957×` |
| `block-143.L` | 1.644 | **0.109** | 1.740 | `1.059×` | 3.488 | **3.340** | 3.582 | 3.437 | **`0.958×`** | `1.027×` | `0.985×` |
| `block-207.L` | 1.036 | **0.105** | 1.090 | `1.053×` | 2.312 | **2.194** | 2.386 | 2.236 | **`0.949×`** | `1.032×` | `0.967×` |
| `block-210.L` | 1.579 | **0.106** | 1.636 | `1.036×` | 3.014 | **2.876** | 3.075 | 2.939 | **`0.954×`** | `1.020×` | `0.975×` |

**Pre-registered G5 Verdict:** **FAIL on single-thread wall time** (`0/6` cleared `Post <= 0.88×`); **PASS on exactness & `-900 KB/worker` `s.ll` elimination**.

---

## 4. Learn: Both Pipeline Boundaries Quantified (Closing `FRONTIER.md` Obligation #5 & FQ #4)

1. **Why Post-BWT Fusion Has `< 7%` Single-Thread Headroom (`PostOnl = 0.105–0.109 ms`):**
   Shipped `rle::unrle_avx2` + `crc::clmul_update` process `900 KB` in **`0.105–0.109 ms` (`~8.5 GB/s`, `3.1%–6.8%` of `Open + Post`)**! Staging through a `4 KiB` L1 buffer (`L1Fused`) adds an extra L1 `memcpy` + window-threshold branch inside `Open::emit` (`+0.03–0.09 ms`), reproducing `METHOD.md` §7's *fusion can lose* lesson at 1 thread, while proving a clean **`-900 KB/worker` (`-18 MB` at 20 threads) RSS / L3-traffic lever** if `OPEN-QUESTIONS.md` §3 ever finds 20-thread throughput DRAM-bandwidth-bound.
2. **Pre-BWT `CAP = 127` Boundary (`Max/Sep = 0.936–0.958×`):**
   Passing uncapped maximal runs + `runs.counts` saves `4.2%–6.4%` in C (`below Gate G4's 10% threshold`, and likely smaller in Rust given `#664`). Combined with pre-`EOB` `C[c]` non-causality and cross-block independence, **starting before `records` or fusing after `emit` cannot bypass the core inverse-BWT work**.
