# Carbon Cycle 21: Producer-Side OoO Super-Edge Contraction (`g -> g^k`) & Fast Kill

**Date:** 2026-11-21  
**Branch:** `research/carbon-word-transport`  
**Baseline:** `origin/master` at `90e67196` (`mibz2 0.5.11` tree)  
**Artifact:** [c21_superedge.rs](file:///home/antigravity/lmmx/mibz2/docs/journal/2026-11-21-carbon-cycle21/c21_superedge.rs)

---

## 1. Hypothesis: Breaking the Non-Jumping Floor ($D_{\text{serial}} < m_{16}$) Without Touching `emit`

Wormhole's $D_{\text{serial}}(\mu) \ge \lceil m_{16}/\mu \rceil$ lower bound relies on *non-jumping* preprocessing (`1` retained row emitted per step). Cycle 20 proved that adding instructions inside `emit` loses natively because `origin/master`'s `emit` executes only ~11 instructions (`~15–20` cycles) per step.

**Cycle 21 Hypothesis (`contract_superedges`):** Leave `emit` **100% unchanged** (`0` extra instructions or branches per `emit` step), and instead run a branchless (`cmov`) reverse linear producer scan `for j in (0..m).rev()` over `edges[..m]`. Whenever $k = g(j) > j$ and $k \ne \text{mark}$ with $\text{len}(c_1) + \text{len}(c_2) \le 16$, concatenate $w(c_1) \circ w(c_2)$ into a new `super_code` in `inline16` (capped at `4,096` total entries = `64 KiB`, deduplicated via an L1d-resident `2,048`-slot hash cache) and overwrite `edges[j] = (super_code << bits) | g(k)`. By induction on reverse index order with $k > j$ and $k \ne \text{mark}$, no contracted super-edge ever crosses `mark`, and ascending chains $v_t < v_{t+1} < \dots$ collapse multiple steps into a single `16 B` `emit` copy.

Byte-exact parity (`warm` + `fresh` + `Plain`) passed across all `431` blocks of `wikidata.bz2`, and `W = 10` max worker heap stayed within `+12.5 KiB` (`1,268.7 -> 1,281.2 KiB`).

---

## 2. Headline Native Safe-Rust A/B + A/A Results (`W = 1` and `W = 4`)

| Metric / Workload | Workers | `A/A` Control (`MasterB / MasterA`) | `SuperEdge / MasterA` (Cycle 21) |
|---|---:|---:|---:|
| `six-sum` (`6` admitted blocks, kernel) | `W = 1` | `0.9982×` (`11.62 / 11.60 ms`) | **`1.2893×`** (`11.57 / 14.92 ms`) |
| `dense-sum` (`5` dense blocks, kernel) | `W = 1` | `0.9926×` (`12.78 / 12.69 ms`) | **`1.2969×`** (`12.75 / 16.53 ms`) |
| `wiki-open233` (`233` Open blocks, kernel) | `W = 1` | `1.0110×` (`355.3 / 359.2 ms`) | **`1.2673×`** (`358.1 / 453.8 ms`) |
| `wiki-all431` (`431` blocks, kernel) | `W = 1` | `1.0110×` (`355.6 / 359.5 ms`) | **`1.2672×`** (`358.4 / 454.1 ms`) |
| `six-pass` (barrier-synced homogeneous) | `W = 4` | `1.0078×` (`23.65 / 23.84 ms`) | **`1.2557×`** (`23.96 / 30.08 ms`) |
| `dense-pass` (barrier-synced homogeneous) | `W = 4` | `1.0062×` (`27.99 / 28.16 ms`) | **`1.2348×`** (`27.64 / 34.13 ms`) |
| `wiki-open233` (barrier-synced homogeneous) | `W = 4` | `0.9973×` (`757.2 / 755.2 ms`) | **`1.2537×`** (`749.8 / 940.0 ms`) |
| `wiki-all431` (barrier-synced homogeneous) | `W = 4` | `0.9980×` (`757.6 / 756.1 ms`) | **`1.2536×`** (`750.2 / 940.5 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 1` | `1.0108×` (`434.9 / 439.6 ms`) | **`1.2170×`** (`439.0 / 534.2 ms`) |
| `E2E BzReader` (`wikidata.bz2` full decode) | `W = 4` | `1.0067×` (`292.5 / 294.5 ms`) | **`1.1635×`** (`290.6 / 338.2 ms`) |

---

## 3. `chaingrind` Diagnostic & Why Post-Wire Pointer Jumping Fails

| Arm (`six`, `5,399,692` B) | Instructions | Proxy Cycles | `dep_past_l2_loads` | `dep_any_loads` | `max_chain_past_l2` | `l2_miss` | DRAM B/outB |
|---|---:|---:|---:|---:|---:|---:|---:|
| `origin/master` (`90e67196`) | `88,300,666` | `38,485,418` | `395,446` | `5,237,339` | `2,881` | `517,730` | `0.798` |
| `SuperEdge` (`MAX_SUPER = 4096`) | `144,959,517` | `52,482,521` | `401,594` (`+1.6%`) | `9,698,025` (`+85%`) | `2,219` | `600,021` (`+16%`) | `0.824` |

The diagnostic pinpoints two fatal structural conservation laws against post-wire `edges` pointer jumping:
1. **In-place store-to-load aliasing:** Mutating `edges[j]` in-place while loading `edges[safe_k]` from the same `&mut [u32]` slice creates store-to-load disambiguation stalls and serializes `dep_any_loads` (`5.24M -> 9.70M`), while adding `0.5 m` random `edges[k]` misses on top of the uncontracted `emit` misses (`dep_past_l2_loads` actually rose from `395k -> 402k`, even as `max_chain_past_l2` fell `2,881 -> 2,219`).
2. **Instruction conservation:** Spending ~28 instructions per row across all $m$ rows in a fourth producer pass (`+56.7M` instructions on `six`) costs more wall time (`+300–800 µs`/block) than the `~25%` step reduction saves in `emit`.

---

## 4. Kill Decision

- **`SuperEdge` (`contract_superedges`):** **KILLED** — post-wire `g -> g^k` contraction over `edges[..m]` adds `+64.2%` instructions, `+15.9%` L2 misses, and store-to-load aliasing stalls without net reduction in `dep_past_l2_loads`, losing `1.267×` (`W = 1`) and `1.254×` (`W = 4`) natively against `origin/master`.
