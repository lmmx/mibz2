# Carbon Kitchen Sink — Cycle 18: $S_m$ Coordinate Gauge Theory, Mattson L2 Shift-Invariance, and the Bipartite Contraction Obstruction

**Date:** 2026-11-21  
**Branch:** `research/carbon-kitchen-sink`  
**Theme:** Symmetric-group coordinate gauges ($\tilde{\pi} = \gamma \circ \pi \circ \gamma^{-1}$), Fenwick-tree Mattson `256 KiB` L2 stack-distance geometry, and Cole–Vishkin / 2-adic bipartite independent-set contraction (`E_{0\to 1}`) on `edges[0..m)`.

---

## 1. Mathematical Framework: Coordinate Gauge Freedom $\gamma \in S_m$ & 2-Adic Bipartite Matchings

Every choice of bijection $\gamma : [0, m) \to [0, m)$ yields an isomorphic representation $(\tilde{\pi} = \gamma \circ \pi_{\mathrm{emit}} \circ \gamma^{-1},\, \widetilde{\mathrm{code}} = \mathrm{code} \circ \gamma^{-1},\, \widetilde{\mathrm{mark}} = \gamma(\mathrm{mark}))$ spelling the identical output byte stream. While `master` uses the **$L$-domain gauge** $\gamma_L = \mathrm{id}$, two alternative constructions seek to improve L2 locality or shrink $m$ without raising `LIMIT`:
1. **Wheeler / $F$-Bucket & Digram Coordinate Gauges ($\gamma_F, \gamma_{2\mathrm{gram}}$) and MRU Wire Ordering (`HotWire`):** Numbering retained vertices by $F$-thin order $\gamma_F(j) = \mathrm{starts\_thin}[L[x_j]] + r_{L[x_j]}(j)$, by 2-gram context $(L[x_j], F[x_j], j)$, or wiring bytes in ascending `thin[a]` frequency order (`HotWire`) so the heaviest JSON buckets sit at the MRU top of L2 when `emit` starts.
2. **2-Adic Bipartite Independent-Set Contraction (`Bipartite-Bigram`):** Defining $E_{0\to 1} = \{(u, v) : (u \ \& \ 1) == 0 \wedge (v \ \& \ 1) == 1 \wedge v \neq \mathrm{mark}\}$. Because no vertex can be simultaneously odd (as target of an incoming edge in $E_{0\to 1}$) and even (as source of an outgoing edge in $E_{0\to 1}$), $E_{0\to 1}$ is **universally a vertex-disjoint matching** on every directed graph with zero neighbour coordination, covering $\approx m / 4$ cycle edges and trading data-independent parallel reads in `construct` ($\mu \approx 8\text{--}12$) to skip $\approx 20\%$ of serial $\mu = 1$ hops in `emit` via a `4 KiB` L1D `[u8; 2]` bigram table.

### Theorem 18.1 (Digram Gauge Degeneracy $\gamma_{2\mathrm{gram}} \equiv \gamma_F$ & Mattson Shift-Conjugacy Invariance)
1. **Exact Gauge Collapse ($\gamma_{2\mathrm{gram}} \equiv \gamma_F$):** Because $j \in [0, m)$ is ordered monotonically by BWT row $x_j \in [0, n)$ and $F[x]$ is a monotone step function of $x$, lexicographic sorting by $(L[x_j], F[x_j], j)$ produces the **identical** permutation to sorting by $(L[x_j], j)$: $\gamma_{2\mathrm{gram}} = \gamma_F$ identically on every BWT block.
2. **1-Step $\mathrm{LF}$ Conjugacy & Boundary Split Tax:** $\gamma_F$ maps each retained row $x_j$ to its relative thin rank in $\mathrm{LF}(x_j)$, conjugating $\pi_{\mathrm{ret}}$ along its own $\mathrm{LF}$ trajectory up to thick excursions. Consequently, under `16`-entry (`64 B`) cache lines at `256 KiB` (`4,096`-line) LRU capacity, $\gamma_F$ preserves the bulk stack-distance distribution of $\gamma_L$ while introducing up to `256` bucket-boundary cache-line splits, making $\gamma_F$ strictly **worse** than `master`'s $\gamma_L$ on **6 / 6** authenticated blocks (`+157` to `+349` L2 misses).

### Theorem 18.2 (Tag-Entropy Branch Misprediction vs Dictionary Capacity Dilemma for Post-Wire Contraction)
Contracting a size-$\alpha m$ matching $M \subset E(\pi_{\mathrm{emit}})$ (`\alpha \approx 0.18..0.25`) into `edges[u]` faces a two-sided hardware obstruction:
1. **Uninlineable Dictionary Blowup:** Folding arbitrary $(W(u), W(v))$ pairs into unconditional 16-byte `inline16` entries generates `15,000–35,000` distinct digram words (`240–560 KiB`), blowing past both `32 KiB` L1D and `256 KiB` L2.
2. **Tag-Branch Entropy Tax:** Restricting to literal bigrams stored in a `4 KiB` `[u8; 2]` table (`0x8000_0000` tag bit in `edges[u]`) keeps L1D tiny (`+4.0 KiB`), **but** replaces `master`'s `>99.5%`-predictable `len != LONG` branch with a pseudo-random parity branch of firing probability $p \approx 0.18$ and Shannon entropy $H(p) \approx 0.68\text{ bits/step}$, causing `~35,000` branch mispredictions per block (`~175 µs`) that exceed the skipped-hop savings in `emit` (`walk` slows to **`1.050x`**) on top of `+2,095 µs` (`1.384x`) of random `edges[v]` loads in `construct`.

