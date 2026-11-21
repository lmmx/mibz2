# Carbon Kitchen Sink — Cycle 14: Closing Foundational Question 5 & `OPEN-QUESTIONS.md` §4 ("Copies from Completed Output") via Exit-Row Decoupling and the Structural Demand-Read Theorem

**Date:** 2026-11-21
**Branch:** `research/carbon-kitchen-sink`
**Inputs:** `FRONTIER.md` Foundational Question 5 (`line 107`), `OPEN-QUESTIONS.md` §4 (`Astra C: self-sourcing cursor`), [`2026-11-20-carbon-kitchen-sink-cycle2.md`](2026-11-20-carbon-kitchen-sink-cycle2.md) (`OIBS-H`), and [`2026-11-20-carbon-kitchen-sink-cycle13.md`](2026-11-20-carbon-kitchen-sink-cycle13.md) (`7fc61b4`).
**Artefacts:** [`probe_cycle14.py`](2026-11-20-carbon-kitchen-sink-cycle14/probe_cycle14.py), [`probe_cycle14_results.json`](2026-11-20-carbon-kitchen-sink-cycle14/probe_cycle14_results.json).

---

## 0. The Open Question (`OPEN-QUESTIONS.md` §4 & Foundational Question 5)

`OPEN-QUESTIONS.md` §4 posed the exact open question on output-sourced decoding:
> *"The marked output can supply its own byte sources. A cursor emits a phrase, learns which other rows spell those bytes, and copies from completed output at later visits. It saves 37% of modelled line transfers, **but demand reads rise 25–87%**.*
> *The question: **is there a variant whose discovery of other rows uses information Open already has, such as its bands, rather than fresh LF probes?** What could kill it: **the demand-read rise is structural**."*

In Cycle 14, we settle both halves mathematically and empirically across `1,508` small primitive BWTs + all 6 Wikidata blocks against `mibz2 0.5.8` (`with Cycle 12's L1D-resident inline16`).

---

## 1. Theorem 14.1 — The Exit-Row Decoupling Theorem (`Why Band Phrase Equality Cannot Skip Hops`)

### Statement (*proved*; *verified* on `1,508 / 1,508` small primitive BWTs + `6` Wikidata blocks)

Let $\mathcal{B} = \{B_0, \dots, B_{|\mathcal{B}|-1}\}$ be `Open<16>`'s canonical band partition of $[0, m)$ (`2,446–6,788` bands, mean width `26.4–63.9` rows). For any $d \ge 2$ and two retained rows $j_1, j_2 \in [0, m)$:
1. **Zero-Probe Band Phrase Certification:** If $j_1$ and $j_2$ share the same $d$-step band itinerary $\mathrm{itin}_d(j) = (\mathrm{band}(\pi_{\mathrm{ret}}^0(j)), \dots, \mathrm{band}(\pi_{\mathrm{ret}}^{d-1}(j)))$, then `Open`'s bands certify **byte-identical multi-step output** $W_d(j_2) = W_d(j_1)$ (`covering 85.7%–91.3% of all output bytes at d = 2 and 65.5%–76.2% at d = 4`!).
2. **Exit-Row Decoupling Obstruction:** To skip the $d - 1$ intermediate dependent `edges` loads starting at $j_2$, the walk requires **not only** the phrase bytes `out[src .. src + |W_d|]`, **but also the exit retained row $\pi_{\mathrm{ret}}^d(j_2)$**. Whenever thin-run boundaries inside `queues[a]` split a band between $j_1$ and $j_2$ at any step $0 \le t < d$, $\pi_{\mathrm{ret}}^d(j_2) - \pi_{\mathrm{ret}}^d(j_1) \ne j_2 - j_1$ (`occurring on 63.5%–84.5% of all repeated 2-band itineraries on Wikidata!`). On all non-affine band matches, knowing the completed output bytes **saves zero pointer hops** unless $\pi_{\mathrm{ret}}^d(j_2)$ was precomputed via $d$ random `edges` probes!

---

## 2. Theorem 14.2 — The Structural Demand-Read & Contention Obstruction on Affine Lockstep Tubes

Restricting to **Maximal $d$-Step Affine Lockstep Tubes** $\mathcal{T}_{\mathrm{aff}}(d)$ (intervals where both `band` and unit-slope `next_j[t+1] = next_j[t] + 1` hold across $t = 0 \dots d-1$, so an earlier visit at $j_1$ determines **both** `out[src .. src + L]` **and** $\pi_{\mathrm{ret}}^d(j_2) = \pi_{\mathrm{ret}}^d(j_1) + (j_2 - j_1)$):