---

## 2. Exact Mattson `256 KiB` L2 Census & Native Safe-Rust Confrontation (`A/B + A/A`)

### 2.1 Fenwick-Tree Exact `256 KiB` (`4,096` Cache Lines) LRU Miss Census

| Block | `m` | `64 B` Lines | `master` $\gamma_L$ Misses (`%` of `m`) | $\gamma_F \equiv \gamma_{2\mathrm{gram}}$ Misses (`%` of `m`) | Oracle $\gamma_{\mathrm{id}}$ Floor (`6.25%`) | $E_{0\to 1}$ Matching Pairs (`%` of `m`) |
|---|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `89,874` | `5,618` | **`14,594` (`16.2%`)** | `14,846` (`16.5%`) | `5,618` (`6.3%`) | `22,351` (`24.9%`) |
| `block-2.L` | `112,070` | `7,005` | **`26,082` (`23.3%`)** | `26,431` (`23.6%`) | `7,005` (`6.3%`) | `27,961` (`24.9%`) |
| `block-85.L` | `156,404` | `9,776` | **`39,067` (`25.0%`)** | `39,319` (`25.1%`) | `9,776` (`6.3%`) | `38,771` (`24.8%`) |
| `block-143.L` | `196,841` | `12,303` | **`82,283` (`41.8%`)** | `82,526` (`41.9%`) | `12,303` (`6.3%`) | `49,330` (`25.1%`) |
| `block-210.L` | `204,281` | `12,768` | **`78,732` (`38.5%`)** | `79,040` (`38.7%`) | `12,768` (`6.3%`) | `51,211` (`25.1%`) |
| `block-207.L` | `135,921` | `8,496` | **`40,093` (`29.5%`)** | `40,250` (`29.6%`) | `8,496` (`6.3%`) | `33,573` (`24.7%`) |

### 2.2 Native Safe-Rust Stage Attribution & Interleaved A/A + A/B Against `0.5.11` `master`

Byte-for-byte parity passed `6 / 6` on all arms (`HotWire` scratch `+0 B`; `Bipartite-Bigram` scratch `+4,096 B`). Stage-attributed verification (`80` interleaved pairs/block on `HotWire`, `45–50` pairs/block + `35` `2T` passes on `Bipartite-Bigram`):

| Candidate Arm | `construct` Ratio (`6`-blk sum) | `walk/emit` Ratio (`6`-blk sum) | `1T` 6-Block Ratio (vs A/A) | `2T` Pass Ratio (`p10` / `p50`) |Scratch Delta |
|---|---:|---:|---:|---:|---:|
| `A/A Control` (`Master2 / Master1`) | — | — | `1.0079x` / `1.0095x` | `0.9992x` / `0.9988x` | `0 B` |
| `HotWire` (Ascending-`thin[a]` MRU wire) | `1.0053x` (`5662.2 / 5632.3 µs`) | `0.9987x` (`5966.8 / 5974.5 µs`) | `1.0154x` (80-rep isolated) | `0.9885x` / `0.9920x` | `0 B` |
| `Bipartite-Bigram` ($E_{0\to 1}$ + `4 KiB` `[u8;2]`) | **`1.3835x`** (`7557.3 / 5462.3 µs`) | **`1.0497x`** (`6156.0 / 5864.4 µs`) | **`1.2186x`** | **`1.2251x` / `1.2060x`** | `+4.0 KiB` |

---

## 3. Cycle 18 Verdicts (One-Line Kills)

- **Kill (`\gamma_F` / `\gamma_{2\mathrm{gram}}` Coordinate Gauges):** Killed by exact Mattson `256 KiB` L2 census because $\gamma_{2\mathrm{gram}} \equiv \gamma_F$ is 1-step $\mathrm{LF}$-conjugate to `master`'s $\gamma_L$ and strictly increases L2 misses (`+157..+349` on `6/6` blocks) while requiring an extra remap pass.
- **Kill (`HotWire` Ascending-Frequency MRU Wire Order):** Killed because 80-rep stage attribution proves `walk` gains only `0.13%` (`0.9987x`, noise) while sorting 256 byte buckets adds `+0.53%` in `construct` (`1.0154x` end-to-end vs A/A `1.0079x`).
- **Kill (`Bipartite-Bigram` $E_{0\to 1}$ Contraction):** Killed (`1.2186x` 1T, `1.2251x` 2T) because random `edges[v]` pre-loads add `+38.4%` to `construct` AND the `18%`-taken bigram tag branch adds `+5.0%` branch-mispredict penalty to `walk/emit` despite skipping `~20%` of vertices.