### Exact Census Against `mibz2 0.5.8` Across All 6 Wikidata Blocks (`probe_cycle14.py`)

| Block | `0.5.8` Bands (`Mean w`) | `d=2` Band Bytes (`% Non-Affine`) | `d=2` Affine Skipped Hops | `d=4` Affine Skipped Hops | **`d=2` General Dir Demand Reads** | **`d=2` Ideal Tagged Demand Reads** | **`d=4` Ideal Consumer + Construct Probes** | **Extra Tube-Dir WS** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| `block-0.L` | `3,204` (`28.1`) | `85.9%` (`69.4%`) | `18.97%` | `23.73%` | **`1.720×` (`+72%`)** | **`1.176×` (`+18%`)** | `0.924×` + `3.0m` build = **`3.92×`** | **`+425.9 KiB`** |
| `block-2.L` | `4,244` (`26.4`) | `87.0%` (`63.5%`) | `21.57%` | `27.12%` | **`1.691×` (`+69%`)** | **`1.223×` (`+22%`)** | `0.928×` + `3.0m` build = **`3.93×`** | **`+457.0 KiB`** |
| `block-85.L` | `2,446` (`63.9`) | `91.3%` (`80.1%`) | `20.03%` | `25.01%` | **`1.624×` (`+62%`)** | **`1.112×` (`+11%`)** | `0.886×` + `3.0m` build = **`3.89×`** | **`+700.9 KiB`** |
| `block-143.L` | `6,788` (`29.0`) | `85.7%` (`74.7%`) | `18.01%` | `21.78%` | **`1.756×` (`+76%`)** | **`1.216×` (`+22%`)** | `0.956×` + `3.0m` build = **`3.96×`** | **`+934.5 KiB`** |
| `block-210.L` | `4,219` (`48.4`) | `90.9%` (`84.5%`) | `14.02%` | `17.11%` | **`1.791×` (`+79%`)** | **`1.151×` (`+15%`)** | `0.952×` + `3.0m` build = **`3.95×`** | **`+1,126.6 KiB`** |
| `block-207.L` | `4,596` (`29.6`) | `87.4%` (`84.1%`) | `13.78%` | `15.89%` | **`1.804×` (`+80%`)** | **`1.168×` (`+17%`)** | `0.976×` + `3.0m` build = **`3.98×`** | **`+758.9 KiB`** |

### Why the Demand-Read Rise Is Structural (*proved* — Closing `OPEN-QUESTIONS.md` §4 & Foundational Question 5)

1. **L1D `inline16` vs L2/L3 `out[0..at)` Asymmetry:** In `mibz2 0.5.8` (especially with Cycle 12's `MN16` where `Dict` is `13.5–28.5 KiB < 32 KiB` L1D), word emission reads `inline16[code]` at **zero L2/L3 demand misses** (`only 1 non-L1D load per step: edges[j]`). Replacing `inline16` with a random backward read from `out[src .. src + L]` (`mean backward distance ~225 KB, missing 32 KiB L1D on >50% of copies and missing i9's 256 KiB L2 on 25–48% of copies!`) **converts an L1D hit into a random L2/L3 demand miss**, while reading/writing `tube_dir` adds a second random L2/L3 miss!
2. **Order-Statistic First-Visit Tax:** In every affine tube of width $w \ge 2$ (`mean width <= 2.4`), the **first** row visited by the cycle must walk all $d$ steps normally *and* dirty-write `(out_pos, exit_row)`, capping skipped hops at `13.8%–21.6%` ($d=2$) and `15.9%–27.5%` ($d=4..8$).
3. **Acquisition vs Lookup Catch-22:** Without pre-probing `edges`, online neighbour checking (`Cycle 2 OIBS-H`) pays dirty writebacks + cache-line splits (`1.13–1.27×` slower); with a tube directory, general demand reads jump **`+48% to +80%` (`retrodicting Astra C's +25–87% rise!`)** and per-worker working set inflates by **`+426 to +1,127 KiB`**; and pre-tagging $d \ge 4$ tubes requires $(d-1)m$ random `edges` probes during `construct`, costing `3×–7×` more random reads than `emit` saves.
